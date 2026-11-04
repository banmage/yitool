//! HKO 1901–2100 交叉校核的数据回归快照。
//!
//! 这些断言固定 `CAL-DATA-v0.2` 的校核结论：HKO 对照表中三处农历月首
//! 边界修正、六处节气午夜边界容差。数据版本升级时必须同步复核本文件。

use std::path::PathBuf;

use cal::{jdn_from_ymd, CalData};
use chrono::{DateTime, Timelike};

fn data() -> CalData {
    CalData::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/cal")).unwrap()
}

#[test]
fn data_version_and_status_are_pinned() {
    let data = data();
    assert_eq!(data.version, "CAL-DATA-v0.2");
    assert_eq!(data.status, "VERIFIED");
    assert_eq!(data.months.len(), 2486);
    assert_eq!(data.jieqi.len(), 4872);
}

#[test]
fn hko_lunar_month_boundary_corrections() {
    let data = data();
    for (date, expected) in [
        ((1914, 11, 17), (1914, 10, 1, false)),
        ((1916, 2, 3), (1916, 1, 1, false)),
        ((1920, 11, 10), (1920, 10, 1, false)),
    ] {
        let month = data
            .lunar_for_jdn(jdn_from_ymd(date.0, date.1, date.2))
            .unwrap_or_else(|| panic!("{date:?} 无农历月"));
        assert_eq!(
            (month.year, month.month, month.start_jdn, month.leap),
            (
                expected.0,
                expected.1,
                jdn_from_ymd(date.0, date.1, date.2),
                expected.3
            ),
            "{date:?} 应为农历 {expected:?} 初一"
        );
    }
}

#[test]
fn hko_midnight_boundary_terms_within_15_minutes() {
    let data = data();
    let cases = [
        ("1912-11-22", "小雪"),
        ("1913-09-23", "秋分"),
        ("1917-12-08", "大雪"),
        ("1927-09-09", "白露"),
        ("1928-06-22", "夏至"),
        ("1979-01-20", "大寒"),
    ];
    for (date, name) in cases {
        let event = data
            .jieqi
            .iter()
            .find(|event| event.local.starts_with(date) && event.name == name)
            .unwrap_or_else(|| panic!("缺少 {date} {name}"));
        let local = DateTime::parse_from_rfc3339(&event.local).unwrap();
        let midnight = (local.hour() * 3600 + local.minute() * 60 + local.second()) as i64;
        let distance = midnight.min(86_400 - midnight);
        assert!(
            distance <= 900,
            "{date} {name} 距午夜 {distance} 秒，超出 15 分钟边界容差"
        );
    }
}
