//! SQLite 存储底座：档案、设置、推演、步骤、随机快照、笔记。
//!
//! 生辰与问事原文不进入任何字段；`runs` 仅保留 `bazi_present` 布尔与 `question_hmac`。

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use domain::{StepLog, VersionSet};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SCHEMA_VERSION: &str = "DB-v0.3";

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
    #[error("未知的 schema 版本: {0}")]
    UnknownSchemaVersion(String),
    #[error("迁移失败: {0}")]
    Migration(String),
}

/// 一次 `migrate` 的结果；用于 CLI 输出与测试。
#[derive(Debug, Clone, Serialize)]
pub struct MigrationReport {
    pub from: String,
    pub to: String,
    pub applied: Vec<String>,
    pub backup: Option<PathBuf>,
}

impl MigrationReport {
    pub fn changed(&self) -> bool {
        !self.applied.is_empty()
    }
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

/// 历史列表使用的非敏感摘要；不解析完整结果 JSON。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub run_id: String,
    pub profile_id: String,
    pub created_at: String,
    pub mode: String,
    pub scheme: String,
    pub status: String,
    pub original_name: String,
    pub changed_name: String,
    pub moving_line: u8,
    pub formal_allowed: bool,
}

/// 保存时生成的报告与结果快照；历史复盘按此原样渲染。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub run_id: String,
    pub scheme: String,
    pub status: String,
    pub original_name: String,
    pub changed_name: String,
    pub moving_line: u8,
    pub formal_allowed: bool,
    pub cal_json: String,
    pub result_json: String,
    pub report_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub note_id: String,
    pub run_id: Option<String>,
    pub body: String,
    pub created_at: String,
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
    path: PathBuf,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Self { conn, path })
    }

    pub fn init(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let (store, _) = Self::init_with_report(path)?;
        Ok(store)
    }

    /// 同 [`Store::init`]，并返回本次迁移报告（是否升级、备份路径）。
    pub fn init_with_report(path: impl AsRef<Path>) -> Result<(Self, MigrationReport), StoreError> {
        let store = Self::open(path)?;
        let report = store.migrate()?;
        Ok((store, report))
    }

    /// 按版本顺序执行迁移；升级前自动备份，失败回滚。
    ///
    /// 新库从 `DB-v0.1` 基线开始，再逐版本升级到 [`SCHEMA_VERSION`]。
    pub fn migrate(&self) -> Result<MigrationReport, StoreError> {
        self.conn.pragma_update(None, "journal_mode", "WAL")?;

        let existing: Vec<String> = self.table_names()?;
        let had_meta = existing.iter().any(|t| t == "meta");
        if !had_meta {
            if !existing.is_empty() {
                return Err(StoreError::UnknownSchemaVersion(
                    "MISSING（存在表但无 meta）".into(),
                ));
            }
            self.ensure_meta()?;
        }

        let current = if had_meta {
            self.read_schema_version()?
                .ok_or_else(|| StoreError::UnknownSchemaVersion("MISSING".into()))?
        } else {
            String::new()
        };

        let start = if current.is_empty() {
            0
        } else {
            MIGRATIONS
                .iter()
                .position(|m| m.version == current)
                .map(|i| i + 1)
                .ok_or_else(|| StoreError::UnknownSchemaVersion(current.clone()))?
        };

        let pending = &MIGRATIONS[start..];
        if pending.is_empty() {
            return Ok(MigrationReport {
                from: if current.is_empty() {
                    "NONE".into()
                } else {
                    current
                },
                to: SCHEMA_VERSION.into(),
                applied: Vec::new(),
                backup: None,
            });
        }

        let backup = if current.is_empty() {
            None
        } else {
            Some(self.backup_before_migration(&current)?)
        };

        let mut applied = Vec::new();
        self.conn.pragma_update(None, "foreign_keys", "OFF")?;
        let result = (|| -> Result<(), StoreError> {
            let tx = self.conn.unchecked_transaction()?;
            for migration in pending {
                tx.execute_batch(migration.sql)?;
                applied.push(migration.version.to_string());
            }
            tx.execute(
                "INSERT INTO meta(key, value) VALUES('schema_version', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![SCHEMA_VERSION],
            )?;
            let violations: i64 =
                tx.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
                    row.get(0)
                })?;
            if violations > 0 {
                return Err(StoreError::Migration(format!(
                    "迁移后外键校验失败：{violations} 条"
                )));
            }
            tx.commit()?;
            Ok(())
        })();
        self.conn.pragma_update(None, "foreign_keys", "ON")?;
        result?;

        Ok(MigrationReport {
            from: if current.is_empty() {
                "NONE".into()
            } else {
                current
            },
            to: SCHEMA_VERSION.into(),
            applied,
            backup,
        })
    }

    fn table_names(&self) -> Result<Vec<String>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn ensure_meta(&self) -> Result<(), StoreError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    fn read_schema_version(&self) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT value FROM meta WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// 迁移前备份。文件名包含旧版本与时间戳，避免覆盖既有备份。
    fn backup_before_migration(&self, current: &str) -> Result<PathBuf, StoreError> {
        if self.path.as_os_str() == ":memory:" {
            return Err(StoreError::Migration("内存数据库无法在迁移前备份".into()));
        }
        let file = self
            .path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "yitool.db".into());
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let backup = self
            .path
            .with_file_name(format!("{file}.bak-{current}-{stamp}"));
        self.backup_to(&backup)?;
        Ok(backup)
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

    pub fn list_profiles(&self) -> Result<Vec<Profile>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT profile_id, display_name, created_at, is_default
             FROM profiles ORDER BY is_default DESC, created_at ASC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Profile {
                    profile_id: row.get(0)?,
                    display_name: row.get(1)?,
                    created_at: row.get(2)?,
                    is_default: row.get::<_, i64>(3)? == 1,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn rename_profile(&self, profile_id: &str, display_name: &str) -> Result<(), StoreError> {
        if display_name.trim().is_empty() {
            return Err(StoreError::Invalid("档案名称不能为空".into()));
        }
        let n = self.conn.execute(
            "UPDATE profiles SET display_name=?1 WHERE profile_id=?2",
            params![display_name.trim(), profile_id],
        )?;
        if n == 0 {
            return Err(StoreError::NotFound(profile_id.into()));
        }
        Ok(())
    }

    pub fn list_run_summaries(&self, profile_id: &str) -> Result<Vec<RunSummary>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT r.run_id, r.profile_id, r.created_at, r.mode,
                    p.scheme, p.status, p.original_name, p.changed_name,
                    p.moving_line, p.formal_allowed
             FROM runs r JOIN run_reports p ON p.run_id = r.run_id
             WHERE r.profile_id=?1
             ORDER BY r.created_at DESC",
        )?;
        let rows = stmt
            .query_map(params![profile_id], |row| {
                Ok(RunSummary {
                    run_id: row.get(0)?,
                    profile_id: row.get(1)?,
                    created_at: row.get(2)?,
                    mode: row.get(3)?,
                    scheme: row.get(4)?,
                    status: row.get(5)?,
                    original_name: row.get(6)?,
                    changed_name: row.get(7)?,
                    moving_line: row.get::<_, i64>(8)? as u8,
                    formal_allowed: row.get::<_, i64>(9)? == 1,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_run(&self, profile_id: &str, run_id: &str) -> Result<Option<RunRecord>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT run_id, profile_id, created_at, mode, engine_scope, question_hmac,
                        question_saved, bazi_present, hmac_key_id, random_snapshot_digest,
                        srs_version, ars_version, ir_version, cal_version, rule_set,
                        data_version, tzdata_version, seed_algo, rng_version,
                        disclaimer_present
                 FROM runs WHERE run_id=?1 AND profile_id=?2",
                params![run_id, profile_id],
                |row| {
                    Ok(RunRecord {
                        run_id: row.get(0)?,
                        profile_id: row.get(1)?,
                        created_at: row.get(2)?,
                        mode: row.get(3)?,
                        engine_scope: row.get(4)?,
                        question_hmac: row.get(5)?,
                        question_saved: row.get::<_, i64>(6)? == 1,
                        bazi_present: row.get::<_, i64>(7)? == 1,
                        hmac_key_id: row.get(8)?,
                        random_snapshot_digest: row.get(9)?,
                        versions: VersionSet {
                            srs: row.get(10)?,
                            ars: row.get(11)?,
                            ir: row.get(12)?,
                            cal: row.get(13)?,
                            rule_set: row.get(14)?,
                            data: row.get(15)?,
                            cal_data: VersionSet::default().cal_data,
                            tzdata: row.get(16)?,
                            seed_algo: row.get(17)?,
                            rng: row.get(18)?,
                            app: VersionSet::default().app,
                            db_schema: SCHEMA_VERSION.into(),
                        },
                        disclaimer_present: row.get::<_, i64>(19)? == 1,
                    })
                },
            )
            .optional()?)
    }

    pub fn save_run_report(&self, report: &RunReport) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO run_reports(run_id, scheme, status, original_name, changed_name,
                 moving_line, formal_allowed, cal_json, result_json, report_markdown)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
             ON CONFLICT(run_id) DO UPDATE SET
               scheme=excluded.scheme, status=excluded.status,
               original_name=excluded.original_name, changed_name=excluded.changed_name,
               moving_line=excluded.moving_line, formal_allowed=excluded.formal_allowed,
               cal_json=excluded.cal_json, result_json=excluded.result_json,
               report_markdown=excluded.report_markdown",
            params![
                report.run_id,
                report.scheme,
                report.status,
                report.original_name,
                report.changed_name,
                report.moving_line as i64,
                report.formal_allowed as i64,
                report.cal_json,
                report.result_json,
                report.report_markdown
            ],
        )?;
        Ok(())
    }

    pub fn get_run_report(
        &self,
        profile_id: &str,
        run_id: &str,
    ) -> Result<Option<RunReport>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT p.run_id, p.scheme, p.status, p.original_name, p.changed_name,
                        p.moving_line, p.formal_allowed, p.cal_json, p.result_json,
                        p.report_markdown
                 FROM run_reports p JOIN runs r ON r.run_id = p.run_id
                 WHERE p.run_id=?1 AND r.profile_id=?2",
                params![run_id, profile_id],
                |row| {
                    Ok(RunReport {
                        run_id: row.get(0)?,
                        scheme: row.get(1)?,
                        status: row.get(2)?,
                        original_name: row.get(3)?,
                        changed_name: row.get(4)?,
                        moving_line: row.get::<_, i64>(5)? as u8,
                        formal_allowed: row.get::<_, i64>(6)? == 1,
                        cal_json: row.get(7)?,
                        result_json: row.get(8)?,
                        report_markdown: row.get(9)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn list_steps(&self, run_id: &str) -> Result<Vec<StepLog>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT step_index, step_id, engine, rule_ids_json, data_refs_json,
                    source_ref, rule_level, formula_summary, output_json, status
             FROM run_steps WHERE run_id=?1 ORDER BY step_index ASC",
        )?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                let engine: String = row.get(2)?;
                let rule_level: String = row.get(6)?;
                let status: String = row.get(9)?;
                let rule_ids: String = row.get(3)?;
                let data_refs: String = row.get(4)?;
                let output: String = row.get(8)?;
                Ok(StepLog {
                    step_index: row.get::<_, i64>(0)? as usize,
                    step_id: row.get(1)?,
                    engine: parse_engine(&engine)?,
                    rule_ids: serde_json::from_str(&rule_ids)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                    data_refs: serde_json::from_str(&data_refs)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                    source_ref: row.get(5)?,
                    rule_level: parse_rule_level(&rule_level)?,
                    formula_summary: row.get(7)?,
                    output: serde_json::from_str(&output)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                    status: parse_status(&status)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn add_note(&self, note: &Note) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO notes(note_id, run_id, body, created_at) VALUES(?1,?2,?3,?4)",
            params![note.note_id, note.run_id, note.body, note.created_at],
        )?;
        Ok(())
    }

    pub fn list_notes(&self, run_id: &str) -> Result<Vec<Note>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT note_id, run_id, body, created_at FROM notes
             WHERE run_id=?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt
            .query_map(params![run_id], |row| {
                Ok(Note {
                    note_id: row.get(0)?,
                    run_id: row.get(1)?,
                    body: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 事务内保存一次完整推演；任一步失败即回滚，不留下半条历史。
    #[allow(clippy::too_many_arguments)]
    pub fn save_run_bundle(
        &self,
        run: &RunRecord,
        steps: &[StepLog],
        report: &RunReport,
        snapshot: Option<(&str, &str, &str)>,
    ) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        self.insert_run(run)?;
        for step in steps {
            self.insert_step(&run.run_id, step)?;
        }
        self.save_run_report(report)?;
        if let Some((snapshot_json, snapshot_digest, rng_version)) = snapshot {
            self.insert_random_snapshot(&run.run_id, snapshot_json, snapshot_digest, rng_version)?;
        }
        tx.commit()?;
        Ok(())
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

const MIGRATION_V1_SQL: &str = r#"
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
    CHECK(question_saved = 0 OR (question_hmac IS NOT NULL AND hmac_key_id IS NOT NULL))
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

/// DB-v0.2：任何已保存的 `question_hmac` 都必须带 `hmac_key_id`。
///
/// SQLite 不能直接修改 CHECK 约束，因此重建 `runs` 并迁移数据。
const MIGRATION_V2_SQL: &str = r#"
CREATE TABLE runs_v2 (
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
INSERT INTO runs_v2 (
    run_id, profile_id, created_at, mode, engine_scope, question_hmac,
    question_saved, bazi_present, hmac_key_id, random_snapshot_digest,
    srs_version, ars_version, ir_version, cal_version, rule_set, data_version,
    tzdata_version, seed_algo, rng_version, disclaimer_present
)
SELECT
    run_id, profile_id, created_at, mode, engine_scope, question_hmac,
    question_saved, bazi_present, hmac_key_id, random_snapshot_digest,
    srs_version, ars_version, ir_version, cal_version, rule_set, data_version,
    tzdata_version, seed_algo, rng_version, disclaimer_present
FROM runs;
DROP TABLE runs;
ALTER TABLE runs_v2 RENAME TO runs;
CREATE INDEX runs_by_profile_time ON runs(profile_id, created_at DESC);
CREATE INDEX runs_by_mode_time ON runs(mode, created_at);
"#;

/// DB-v0.3：保存历史报告，并允许 `VERIFIED/FROZEN/DEPRECATED` 步骤状态。
///
/// 原有 `run_steps` 的 CHECK 不允许 `VERIFIED`；SQLite 不能直接修改 CHECK，
/// 因此重建表并保留全部历史行。
const MIGRATION_V3_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS run_reports (
    run_id TEXT PRIMARY KEY REFERENCES runs(run_id) ON DELETE CASCADE,
    scheme TEXT NOT NULL,
    status TEXT NOT NULL,
    original_name TEXT NOT NULL,
    changed_name TEXT NOT NULL,
    moving_line INTEGER NOT NULL CHECK(moving_line BETWEEN 1 AND 6),
    formal_allowed INTEGER NOT NULL DEFAULT 0 CHECK(formal_allowed IN (0,1)),
    cal_json TEXT NOT NULL,
    result_json TEXT NOT NULL,
    report_markdown TEXT NOT NULL
);

CREATE TABLE run_steps_v3 (
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
    status TEXT NOT NULL CHECK(status IN ('OK','PENDING','ERROR','VERIFIED','FROZEN','DEPRECATED')),
    PRIMARY KEY(run_id, step_index)
);
INSERT INTO run_steps_v3 (
    run_id, step_index, engine, step_id, rule_ids_json, data_refs_json,
    source_ref, rule_level, formula_summary, output_json, status
)
SELECT
    run_id, step_index, engine, step_id, rule_ids_json, data_refs_json,
    source_ref, rule_level, formula_summary, output_json, status
FROM run_steps;
DROP TABLE run_steps;
ALTER TABLE run_steps_v3 RENAME TO run_steps;
"#;

struct Migration {
    version: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: "DB-v0.1",
        sql: MIGRATION_V1_SQL,
    },
    Migration {
        version: "DB-v0.2",
        sql: MIGRATION_V2_SQL,
    },
    Migration {
        version: "DB-v0.3",
        sql: MIGRATION_V3_SQL,
    },
];

fn parse_engine(value: &str) -> rusqlite::Result<domain::Engine> {
    match value {
        "MEIHUA" => Ok(domain::Engine::Meihua),
        "JIAOLIN" => Ok(domain::Engine::Jiaolin),
        "LIUYAO" => Ok(domain::Engine::Liuyao),
        "ZEJI" => Ok(domain::Engine::Zeji),
        "CAL" => Ok(domain::Engine::Cal),
        "SEED" => Ok(domain::Engine::Seed),
        "IR" => Ok(domain::Engine::Ir),
        "RNG" => Ok(domain::Engine::Rng),
        "LOG" => Ok(domain::Engine::Log),
        "STORE" => Ok(domain::Engine::Store),
        "APP" => Ok(domain::Engine::App),
        other => Err(rusqlite::Error::InvalidParameterName(format!(
            "未知引擎: {other}"
        ))),
    }
}

fn parse_rule_level(value: &str) -> rusqlite::Result<domain::RuleLevel> {
    match value {
        "L1" => Ok(domain::RuleLevel::L1),
        "L2" => Ok(domain::RuleLevel::L2),
        "L3" => Ok(domain::RuleLevel::L3),
        "PROVISIONAL" => Ok(domain::RuleLevel::Provisional),
        other => Err(rusqlite::Error::InvalidParameterName(format!(
            "未知规则层级: {other}"
        ))),
    }
}

fn parse_status(value: &str) -> rusqlite::Result<domain::Status> {
    match value {
        "OK" => Ok(domain::Status::Ok),
        "PENDING" => Ok(domain::Status::Pending),
        "ERROR" => Ok(domain::Status::Error),
        "VERIFIED" => Ok(domain::Status::Verified),
        "FROZEN" => Ok(domain::Status::Frozen),
        "DEPRECATED" => Ok(domain::Status::Deprecated),
        other => Err(rusqlite::Error::InvalidParameterName(format!(
            "未知状态: {other}"
        ))),
    }
}

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
    fn settings_are_profile_scoped_and_survive_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("app.db");
        {
            let store = Store::init(&db).unwrap();
            store.create_profile(&profile("a", true)).unwrap();
            store.create_profile(&profile("b", false)).unwrap();
            let mut a = Setting::default_for("a");
            a.day_bound = "ZIZHENG".into();
            let mut b = Setting::default_for("b");
            b.timezone = "UTC".into();
            store.save_setting(&a).unwrap();
            store.save_setting(&b).unwrap();
        }

        let store = Store::init(&db).unwrap();
        let a = store.get_setting("a").unwrap().unwrap();
        let b = store.get_setting("b").unwrap().unwrap();
        assert_eq!(a.day_bound, "ZIZHENG");
        assert_eq!(b.day_bound, "ZICHU");
        assert_eq!(b.timezone, "UTC");
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
    fn v3_history_saves_verified_steps_and_isolates_profiles() {
        let (_dir, store) = open_temp();
        store.create_profile(&profile("a", true)).unwrap();
        store.create_profile(&profile("b", false)).unwrap();
        store.insert_run(&run("r1", "a")).unwrap();

        let step = StepLog {
            step_index: 0,
            step_id: "MEIHUA-QIGUA".into(),
            engine: domain::Engine::Meihua,
            rule_ids: vec!["MEIHUA-QIGUA-001".into()],
            data_refs: vec!["COLOR-NUM-02".into()],
            source_ref: "SRC-MEIHUA-001".into(),
            rule_level: domain::RuleLevel::L3,
            formula_summary: "C mod 8".into(),
            output: serde_json::json!({"upper": "乾"}),
            status: domain::Status::Verified,
        };
        store.insert_step("r1", &step).unwrap();
        store
            .save_run_report(&RunReport {
                run_id: "r1".into(),
                scheme: "NINE_COLOR".into(),
                status: "PENDING".into(),
                original_name: "天火同人".into(),
                changed_name: "风火家人".into(),
                moving_line: 4,
                formal_allowed: false,
                cal_json: "{}".into(),
                result_json: "{}".into(),
                report_markdown: "# report".into(),
            })
            .unwrap();

        let steps = store.list_steps("r1").unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].status, domain::Status::Verified);
        assert_eq!(steps[0].output["upper"], "乾");

        let summaries = store.list_run_summaries("a").unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].original_name, "天火同人");
        assert!(store.list_run_summaries("b").unwrap().is_empty());
        assert!(store.get_run_report("b", "r1").unwrap().is_none());
        assert!(store.get_run_report("a", "r1").unwrap().is_some());

        store.delete_run("r1").unwrap();
        assert!(store.get_run_report("a", "r1").unwrap().is_none());
        assert!(store.list_steps("r1").unwrap().is_empty());
    }

    #[test]
    fn v2_steps_survive_v3_rebuild() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(MIGRATION_V1_SQL).unwrap();
            conn.execute_batch(MIGRATION_V2_SQL).unwrap();
            conn.execute(
                "INSERT INTO meta(key, value) VALUES('schema_version', 'DB-v0.2')",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO profiles(profile_id, display_name, created_at, is_default)
                 VALUES('a', 'a', '2026-11-04T12:00:00+08:00', 1)",
                [],
            )
            .unwrap();
            insert_legacy_run(&conn, "r1", None, None);
            conn.execute(
                "INSERT INTO run_steps(run_id, step_index, engine, step_id,
                     rule_ids_json, data_refs_json, source_ref, rule_level,
                     formula_summary, output_json, status)
                 VALUES('r1', 0, 'MEIHUA', 'MEIHUA-QIGUA', '[\"MEIHUA-QIGUA-001\"]',
                     '[\"COLOR-NUM-02\"]', 'SRC-MEIHUA-001', 'L3', 'C mod 8',
                     '{\"upper\":\"乾\"}', 'PENDING')",
                [],
            )
            .unwrap();
        }

        let store = Store::open(&path).unwrap();
        let report = store.migrate().unwrap();
        assert_eq!(report.applied, vec!["DB-v0.3".to_string()]);
        assert!(report.backup.as_ref().unwrap().exists());

        let steps = store.list_steps("r1").unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_id, "MEIHUA-QIGUA");
        assert_eq!(steps[0].status, domain::Status::Pending);

        let verified = StepLog {
            step_index: 1,
            step_id: "MEIHUA-HUGUA".into(),
            engine: domain::Engine::Meihua,
            rule_ids: vec!["MEIHUA-HUGUA-001".into()],
            data_refs: vec!["G-01".into()],
            source_ref: "SRC-MEIHUA-002".into(),
            rule_level: domain::RuleLevel::L3,
            formula_summary: "互卦".into(),
            output: serde_json::json!({"mutual": "乾"}),
            status: domain::Status::Verified,
        };
        store.insert_step("r1", &verified).unwrap();
        let steps = store.list_steps("r1").unwrap();
        assert_eq!(steps[1].status, domain::Status::Verified);
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

    fn legacy_v1_db(path: &Path) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(MIGRATION_V1_SQL).unwrap();
        conn.execute(
            "INSERT INTO meta(key, value) VALUES('schema_version', 'DB-v0.1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO profiles(profile_id, display_name, created_at, is_default)
             VALUES('a', 'a', '2026-11-04T12:00:00+08:00', 1)",
            [],
        )
        .unwrap();
    }

    fn insert_legacy_run(
        conn: &Connection,
        run_id: &str,
        hmac: Option<&str>,
        key_id: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO runs(run_id, profile_id, created_at, mode, engine_scope,
                 question_hmac, question_saved, bazi_present, hmac_key_id,
                 random_snapshot_digest, srs_version, ars_version, ir_version,
                 cal_version, rule_set, data_version, tzdata_version, seed_algo,
                 rng_version, disclaimer_present)
             VALUES(?1, 'a', '2026-11-04T12:00:00+08:00', 'FIXED', 'MEIHUA',
                 ?2, 0, 0, ?3, NULL, 'SRS-v0.5', 'ARS-v0.2', 'IR-v0.1',
                 'CAL-v0.2', 'RULE-v0.2', 'DATA-v0.2', '2025b', 'SEED-v2',
                 'RNG-v1', 1)",
            params![run_id, hmac, key_id],
        )
        .unwrap();
    }

    #[test]
    fn fresh_db_runs_all_migrations_without_backup() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("app.db")).unwrap();
        let report = store.migrate().unwrap();
        assert_eq!(report.from, "NONE");
        assert_eq!(report.to, SCHEMA_VERSION);
        assert_eq!(
            report.applied,
            vec![
                "DB-v0.1".to_string(),
                "DB-v0.2".to_string(),
                "DB-v0.3".to_string()
            ]
        );
        assert!(report.backup.is_none());
        assert_eq!(store.check().unwrap().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn migrate_is_idempotent() {
        let (_dir, store) = open_temp();
        let report = store.migrate().unwrap();
        assert!(!report.changed());
        assert!(report.backup.is_none());
    }

    #[test]
    fn v1_db_is_backed_up_and_upgraded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        legacy_v1_db(&path);
        {
            let conn = Connection::open(&path).unwrap();
            insert_legacy_run(&conn, "r1", None, None);
        }

        let store = Store::open(&path).unwrap();
        assert_eq!(store.check().unwrap().schema_version, "DB-v0.1");
        let report = store.migrate().unwrap();
        assert_eq!(report.from, "DB-v0.1");
        assert_eq!(
            report.applied,
            vec!["DB-v0.2".to_string(), "DB-v0.3".to_string()]
        );
        let backup = report.backup.expect("升级前必须备份");
        assert!(backup.exists());
        assert!(store.run_visible("a", "r1").unwrap());
        assert_eq!(store.check().unwrap().schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn failed_migration_rolls_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        legacy_v1_db(&path);
        {
            let conn = Connection::open(&path).unwrap();
            // v0.1 允许“仅关联”的 HMAC 不带 key_id；v0.2 不允许。
            insert_legacy_run(&conn, "bad", Some(&"aa".repeat(32)), None);
        }

        let store = Store::open(&path).unwrap();
        let err = store.migrate().unwrap_err();
        assert!(matches!(err, StoreError::Sqlite(_)));
        assert_eq!(store.check().unwrap().schema_version, "DB-v0.1");
        assert!(store.run_visible("a", "bad").unwrap());
    }

    #[test]
    fn unknown_schema_version_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("app.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(MIGRATION_V1_SQL).unwrap();
            conn.execute(
                "INSERT INTO meta(key, value) VALUES('schema_version', 'DB-v9.9')",
                [],
            )
            .unwrap();
        }
        let store = Store::open(&path).unwrap();
        assert!(matches!(
            store.migrate(),
            Err(StoreError::UnknownSchemaVersion(_))
        ));
    }
}
