use chrono::{
    DateTime, Datelike, Duration, FixedOffset, LocalResult, NaiveDate, NaiveDateTime, Offset,
    TimeZone, Timelike, Utc,
};
use chrono_tz::Tz;
use domain::{GanZhi, Status, VersionSet, WuXing};
use serde::{Deserialize, Serialize};

use crate::data::CalData;
use crate::error::CalError;
use crate::jdn::jdn_from_ymd;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DayBound {
    /// 子初换日：23:00 起算次日。
    Zichu,
    /// 子正换日：00:00 起算次日。
    Zizheng,
}

impl DayBound {
    pub fn name(self) -> &'static str {
        match self {
            Self::Zichu => "ZICHU",
            Self::Zizheng => "ZIZHENG",
        }
    }
}

#[derive(Debug, Clone)]
pub enum CalTime {
    /// 带偏移的绝对时刻。
    Instant(DateTime<FixedOffset>),
    /// 无偏移的本地墙钟时刻；按 `timezone` 解析并检测 DST 缺失/重复。
    Local(NaiveDateTime),
}

#[derive(Debug, Clone)]
pub struct CalQuery {
    pub time: CalTime,
    pub timezone: String,
    pub day_bound: DayBound,
}

#[derive(Debug, Clone, Serialize)]
pub struct LunarOut {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub leap: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LunarMonthIndexOut {
    pub lunar_index: u32,
    pub leap: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct JieqiOut {
    pub current: String,
    pub next: String,
    pub current_term: u8,
    pub next_term: u8,
    pub next_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JieqiMonthBranchOut {
    pub branch_id: u8,
    pub branch: String,
    pub element: String,
    pub jieqi: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GanZhiOut {
    pub year: String,
    pub month: String,
    pub day: String,
    pub hour: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct XunOut {
    pub xun_head: String,
    pub kong_branches: [String; 2],
}

#[derive(Debug, Clone, Serialize)]
pub struct DstOut {
    pub ambiguous: bool,
    pub gap: bool,
    /// 解析后实际采用的本地偏移（如 `-04:00`）。
    pub selected_offset: String,
    pub policy: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalResult {
    pub solar: String,
    pub timezone: String,
    pub day_bound: DayBound,
    pub effective_date: String,
    pub lunar: LunarOut,
    pub lunar_month_index: LunarMonthIndexOut,
    pub jieqi: JieqiOut,
    pub jieqi_month_branch: JieqiMonthBranchOut,
    pub ganzhi: GanZhiOut,
    pub xun: XunOut,
    pub dst: DstOut,
    pub versions: VersionSet,
    pub status: Status,
    pub disclaimer: String,
}

const MIN_DATE: (i32, u32, u32) = (1900, 1, 31);
const MAX_DATE: (i32, u32, u32) = (2100, 12, 31);

pub fn query(data: &CalData, query: &CalQuery) -> Result<CalResult, CalError> {
    let tz: Tz = query
        .timezone
        .parse()
        .map_err(|_| CalError::InvalidTimezone(query.timezone.clone()))?;

    let (local_dt, utc, ambiguous, gap) = resolve_local(query, tz)?;
    let local_date = local_dt.date_naive();
    let (y, m, d) = (local_date.year(), local_date.month(), local_date.day());
    if (y, m, d) < MIN_DATE || (y, m, d) > MAX_DATE {
        return Err(CalError::OutOfRange);
    }

    let day_offset = match query.day_bound {
        DayBound::Zichu if local_dt.hour() >= 23 => 1,
        _ => 0,
    };
    let effective_jdn = jdn_from_ymd(y, m, d) + day_offset;
    let effective_date = if day_offset == 0 {
        local_date
    } else {
        local_date + Duration::days(1)
    };

    // 农历固定以中国标准时间 +08:00 的民用日为锚（与预计算表一致）。
    let china = FixedOffset::east_opt(8 * 3600).expect("+08:00 合法");
    let china_date = utc.with_timezone(&china).date_naive();
    let month = data.lunar_for_jdn(jdn_from_ymd(
        china_date.year(),
        china_date.month(),
        china_date.day(),
    ));
    let month = month
        .ok_or_else(|| CalError::DataUnavailable(format!("农历月表缺少 {china_date} 的数据")))?;
    let lunar_day = (jdn_from_ymd(china_date.year(), china_date.month(), china_date.day())
        - month.start_jdn
        + 1) as u32;
    let lunar = LunarOut {
        year: month.year,
        month: month.month,
        day: lunar_day,
        leap: month.leap,
    };

    let current = data
        .last_term_at(utc)
        .ok_or_else(|| CalError::DataUnavailable("节气表缺少当前节气".into()))?;
    let next = data
        .next_term_after(utc)
        .ok_or_else(|| CalError::DataUnavailable("节气表缺少下一节气".into()))?;

    let jie = data
        .last_jie_at(utc)
        .ok_or_else(|| CalError::DataUnavailable("节气表缺少月建节".into()))?;

    let lichun = data
        .lichun(y)
        .ok_or_else(|| CalError::DataUnavailable(format!("缺少 {y} 年立春")))?;
    let solar_year = if utc < lichun.utc { y - 1 } else { y };

    let year_gz = GanZhi::from_cycle_index((solar_year - 4).rem_euclid(60) as u8);

    let month_branch = jie.month_branch();
    let year_stem = year_gz.stem as i32;
    let yin_stem = (year_stem * 2 + 2).rem_euclid(10) as u8;
    let month_offset = ((month_branch as i32 + 12 - 2) % 12) as u8;
    let month_gz = GanZhi::new((yin_stem + month_offset) % 10, month_branch)?;

    let day_index = (effective_jdn + 49).rem_euclid(60) as u8;
    let day_gz = GanZhi::from_cycle_index(day_index);

    let hour_branch = (local_dt.hour() as u8).div_ceil(2) % 12;
    let hour_stem = (day_gz.stem * 2 + hour_branch) % 10;
    let hour_gz = GanZhi::new(hour_stem, hour_branch)?;

    let (xun_head, kong) = day_gz.xun();
    let xun = XunOut {
        xun_head: xun_head.name(),
        kong_branches: [
            domain::BRANCHES[kong[0] as usize].to_string(),
            domain::BRANCHES[kong[1] as usize].to_string(),
        ],
    };

    let mut versions = VersionSet::default().with_cal_data(data.version.clone());
    versions.tzdata = chrono_tz::IANA_TZDB_VERSION.to_string();

    Ok(CalResult {
        solar: local_dt.to_rfc3339(),
        timezone: query.timezone.clone(),
        day_bound: query.day_bound,
        effective_date: effective_date.format("%Y-%m-%d").to_string(),
        lunar: lunar.clone(),
        lunar_month_index: LunarMonthIndexOut {
            lunar_index: month.month,
            leap: month.leap,
        },
        jieqi: JieqiOut {
            current: current.name.clone(),
            next: next.name.clone(),
            current_term: current.term,
            next_term: next.term,
            next_at: next.utc.to_rfc3339(),
        },
        jieqi_month_branch: JieqiMonthBranchOut {
            branch_id: month_branch,
            branch: domain::BRANCHES[month_branch as usize].to_string(),
            element: WuXing::from_branch(month_branch).name().to_string(),
            jieqi: jie.name.clone(),
        },
        ganzhi: GanZhiOut {
            year: year_gz.name(),
            month: month_gz.name(),
            day: day_gz.name(),
            hour: hour_gz.name(),
        },
        xun,
        dst: DstOut {
            ambiguous,
            gap,
            selected_offset: local_dt.offset().fix().to_string(),
            policy: "AMBIGUOUS_EARLIER_OFFSET; GAP_SHIFT_FORWARD_1H".into(),
        },
        versions,
        status: if data.status == "PENDING" {
            Status::Pending
        } else {
            Status::Verified
        },
        disclaimer: if data.status == "PENDING" {
            "本工具仅供个人学习与娱乐，不构成任何决策建议；历法数据待权威历书校核。".into()
        } else {
            "本工具仅供个人学习与娱乐，不构成任何决策建议；历法日期已按香港天文台 1901–2100 对照表校核，节气时刻为天文计算值。".into()
        },
    })
}

fn resolve_local(
    query: &CalQuery,
    tz: Tz,
) -> Result<(DateTime<Tz>, DateTime<Utc>, bool, bool), CalError> {
    match &query.time {
        CalTime::Instant(instant) => {
            let local = instant.with_timezone(&tz);
            let naive = local.naive_local();
            match tz.from_local_datetime(&naive) {
                LocalResult::Single(_) => Ok((local, instant.with_timezone(&Utc), false, false)),
                LocalResult::Ambiguous(_, _) => {
                    Ok((local, instant.with_timezone(&Utc), true, false))
                }
                LocalResult::None => {
                    // 绝对时刻必然能映射到某个本地墙钟时刻；理论上不可达。
                    Err(CalError::DstGap)
                }
            }
        }
        CalTime::Local(naive) => match tz.from_local_datetime(naive) {
            LocalResult::Single(dt) => Ok((dt, dt.with_timezone(&Utc), false, false)),
            LocalResult::Ambiguous(earlier, _) => {
                Ok((earlier, earlier.with_timezone(&Utc), true, false))
            }
            LocalResult::None => {
                let shifted = *naive + Duration::hours(1);
                let dt = tz
                    .from_local_datetime(&shifted)
                    .earliest()
                    .ok_or(CalError::DstGap)?;
                Ok((dt, dt.with_timezone(&Utc), false, true))
            }
        },
    }
}

/// 由公历日期直接计算日柱（用于基准生成与交叉校验）。
pub fn day_ganzhi_for_date(y: i32, m: u32, d: u32) -> GanZhi {
    GanZhi::from_cycle_index(((jdn_from_ymd(y, m, d) + 49).rem_euclid(60)) as u8)
}

pub fn parse_date(s: &str) -> Result<NaiveDate, CalError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| CalError::InvalidDate(format!("{s}: {e}")))
}
