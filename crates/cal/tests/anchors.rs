use std::path::PathBuf;

use cal::{CalData, CalQuery, CalTime, DayBound};
use chrono::DateTime;

fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

fn query(data: &CalData, raw: &str) -> cal::CalResult {
    let instant = DateTime::parse_from_rfc3339(raw).unwrap();
    cal::query(
        data,
        &CalQuery {
            time: CalTime::Instant(instant),
            timezone: "Asia/Shanghai".into(),
            day_bound: DayBound::Zichu,
        },
    )
    .unwrap()
}

#[test]
fn documented_anchor_2026_11_03() {
    let data = CalData::load(resources().join("cal")).unwrap();
    let r = query(&data, "2026-11-03T12:00:00+08:00");
    assert_eq!((r.lunar.year, r.lunar.month, r.lunar.day), (2026, 9, 25));
    assert!(!r.lunar.leap);
    assert_eq!(r.ganzhi.year, "丙午");
    assert_eq!(r.ganzhi.month, "戊戌");
    assert_eq!(r.ganzhi.day, "辛巳");
    assert_eq!(r.ganzhi.hour, "甲午");
    assert_eq!(r.xun.xun_head, "甲戌");
    assert_eq!(r.xun.kong_branches, ["申".to_string(), "酉".to_string()]);
    assert_eq!(r.jieqi_month_branch.branch, "戌");
}

#[test]
fn day_pillar_anchors() {
    let data = CalData::load(resources().join("cal")).unwrap();
    assert_eq!(query(&data, "1900-01-31T12:00:00+08:00").ganzhi.day, "甲辰");
    assert_eq!(query(&data, "2000-01-01T12:00:00+08:00").ganzhi.day, "戊午");
}

#[test]
fn lunar_new_year_1900() {
    let data = CalData::load(resources().join("cal")).unwrap();
    let r = query(&data, "1900-01-31T12:00:00+08:00");
    assert_eq!((r.lunar.year, r.lunar.month, r.lunar.day), (1900, 1, 1));
}

#[test]
fn spring_festival_2026_lunar_year_is_2025() {
    let data = CalData::load(resources().join("cal")).unwrap();
    let r = query(&data, "2026-02-04T12:00:00+08:00");
    assert_eq!(r.lunar.year, 2025);
    assert_eq!(r.lunar.month, 12);
}

#[test]
fn leap_month_2025_is_leap_six() {
    let data = CalData::load(resources().join("cal")).unwrap();
    let r = query(&data, "2025-08-01T12:00:00+08:00");
    assert_eq!(r.lunar.month, 6);
    assert!(r.lunar.leap);
    assert_eq!(r.lunar_month_index.lunar_index, 6);
    assert!(r.lunar_month_index.leap);
}
