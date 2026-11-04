//! 从已校核的 CAL 预计算表生成逐日回归基准。
//!
//! 农历月表与节气时刻表由 `scripts/gen_cal_precise.py`（sxtwl，VSOP87/ELP2000）
//! 生成，并经香港天文台 1901–2100 年公历/农历对照表交叉校核；
//! 本工具只读取 `resources/cal/*.json`，不改写预计算表，避免用低精度算法覆盖已校核数据。

use std::path::{Path, PathBuf};

use chrono::{Datelike, FixedOffset, TimeZone};
use serde_json::json;

const BASELINE_VERSION: &str = "CAL-BASELINE-v0.2";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("baseline");
    let root = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("resources/cal"));

    match mode {
        "baseline" | "all" => gen_baselines(&root),
        other => {
            eprintln!("未知模式: {other}（baseline）");
            eprintln!("CAL 预计算表请用 scripts/gen_cal_precise.py 生成。");
            std::process::exit(2);
        }
    }
}

fn gen_baselines(cal_dir: &Path) {
    let data = cal::CalData::load(cal_dir).expect("加载历法数据");
    let baseline_dir = cal_dir.join("baseline");
    std::fs::create_dir_all(&baseline_dir).unwrap();
    let tz = FixedOffset::east_opt(8 * 3600).unwrap();

    for year in [2025, 2026] {
        let mut days = Vec::new();
        let mut date = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        while date.year() == year {
            let naive = date.and_hms_opt(12, 0, 0).unwrap();
            let instant = tz.from_local_datetime(&naive).single().unwrap();
            let result = cal::query(
                &data,
                &cal::CalQuery {
                    time: cal::CalTime::Instant(instant),
                    timezone: "Asia/Shanghai".into(),
                    day_bound: cal::DayBound::Zichu,
                },
            )
            .expect("历法查询");
            days.push(json!({
                "date": date.format("%Y-%m-%d").to_string(),
                "lunar": result.lunar,
                "jieqi": result.jieqi.current,
                "month_branch": result.jieqi_month_branch.branch,
                "ganzhi": result.ganzhi,
                "xun": result.xun,
            }));
            date += chrono::Duration::days(1);
        }
        let doc = json!({
            "version": BASELINE_VERSION,
            "status": "PENDING",
            "source": format!(
                "regression snapshot derived from {}; underlying CAL data HKO 1901-2100 cross-checked",
                data.version
            ),
            "timezone": "Asia/Shanghai",
            "day_bound": "ZICHU",
            "year": year,
            "days": days,
        });
        write_json(&baseline_dir.join(format!("{year}.json")), &doc);
        println!("baseline {year}: {}", doc["days"].as_array().unwrap().len());
    }
}

fn write_json(path: &Path, value: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    println!("wrote {}", path.display());
}
