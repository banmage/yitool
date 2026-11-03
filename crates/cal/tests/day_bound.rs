use std::path::PathBuf;

use cal::{CalData, CalQuery, CalTime, DayBound};
use chrono::DateTime;

fn data() -> CalData {
    CalData::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/cal")).unwrap()
}

fn run(raw: &str, bound: DayBound) -> cal::CalResult {
    cal::query(
        &data(),
        &CalQuery {
            time: CalTime::Instant(DateTime::parse_from_rfc3339(raw).unwrap()),
            timezone: "Asia/Shanghai".into(),
            day_bound: bound,
        },
    )
    .unwrap()
}

#[test]
fn zizheng_keeps_civil_day_until_midnight() {
    let r = run("2026-11-03T23:30:00+08:00", DayBound::Zizheng);
    assert_eq!(r.effective_date, "2026-11-03");
    assert_eq!(r.ganzhi.day, "辛巳");
}

#[test]
fn zichu_advances_at_2300() {
    let before = run("2026-11-03T22:59:00+08:00", DayBound::Zichu);
    let at = run("2026-11-03T23:00:00+08:00", DayBound::Zichu);
    let late = run("2026-11-03T23:59:00+08:00", DayBound::Zichu);
    assert_eq!(before.effective_date, "2026-11-03");
    assert_eq!(before.ganzhi.day, "辛巳");
    assert_eq!(at.effective_date, "2026-11-04");
    assert_eq!(late.effective_date, "2026-11-04");
    assert_eq!(at.ganzhi.day, late.ganzhi.day);
    assert_ne!(before.ganzhi.day, at.ganzhi.day);
    // 子时（23:00）时柱为子。
    assert_eq!(at.ganzhi.hour.chars().nth(1).unwrap(), '子');
}

#[test]
fn midnight_boundary() {
    let r = run("2026-11-04T00:00:00+08:00", DayBound::Zichu);
    assert_eq!(r.effective_date, "2026-11-04");
    assert_eq!(r.ganzhi.day, "壬午");

    let r = run("2026-11-04T00:59:00+08:00", DayBound::Zichu);
    assert_eq!(r.effective_date, "2026-11-04");
}

#[test]
fn hour_branch_boundaries() {
    let r = run("2026-11-03T01:00:00+08:00", DayBound::Zichu);
    assert_eq!(r.ganzhi.hour.chars().nth(1).unwrap(), '丑');
    let r = run("2026-11-03T02:59:00+08:00", DayBound::Zichu);
    assert_eq!(r.ganzhi.hour.chars().nth(1).unwrap(), '丑');
    let r = run("2026-11-03T03:00:00+08:00", DayBound::Zichu);
    assert_eq!(r.ganzhi.hour.chars().nth(1).unwrap(), '寅');
}
