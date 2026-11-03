use std::path::PathBuf;

use cal::{CalData, CalError, CalQuery, CalTime, DayBound};
use chrono::{DateTime, NaiveDate};

fn data() -> CalData {
    CalData::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/cal")).unwrap()
}

fn query(time: CalTime, tz: &str) -> Result<cal::CalResult, CalError> {
    cal::query(
        &data(),
        &CalQuery {
            time,
            timezone: tz.into(),
            day_bound: DayBound::Zichu,
        },
    )
}

#[test]
fn cross_timezone_keeps_instant_but_changes_wall_clock() {
    let instant = DateTime::parse_from_rfc3339("2026-11-03T12:00:00+08:00").unwrap();
    let shanghai = query(CalTime::Instant(instant), "Asia/Shanghai").unwrap();
    let new_york = query(CalTime::Instant(instant), "America/New_York").unwrap();
    assert_eq!(shanghai.ganzhi.day, new_york.ganzhi.day);
    assert_ne!(shanghai.solar, new_york.solar);
    assert_eq!(shanghai.lunar.day, new_york.lunar.day);
}

#[test]
fn dst_gap_is_shifted_and_flagged() {
    let gap = NaiveDate::from_ymd_opt(2026, 3, 8)
        .unwrap()
        .and_hms_opt(2, 30, 0)
        .unwrap();
    let r = query(CalTime::Local(gap), "America/New_York").unwrap();
    assert!(r.dst.gap);
    assert!(!r.dst.ambiguous);
}

#[test]
fn dst_ambiguous_is_flagged() {
    let ambiguous = NaiveDate::from_ymd_opt(2026, 11, 1)
        .unwrap()
        .and_hms_opt(1, 30, 0)
        .unwrap();
    let r = query(CalTime::Local(ambiguous), "America/New_York").unwrap();
    assert!(r.dst.ambiguous);
    assert!(!r.dst.gap);
}

#[test]
fn invalid_timezone_is_rejected() {
    let instant = DateTime::parse_from_rfc3339("2026-11-03T12:00:00+08:00").unwrap();
    let err = query(CalTime::Instant(instant), "Mars/Olympus").unwrap_err();
    assert!(matches!(err, CalError::InvalidTimezone(_)));
}

#[test]
fn out_of_range_is_explicit() {
    let instant = DateTime::parse_from_rfc3339("1899-12-31T12:00:00+08:00").unwrap();
    let err = query(CalTime::Instant(instant), "Asia/Shanghai").unwrap_err();
    assert_eq!(err, CalError::OutOfRange);

    let instant = DateTime::parse_from_rfc3339("2101-01-01T12:00:00+08:00").unwrap();
    let err = query(CalTime::Instant(instant), "Asia/Shanghai").unwrap_err();
    assert_eq!(err, CalError::OutOfRange);
}
