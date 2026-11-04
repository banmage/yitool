//! SQLite 存储底座：档案、设置、推演、步骤、随机快照、笔记。
//!
//! 生辰与问事原文不进入任何字段；`runs` 仅保留 `bazi_present` 布尔与 `question_hmac`。

use std::path::Path;

use domain::{StepLog, VersionSet};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SCHEMA_VERSION: &str = "DB-v0.2";

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("数据库错误: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("约束冲突: {0}")]
    Conflict(String),
    #[error("数据格式错误: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub profile_id: String,
    pub display_name: String,
    pub created_at: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub profile_id: String,
    pub mode: String,
    pub input_mode: String,
    pub aux_mode: String,
    pub day_bound: String,
    pub timezone: String,
    pub ab_mode: String,
    pub animation_enabled: bool,
    pub privacy_redact: bool,
}

impl Setting {
    pub fn default_for(profile_id: impl Into<String>) -> Self {
        Self {
            profile_id: profile_id.into(),
            mode: "FIXED".into(),
            input_mode: "NINE_COLOR".into(),
            aux_mode: "ZODIAC".into(),
            day_bound: "ZICHU".into(),
            timezone: "Asia/Shanghai".into(),
            ab_mode: "INDEPENDENT".into(),
            animation_enabled: true,
            privacy_redact: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: String,
    pub profile_id: String,
    pub created_at: String,
    pub mode: String,
    pub engine_scope: String,
    pub question_hmac: Option<String>,
    pub question_saved: bool,
    pub bazi_present: bool,
    pub hmac_key_id: Option<String>,
    pub random_snapshot_digest: Option<String>,
    pub versions: VersionSet,
    pub disclaimer_present: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoreCheck {
    pub schema_version: String,
    pub integrity: String,
    pub foreign_keys: bool,
    pub journal_mode: String,
    pub tables: Vec<String>,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Self { conn })
    }

    pub fn init(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let store = Self::open(path)?;
        store.migrate()?;
        Ok(store)
    }

    pub fn migrate(&self) -> Result<(), StoreError> {
        self.conn.pragma_update(None, "journal_mode", "WAL")?;
        self.conn.execute_batch(SCHEMA_SQL)?;
        self.conn.execute(
            "INSERT INTO meta(key, value) VALUES('schema_version', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![SCHEMA_VERSION],
        )?;
        Ok(())
    }

    pub fn check(&self) -> Result<StoreCheck, StoreError> {
        let integrity: String = self
            .conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        let foreign_keys: i64 = self
            .conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))?;
        let journal_mode: String = self
            .conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        let schema_version: String = self
            .conn
            .query_row(
                "SELECT value FROM meta WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or_else(|| "MISSING".into());
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let tables = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoreCheck {
            schema_version,
            integrity,
            foreign_keys: foreign_keys == 1,
            journal_mode,
            tables,
        })
    }

    /// 备份到新文件（SQLite `VACUUM INTO`）。目标文件必须不存在。
    pub fn backup_to(&self, path: impl AsRef<Path>) -> Result<(), StoreError> {
        let path = path.as_ref();
        let escaped = path.to_string_lossy().replace('\'', "''");
        self.conn
            .execute_batch(&format!("VACUUM INTO '{escaped}'"))?;
        Ok(())
    }

    pub fn create_profile(&self, profile: &Profile) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO profiles(profile_id, display_name, created_at, is_default)
             VALUES(?1, ?2, ?3, ?4)",
            params![
                profile.profile_id,
                profile.display_name,
                profile.created_at,
                profile.is_default as i64
            ],
        )?;
        Ok(())
    }

    pub fn delete_profile(&self, profile_id: &str) -> Result<(), StoreError> {
        let n = self.conn.execute(
            "DELETE FROM profiles WHERE profile_id=?1",
            params![profile_id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(profile_id.into()));
        }
        Ok(())
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Option<Profile>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT profile_id, display_name, created_at, is_default FROM profiles WHERE profile_id=?1",
                params![profile_id],
                |row| {
                    Ok(Profile {
                        profile_id: row.get(0)?,
                        display_name: row.get(1)?,
                        created_at: row.get(2)?,
                        is_default: row.get::<_, i64>(3)? == 1,
                    })
                },
            )
            .optional()?)
    }

    pub fn save_setting(&self, setting: &Setting) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO settings(profile_id, mode, input_mode, aux_mode, day_bound, timezone, ab_mode, animation_enabled, privacy_redact)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)
             ON CONFLICT(profile_id) DO UPDATE SET
               mode=excluded.mode, input_mode=excluded.input_mode, aux_mode=excluded.aux_mode,
               day_bound=excluded.day_bound, timezone=excluded.timezone, ab_mode=excluded.ab_mode,
               animation_enabled=excluded.animation_enabled, privacy_redact=excluded.privacy_redact",
            params![
                setting.profile_id,
                setting.mode,
                setting.input_mode,
                setting.aux_mode,
                setting.day_bound,
                setting.timezone,
                setting.ab_mode,
                setting.animation_enabled as i64,
                setting.privacy_redact as i64
            ],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, profile_id: &str) -> Result<Option<Setting>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT profile_id, mode, input_mode, aux_mode, day_bound, timezone, ab_mode, animation_enabled, privacy_redact
                 FROM settings WHERE profile_id=?1",
                params![profile_id],
                |row| {
                    Ok(Setting {
                        profile_id: row.get(0)?,
                        mode: row.get(1)?,
                        input_mode: row.get(2)?,
                        aux_mode: row.get(3)?,
                        day_bound: row.get(4)?,
                        timezone: row.get(5)?,
                        ab_mode: row.get(6)?,
                        animation_enabled: row.get::<_, i64>(7)? == 1,
                        privacy_redact: row.get::<_, i64>(8)? == 1,
                    })
                },
            )
            .optional()?)
    }

    pub fn insert_run(&self, run: &RunRecord) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO runs(run_id, profile_id, created_at, mode, engine_scope, question_hmac,
                 question_saved, bazi_present, hmac_key_id, random_snapshot_digest,
                 srs_version, ars_version, ir_version, cal_version, rule_set, data_version,
                 tzdata_version, seed_algo, rng_version, disclaimer_present)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
            params![
                run.run_id,
                run.profile_id,
                run.created_at,
                run.mode,
                run.engine_scope,
                run.question_hmac,
                run.question_saved as i64,
                run.bazi_present as i64,
                run.hmac_key_id,
                run.random_snapshot_digest,
                run.versions.srs,
                run.versions.ars,
                run.versions.ir,
                run.versions.cal,
                run.versions.rule_set,
                run.versions.data,
                run.versions.tzdata,
                run.versions.seed_algo,
                run.versions.rng,
                run.disclaimer_present as i64
            ],
        )?;
        Ok(())
    }

    pub fn insert_step(&self, run_id: &str, step: &StepLog) -> Result<(), StoreError> {
        let rule_ids = serde_json::to_string(&step.rule_ids)
            .map_err(|e| StoreError::Invalid(e.to_string()))?;
        let data_refs = serde_json::to_string(&step.data_refs)
            .map_err(|e| StoreError::Invalid(e.to_string()))?;
        let output =
            serde_json::to_string(&step.output).map_err(|e| StoreError::Invalid(e.to_string()))?;
        self.conn.execute(
            "INSERT INTO run_steps(run_id, step_index, engine, step_id, rule_ids_json, data_refs_json,
                 source_ref, rule_level, formula_summary, output_json, status)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                run_id,
                step.step_index as i64,
                step.engine.name(),
                step.step_id,
                rule_ids,
                data_refs,
                step.source_ref,
                step.rule_level.name(),
                step.formula_summary,
                output,
                step.status.label()
            ],
        )?;
        Ok(())
    }

    pub fn insert_random_snapshot(
        &self,
        run_id: &str,
        snapshot_json: &str,
        snapshot_digest: &str,
        rng_version: &str,
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO random_snapshots(run_id, snapshot_json, snapshot_digest, rng_version)
             VALUES(?1,?2,?3,?4)",
            params![run_id, snapshot_json, snapshot_digest, rng_version],
        )?;
        Ok(())
    }

    pub fn run_visible(&self, profile_id: &str, run_id: &str) -> Result<bool, StoreError> {
        let found: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM runs WHERE run_id=?1 AND profile_id=?2",
                params![run_id, profile_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(found.is_some())
    }

    pub fn list_runs(&self, profile_id: &str) -> Result<Vec<String>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT run_id FROM runs WHERE profile_id=?1 ORDER BY created_at DESC")?;
        let rows = stmt
            .query_map(params![profile_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn delete_run(&self, run_id: &str) -> Result<(), StoreError> {
        let n = self
            .conn
            .execute("DELETE FROM runs WHERE run_id=?1", params![run_id])?;
        if n == 0 {
            return Err(StoreError::NotFound(run_id.into()));
        }
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}

const SCHEMA_SQL: &str = r#"
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS profiles (
    profile_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL CHECK(length(display_name) BETWEEN 1 AND 50),
    created_at TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0 CHECK(is_default IN (0,1))
);
CREATE UNIQUE INDEX IF NOT EXISTS profiles_one_default
    ON profiles(is_default) WHERE is_default = 1;

CREATE TABLE IF NOT EXISTS settings (
    profile_id TEXT PRIMARY KEY REFERENCES profiles(profile_id) ON DELETE CASCADE,
    mode TEXT NOT NULL DEFAULT 'FIXED' CHECK(mode IN ('FIXED','CAST')),
    input_mode TEXT NOT NULL DEFAULT 'NINE_COLOR' CHECK(input_mode IN ('NINE_COLOR','TRADITIONAL_NUMBER')),
    aux_mode TEXT NOT NULL DEFAULT 'ZODIAC' CHECK(aux_mode IN ('ZODIAC','MONTH')),
    day_bound TEXT NOT NULL DEFAULT 'ZICHU' CHECK(day_bound IN ('ZICHU','ZIZHENG')),
    timezone TEXT NOT NULL DEFAULT 'Asia/Shanghai',
    ab_mode TEXT NOT NULL DEFAULT 'INDEPENDENT' CHECK(ab_mode IN ('INDEPENDENT','UNIFIED')),
    animation_enabled INTEGER NOT NULL DEFAULT 1 CHECK(animation_enabled IN (0,1)),
    privacy_redact INTEGER NOT NULL DEFAULT 1 CHECK(privacy_redact IN (0,1))
);

CREATE TABLE IF NOT EXISTS runs (
    run_id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL REFERENCES profiles(profile_id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('FIXED','CAST')),
    engine_scope TEXT NOT NULL CHECK(engine_scope IN ('MEIHUA','MEIHUA+JIAOLIN','ALL')),
    question_hmac TEXT,
    question_saved INTEGER NOT NULL DEFAULT 0 CHECK(question_saved IN (0,1)),
    bazi_present INTEGER NOT NULL DEFAULT 0 CHECK(bazi_present IN (0,1)),
    hmac_key_id TEXT,
    random_snapshot_digest TEXT,
    srs_version TEXT NOT NULL,
    ars_version TEXT NOT NULL,
    ir_version TEXT NOT NULL,
    cal_version TEXT NOT NULL,
    rule_set TEXT NOT NULL,
    data_version TEXT NOT NULL,
    tzdata_version TEXT,
    seed_algo TEXT NOT NULL,
    rng_version TEXT NOT NULL,
    disclaimer_present INTEGER NOT NULL DEFAULT 1 CHECK(disclaimer_present IN (0,1)),
    CHECK(mode <> 'CAST' OR random_snapshot_digest IS NOT NULL),
    CHECK(question_hmac IS NULL OR hmac_key_id IS NOT NULL),
    CHECK(question_saved = 0 OR question_hmac IS NOT NULL)
);
CREATE INDEX IF NOT EXISTS runs_by_profile_time ON runs(profile_id, created_at DESC);
CREATE INDEX IF NOT EXISTS runs_by_mode_time ON runs(mode, created_at);

CREATE TABLE IF NOT EXISTS run_steps (
    run_id TEXT NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
    step_index INTEGER NOT NULL,
    engine TEXT NOT NULL,
    step_id TEXT NOT NULL,
    rule_ids_json TEXT NOT NULL,
    data_refs_json TEXT NOT NULL,
    source_ref TEXT,
    rule_level TEXT NOT NULL CHECK(rule_level IN ('L1','L2','L3','PROVISIONAL')),
    formula_summary TEXT,
    output_json TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('OK','PENDING','ERROR')),
    PRIMARY KEY(run_id, step_index)
);

CREATE TABLE IF NOT EXISTS random_snapshots (
    run_id TEXT PRIMARY KEY REFERENCES runs(run_id) ON DELETE CASCADE,
    snapshot_json TEXT NOT NULL,
    snapshot_digest TEXT NOT NULL,
    rng_version TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS notes (
    note_id TEXT PRIMARY KEY,
    run_id TEXT REFERENCES runs(run_id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::init(dir.path().join("app.db")).unwrap();
        (dir, store)
    }

    fn profile(id: &str, default: bool) -> Profile {
        Profile {
            profile_id: id.into(),
            display_name: id.into(),
            created_at: "2026-11-03T12:00:00+08:00".into(),
            is_default: default,
        }
    }

    fn run(id: &str, profile: &str) -> RunRecord {
        RunRecord {
            run_id: id.into(),
            profile_id: profile.into(),
            created_at: "2026-11-03T12:00:00+08:00".into(),
            mode: "FIXED".into(),
            engine_scope: "MEIHUA".into(),
            question_hmac: None,
            question_saved: false,
            bazi_present: false,
            hmac_key_id: None,
            random_snapshot_digest: None,
            versions: VersionSet::default(),
            disclaimer_present: true,
        }
    }

    #[test]
    fn schema_and_isolation() {
        let (_dir, store) = open_temp();
        let check = store.check().unwrap();
        assert_eq!(check.schema_version, SCHEMA_VERSION);
        assert_eq!(check.integrity, "ok");
        assert!(check.foreign_keys);
        assert!(check.tables.contains(&"runs".to_string()));

        store.create_profile(&profile("a", true)).unwrap();
        store.create_profile(&profile("b", false)).unwrap();
        store.insert_run(&run("r1", "a")).unwrap();
        store.insert_run(&run("r2", "b")).unwrap();

        assert!(store.run_visible("a", "r1").unwrap());
        assert!(!store.run_visible("a", "r2").unwrap());
        assert_eq!(store.list_runs("a").unwrap(), vec!["r1"]);
    }

    #[test]
    fn delete_cascades_without_touching_other_profile() {
        let (_dir, store) = open_temp();
        store.create_profile(&profile("a", true)).unwrap();
        store.create_profile(&profile("b", false)).unwrap();
        store.insert_run(&run("r1", "a")).unwrap();
        store.insert_run(&run("r2", "b")).unwrap();
        store.delete_profile("a").unwrap();
        assert!(!store.run_visible("a", "r1").unwrap());
        assert!(store.run_visible("b", "r2").unwrap());
    }

    #[test]
    fn cast_requires_snapshot_digest() {
        let (_dir, store) = open_temp();
        store.create_profile(&profile("a", true)).unwrap();
        let mut r = run("r1", "a");
        r.mode = "CAST".into();
        assert!(store.insert_run(&r).is_err());
    }

    #[test]
    fn backup_roundtrip() {
        let (dir, store) = open_temp();
        store.create_profile(&profile("a", true)).unwrap();
        store.insert_run(&run("r1", "a")).unwrap();
        let backup = dir.path().join("backup.db");
        store.backup_to(&backup).unwrap();
        drop(store);

        let restored = Store::open(&backup).unwrap();
        let check = restored.check().unwrap();
        assert_eq!(check.schema_version, SCHEMA_VERSION);
        assert!(restored.run_visible("a", "r1").unwrap());
    }
}
