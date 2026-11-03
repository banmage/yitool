use std::path::PathBuf;

use cal::{CalData, CalQuery, CalTime, DayBound};
use chrono::DateTime;

fn data() -> CalData {
    CalData::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/cal")).unwrap()
}

fn run(raw: &str) -> cal::CalResult {
    cal::query(
        &data(),
        &CalQuery {
            time: CalTime::Instant(DateTime::parse_from_rfc3339(raw).unwrap()),
            timezone: "Asia/Shanghai".into(),
            day_bound: DayBound::Zichu,
        },
    )
    .unwrap()
}

#[test]
fn year_pillar_switches_at_lichun_not_new_year() {
    // 2026 立春约在 2026-02-04 03:54 +08:00。
    let before = run("2026-02-04T03:00:00+08:00");
    let after = run("2026-02-04T05:00:00+08:00");
    assert_eq!(before.ganzhi.year, "乙巳");
    assert_eq!(after.ganzhi.year, "丙午");
    assert_eq!(before.lunar.year, 2025);
}

#[test]
fn month_pillar_switches_at_jie() {
    // 惊蛰 2026-03-05 约 22:58 +08:00；月建由寅转卯。
    let before = run("2026-03-05T12:00:00+08:00");
    let after = run("2026-03-06T12:00:00+08:00");
    assert_eq!(before.jieqi_month_branch.branch, "寅");
    assert_eq!(after.jieqi_month_branch.branch, "卯");
    assert_ne!(before.ganzhi.month, after.ganzhi.month);
}

#[test]
fn lunar_index_and_jieqi_branch_are_separate() {
    let r = run("2025-08-01T12:00:00+08:00");
    assert_eq!(r.lunar_month_index.lunar_index, 6);
    assert!(r.lunar_month_index.leap);
    // 节气月建仍为未月（小暑后、立秋前），不因闰六月而重复。
    assert_eq!(r.jieqi_month_branch.branch, "未");
}

#[test]
fn next_term_is_after_current() {
    let r = run("2026-11-03T12:00:00+08:00");
    assert_eq!(r.jieqi.current, "霜降");
    assert_eq!(r.jieqi.next, "立冬");
    let now = DateTime::parse_from_rfc3339(&r.solar).unwrap();
    let next = DateTime::parse_from_rfc3339(&r.jieqi.next_at).unwrap();
    assert!(next > now);
}
