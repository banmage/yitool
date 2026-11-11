use std::path::PathBuf;

use cal::{CalQuery, CalTime, DayBound};
use chrono::DateTime;
use meihua::{MeihuaInput, MeihuaResult};
use seed::{Mode, SeedInput, SeedScheme};
use store::{RunRecord, RunReport, Store};

fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

fn context() -> app::AppContext {
    app::AppContext::load(resources()).expect("load resources")
}

fn query() -> CalQuery {
    CalQuery {
        time: CalTime::Instant(DateTime::parse_from_rfc3339("2026-11-03T12:00:00+08:00").unwrap()),
        timezone: "Asia/Shanghai".into(),
        day_bound: DayBound::Zichu,
    }
}

fn seed_input(profile: &str) -> SeedInput {
    SeedInput {
        profile_id: profile.into(),
        run_time: "2026-11-03T12:00:00+08:00".into(),
        timezone: "Asia/Shanghai".into(),
        day_bound: "ZICHU".into(),
        mode: Mode::Fixed,
        scheme: SeedScheme::NineColor,
        color: Some(1),
        aux: Some(3),
        traditional: None,
        question_hmac: None,
        bazi_present: false,
        base_seed: None,
    }
}

fn save_once(
    ctx: &app::AppContext,
    store: &Store,
    profile: &str,
    run_id: &str,
) -> (String, MeihuaResult) {
    let chain = ctx.seed(&seed_input(profile)).unwrap();
    let result = ctx
        .meihua(
            MeihuaInput::NineColor {
                color: 1,
                aux: 3,
                q_mod: 0,
                b_mod: 0,
            },
            &query(),
        )
        .unwrap();
    let markdown = ctx
        .meihua_report(
            &result,
            run_id,
            "2026-11-03T12:00:00+08:00",
            "FIXED",
            Some(&chain),
        )
        .unwrap();

    let run = RunRecord {
        run_id: run_id.into(),
        profile_id: profile.into(),
        created_at: "2026-11-03T12:00:00+08:00".into(),
        mode: "FIXED".into(),
        engine_scope: "MEIHUA".into(),
        question_hmac: None,
        question_saved: false,
        bazi_present: false,
        hmac_key_id: None,
        random_snapshot_digest: Some(chain.snapshot_digest_hex()),
        versions: ctx.versions(),
        disclaimer_present: true,
    };
    let report = RunReport {
        run_id: run_id.into(),
        scheme: result.scheme.clone(),
        status: result.status.label().into(),
        original_name: result.original.name.clone(),
        changed_name: result.changed.name.clone(),
        moving_line: result.moving_line,
        formal_allowed: result.formal_allowed,
        cal_json: serde_json::to_string(&ctx.cal_query(&query()).unwrap()).unwrap(),
        result_json: serde_json::to_string(&result).unwrap(),
        report_markdown: markdown.clone(),
    };
    let snapshot_json = serde_json::to_string(&chain.input_snapshot).unwrap();
    app::history::save_run(
        store,
        &run,
        &result.steps,
        &report,
        Some((&snapshot_json, &chain.snapshot_digest_hex(), "RNG-v1")),
    )
    .unwrap();

    (markdown, result)
}

fn open_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::init(dir.path().join("app.db")).unwrap();
    app::history::ensure_default_profile(&store, "2026-11-03T12:00:00+08:00").unwrap();
    (dir, store)
}

#[test]
fn fixed_mode_seed_chain_is_reproducible() {
    let ctx = context();
    let first = ctx.seed(&seed_input("family-a")).unwrap();
    let second = ctx.seed(&seed_input("family-a")).unwrap();

    assert_eq!(first.base_seed_hex(), second.base_seed_hex());
    assert_eq!(first.draw_seed_hex(), second.draw_seed_hex());
    assert_eq!(first.snapshot_digest_hex(), second.snapshot_digest_hex());
    assert_eq!(first.run_seed_hex(), second.run_seed_hex());

    let other = ctx.seed(&seed_input("family-b")).unwrap();
    assert_ne!(first.run_seed_hex(), other.run_seed_hex());
}

#[test]
fn report_contains_seed_summary_and_nine_color_notice() {
    let ctx = context();
    let chain = ctx.seed(&seed_input("family-a")).unwrap();
    let result = ctx
        .meihua(
            MeihuaInput::NineColor {
                color: 1,
                aux: 3,
                q_mod: 0,
                b_mod: 0,
            },
            &query(),
        )
        .unwrap();
    let markdown = ctx
        .meihua_report(
            &result,
            "run-seed",
            "2026-11-03T12:00:00+08:00",
            "FIXED",
            Some(&chain),
        )
        .unwrap();

    for label in ["基础种子", "抽卡子种子", "输入快照摘要", "推演种子"] {
        assert!(markdown.contains(label), "报告缺少 {label}");
    }
    assert!(markdown.contains(&chain.base_seed_hex()));
    assert!(markdown.contains(&chain.snapshot_digest_hex()));
    assert!(markdown.contains("九色为系统自定义扩展象数（L3）"));
    assert!(markdown.contains("不得作为正式结论"));
}

#[test]
fn saved_history_renders_original_report_and_isolates_profiles() {
    let ctx = context();
    let (_dir, store) = open_store();
    store
        .create_profile(&store::Profile {
            profile_id: "family-b".into(),
            display_name: "家人 B".into(),
            created_at: "2026-11-03T12:00:00+08:00".into(),
            is_default: false,
        })
        .unwrap();

    let (markdown, result) = save_once(&ctx, &store, "default", "run-1");

    let detail = app::history::load_run(&store, "default", "run-1")
        .unwrap()
        .expect("history exists");
    assert_eq!(detail.report.report_markdown, markdown);
    assert_eq!(detail.steps.len(), result.steps.len());
    assert_eq!(detail.report.original_name, result.original.name);
    assert!(app::history::load_run(&store, "family-b", "run-1")
        .unwrap()
        .is_none());

    let summaries = app::history::list_runs(&store, "default").unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].status, "PENDING");
    assert!(!summaries[0].formal_allowed);

    assert!(app::history::delete_run(&store, "family-b", "run-1").is_err());
    app::history::delete_run(&store, "default", "run-1").unwrap();
    assert!(app::history::load_run(&store, "default", "run-1")
        .unwrap()
        .is_none());
}

#[test]
fn history_survives_store_reopen_with_the_saved_report() {
    let ctx = context();
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("app.db");
    let markdown = {
        let store = Store::init(&db).unwrap();
        app::history::ensure_default_profile(&store, "2026-11-03T12:00:00+08:00").unwrap();
        let (markdown, _) = save_once(&ctx, &store, "default", "run-restart");
        markdown
    };

    let store = Store::init(&db).unwrap();
    let detail = app::history::load_run(&store, "default", "run-restart")
        .unwrap()
        .expect("history survives reopen");
    assert_eq!(detail.report.report_markdown, markdown);
    assert_eq!(detail.steps.len(), 6);
    assert_eq!(detail.report.status, "PENDING");
}

#[test]
fn history_database_has_no_birth_or_question_plaintext() {
    let ctx = context();
    let (dir, store) = open_store();
    save_once(&ctx, &store, "default", "run-privacy");

    let hits = app::scan_paths(
        &[dir.path().to_path_buf()],
        &[
            app::Needle::new("bazi", "1966-06-06 06:06"),
            app::Needle::new("question", "测试问事原文"),
            app::Needle::new("hmac", "hmac-key-material"),
        ],
    )
    .unwrap();
    assert!(hits.is_empty(), "历史主库不应出现敏感明文：{hits:?}");
}
