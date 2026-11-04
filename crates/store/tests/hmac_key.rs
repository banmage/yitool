use domain::VersionSet;
use store::{Profile, RunRecord, Store};

fn run(run_id: &str, hmac_key_id: Option<&str>, question_saved: bool) -> RunRecord {
    RunRecord {
        run_id: run_id.into(),
        profile_id: "family-a".into(),
        created_at: "2026-11-04T12:00:00+08:00".into(),
        mode: "FIXED".into(),
        engine_scope: "MEIHUA".into(),
        question_hmac: if question_saved {
            Some("aa".repeat(32))
        } else {
            None
        },
        question_saved,
        bazi_present: false,
        hmac_key_id: hmac_key_id.map(str::to_string),
        random_snapshot_digest: None,
        versions: VersionSet::default(),
        disclaimer_present: true,
    }
}

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path().join("yitool.db")).unwrap();
    store
        .create_profile(&Profile {
            profile_id: "family-a".into(),
            display_name: "家庭".into(),
            created_at: "2026-11-04T12:00:00+08:00".into(),
            is_default: true,
        })
        .unwrap();
    (dir, store)
}

#[test]
fn runs_keep_hmac_key_id_for_saved_question() {
    let (_dir, store) = store();
    store
        .insert_run(&run("run-1", Some("key-1"), true))
        .unwrap();

    let key_id: String = store
        .conn()
        .query_row(
            "SELECT hmac_key_id FROM runs WHERE run_id='run-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(key_id, "key-1");
}

#[test]
fn saved_question_requires_hmac_and_key_id() {
    let (_dir, store) = store();
    assert!(store.insert_run(&run("run-1", None, true)).is_err());
    assert!(store
        .insert_run(&run("run-2", Some("key-1"), false))
        .is_ok());

    let mut association_only = run("run-3", None, false);
    association_only.question_hmac = Some("bb".repeat(32));
    assert!(store.insert_run(&association_only).is_err());
}

#[test]
fn key_rotation_does_not_rewrite_history() {
    let (_dir, store) = store();
    store
        .insert_run(&run("run-1", Some("key-1"), true))
        .unwrap();
    store
        .insert_run(&run("run-2", Some("key-2"), true))
        .unwrap();

    let keys: Vec<String> = store
        .conn()
        .prepare("SELECT hmac_key_id FROM runs ORDER BY run_id")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(keys, vec!["key-1".to_string(), "key-2".to_string()]);
}

#[test]
fn schema_has_hmac_key_id_column() {
    let (_dir, store) = store();
    let columns: Vec<String> = store
        .conn()
        .prepare("PRAGMA table_info(runs)")
        .unwrap()
        .query_map([], |row| row.get(1))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert!(columns.iter().any(|c| c == "hmac_key_id"));
}
