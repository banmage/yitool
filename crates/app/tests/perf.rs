use std::path::PathBuf;
use std::time::{Duration, Instant};

use cal::{CalData, CalQuery, CalTime, DayBound};
use chrono::{FixedOffset, TimeZone};

fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    let idx = (samples.len() * 95 / 100).min(samples.len() - 1);
    samples[idx]
}

fn report(name: &str, p95: Duration, threshold_ms: u128) {
    eprintln!(
        "perf {}: p95={:?} threshold={}ms os={} arch={}",
        name,
        p95,
        threshold_ms,
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    assert!(
        p95.as_millis() <= threshold_ms,
        "{} p95 {:?} 超过 {}ms",
        name,
        p95,
        threshold_ms
    );
}

#[test]
fn seed_generation_p95() {
    let mut samples = Vec::new();
    for i in 0..200u32 {
        let input = seed::SeedInput {
            profile_id: "family-a".into(),
            run_time: format!("2026-11-03T12:00:{:02}+08:00", i % 60),
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
        };
        let t = Instant::now();
        seed::generate(&input).unwrap();
        samples.push(t.elapsed());
    }
    report("seed", p95(samples), 50);
}

#[test]
fn one_year_calendar_traversal_p95() {
    let data = CalData::load(resources().join("cal")).unwrap();
    let tz = FixedOffset::east_opt(8 * 3600).unwrap();
    let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let mut samples = Vec::new();
    for day in 0..365 {
        let date = start + chrono::Duration::days(day);
        let instant = tz
            .from_local_datetime(&date.and_hms_opt(12, 0, 0).unwrap())
            .single()
            .unwrap();
        let t = Instant::now();
        cal::query(
            &data,
            &CalQuery {
                time: CalTime::Instant(instant),
                timezone: "Asia/Shanghai".into(),
                day_bound: DayBound::Zichu,
            },
        )
        .unwrap();
        samples.push(t.elapsed());
    }
    report("cal-year", p95(samples), 500);
}

#[test]
fn meihua_p95() {
    let mut samples = Vec::new();
    for _ in 0..200 {
        let t = Instant::now();
        meihua::cast(
            meihua::MeihuaInput::NineColor {
                color: 1,
                aux: 3,
                q_mod: 0,
                b_mod: 0,
            },
            10,
        )
        .unwrap();
        samples.push(t.elapsed());
    }
    report("meihua", p95(samples), 200);
}

#[test]
fn history_1000_runs_p95() {
    let dir = tempfile::tempdir().unwrap();
    let store = store::Store::init(dir.path().join("app.db")).unwrap();
    store
        .create_profile(&store::Profile {
            profile_id: "family-a".into(),
            display_name: "家庭".into(),
            created_at: "2026-11-03T12:00:00+08:00".into(),
            is_default: true,
        })
        .unwrap();

    for i in 0..1000u32 {
        store
            .insert_run(&store::RunRecord {
                run_id: format!("run-{i:04}"),
                profile_id: "family-a".into(),
                created_at: format!("2026-11-03T12:{:02}:{:02}+08:00", i / 60 % 60, i % 60),
                mode: "FIXED".into(),
                engine_scope: "MEIHUA".into(),
                question_hmac: None,
                question_saved: false,
                bazi_present: false,
                hmac_key_id: None,
                random_snapshot_digest: None,
                versions: domain::VersionSet::default(),
                disclaimer_present: true,
            })
            .unwrap();
    }

    eprintln!("perf data: history runs=1000 profile=family-a, no cache");
    let mut samples = Vec::new();
    for _ in 0..100 {
        let t = Instant::now();
        let runs = store.list_runs("family-a").unwrap();
        assert_eq!(runs.len(), 1000);
        samples.push(t.elapsed());
    }
    report("history-list-1000", p95(samples), 100);

    let mut samples = Vec::new();
    for i in 0..1000u32 {
        let t = Instant::now();
        assert!(store
            .run_visible("family-a", &format!("run-{i:04}"))
            .unwrap());
        samples.push(t.elapsed());
    }
    report("history-lookup-1000", p95(samples), 10);
}
