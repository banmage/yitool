use std::path::PathBuf;

use cal::{CalData, CalQuery, CalTime, DayBound};
use chrono::{FixedOffset, TimeZone};
use serde::Deserialize;

#[derive(Deserialize)]
struct Baseline {
    year: i32,
    days: Vec<Day>,
}

#[derive(Deserialize)]
struct Day {
    date: String,
    lunar: Lunar,
    month_branch: String,
    ganzhi: Ganzhi,
    xun: Xun,
}

#[derive(Deserialize)]
struct Lunar {
    year: i32,
    month: u32,
    day: u32,
    leap: bool,
}

#[derive(Deserialize)]
struct Ganzhi {
    year: String,
    month: String,
    day: String,
    hour: String,
}

#[derive(Deserialize)]
struct Xun {
    xun_head: String,
    kong_branches: Vec<String>,
}

fn resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources")
}

#[test]
fn full_year_baselines_match() {
    let root = resources();
    let data = CalData::load(root.join("cal")).unwrap();
    let tz = FixedOffset::east_opt(8 * 3600).unwrap();

    for year in [2025, 2026] {
        let text = std::fs::read_to_string(root.join(format!("cal/baseline/{year}.json"))).unwrap();
        let baseline: Baseline = serde_json::from_str(&text).unwrap();
        assert_eq!(baseline.year, year);
        assert_eq!(baseline.days.len(), 365, "{year} 应为平年");

        for day in &baseline.days {
            let naive = chrono::NaiveDate::parse_from_str(&day.date, "%Y-%m-%d")
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap();
            let instant = tz.from_local_datetime(&naive).single().unwrap();
            let r = cal::query(
                &data,
                &CalQuery {
                    time: CalTime::Instant(instant),
                    timezone: "Asia/Shanghai".into(),
                    day_bound: DayBound::Zichu,
                },
            )
            .unwrap();
            assert_eq!(
                (r.lunar.year, r.lunar.month, r.lunar.day, r.lunar.leap),
                (
                    day.lunar.year,
                    day.lunar.month,
                    day.lunar.day,
                    day.lunar.leap
                ),
                "{} 农历不一致",
                day.date
            );
            assert_eq!(
                r.jieqi_month_branch.branch, day.month_branch,
                "{} 月建",
                day.date
            );
            assert_eq!(
                (
                    &r.ganzhi.year,
                    &r.ganzhi.month,
                    &r.ganzhi.day,
                    &r.ganzhi.hour
                ),
                (
                    &day.ganzhi.year,
                    &day.ganzhi.month,
                    &day.ganzhi.day,
                    &day.ganzhi.hour
                ),
                "{} 四柱",
                day.date
            );
            assert_eq!(r.xun.xun_head, day.xun.xun_head, "{} 旬首", day.date);
            assert_eq!(
                r.xun.kong_branches.to_vec(),
                day.xun.kong_branches,
                "{} 旬空",
                day.date
            );
        }
    }
}
