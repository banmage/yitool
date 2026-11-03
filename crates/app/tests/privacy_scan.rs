use std::path::PathBuf;

use app::{export_snapshot, scan_paths, Needle};
use domain::VersionSet;
use meihua::{cast, MeihuaInput};

fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

#[test]
fn scan_finds_sensitive_values_in_files() {
    let dir = tempfile::tempdir().unwrap();
    let sensitive = dir.path().join("app.log");
    std::fs::write(
        &sensitive,
        "normal\nbirth=1990-01-01 12:00\nquestion=测试问事原文\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("clean.txt"), "nothing here").unwrap();

    let hits = scan_paths(
        &[dir.path().to_path_buf()],
        &[
            Needle::new("bazi", "1990-01-01 12:00"),
            Needle::new("question", "测试问事原文"),
        ],
    )
    .unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().any(|h| h.kind == "bazi"));
    assert!(hits.iter().any(|h| h.kind == "question"));
}

#[test]
fn clean_tree_has_zero_hits() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("clean.txt"), "nothing here").unwrap();
    let hits = scan_paths(
        &[dir.path().to_path_buf()],
        &[Needle::new("bazi", "1990-01-01 12:00")],
    )
    .unwrap();
    assert!(hits.is_empty());
}

#[test]
fn exported_snapshot_contains_no_bazi_or_question_text() {
    let result = cast(
        MeihuaInput::NineColor {
            color: 1,
            aux: 3,
            q_mod: 0,
            b_mod: 0,
        },
        10,
    )
    .unwrap();
    let seed_input = seed_input();
    let chain = seed::generate(&seed_input).unwrap();
    let snapshot = export_snapshot(
        "run-1",
        "2026-11-03T12:00:00+08:00",
        &chain,
        Some(&result),
        &VersionSet::default(),
    );
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(!json.contains("1990-01-01"));
    assert!(!json.contains("测试问事原文"));
    assert!(!json.contains("bazi_material"));
    assert!(json.contains("\"bazi_present\":false"));
    assert!(json.contains("天火同人"));
}

fn seed_input() -> seed::SeedInput {
    seed::SeedInput {
        profile_id: "family-a".into(),
        run_time: "2026-11-03T12:00:00+08:00".into(),
        timezone: "Asia/Shanghai".into(),
        day_bound: "ZICHU".into(),
        mode: seed::Mode::Fixed,
        scheme: seed::SeedScheme::NineColor,
        color: Some(1),
        aux: Some(3),
        traditional: None,
        question_hmac: None,
        bazi_present: false,
        base_seed: None,
    }
}

#[test]
fn app_context_loads_real_resources() {
    let ctx = app::AppContext::load(resources()).unwrap();
    assert!(!ctx.cal.months.is_empty());
    assert!(ctx.rules.rule("MEIHUA-QIGUA-001").is_some());
    assert!(!ctx.cal.version.is_empty());
}

#[test]
fn scan_detects_plaintext_in_sqlite_database() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("app.db");
    {
        let store = store::Store::init(&db).unwrap();
        store
            .conn()
            .execute_batch(
                "INSERT INTO notes(note_id, run_id, body, created_at)
                 VALUES('n1', NULL, '测试问事原文', '2026-11-03T12:00:00+08:00');",
            )
            .unwrap();
    }
    let hits = scan_paths(
        &[dir.path().to_path_buf()],
        &[Needle::new("question", "测试问事原文")],
    )
    .unwrap();
    assert!(!hits.is_empty(), "SQLite 主库/WAL 中的明文应被扫描到");
}
