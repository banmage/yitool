use std::path::Path;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

use crate::error::CalError;
use crate::jdn::jdn_from_ymd;

#[derive(Debug, Clone)]
pub struct LunarMonth {
    pub start_jdn: i64,
    pub year: i32,
    pub month: u32,
    pub leap: bool,
    pub days: u32,
}

impl LunarMonth {
    pub fn end_jdn_exclusive(&self) -> i64 {
        self.start_jdn + self.days as i64
    }
}

#[derive(Debug, Clone)]
pub struct JieqiEvent {
    /// 0..24，0 = 小寒，2 = 立春，23 = 冬至。
    pub term: u8,
    pub name: String,
    pub utc: DateTime<Utc>,
    pub local: String,
    pub jd_ut: f64,
}

impl JieqiEvent {
    /// 是否为“节”（奇数索引为气，偶数索引为节）。
    pub fn is_jie(&self) -> bool {
        self.term % 2 == 0
    }

    /// 节气对应的月建地支（0=子 .. 11=亥）。
    pub fn month_branch(&self) -> u8 {
        (self.term / 2 + 1) % 12
    }
}

#[derive(Debug, Clone)]
pub struct CalData {
    pub version: String,
    pub status: String,
    pub reference_tz: String,
    pub months: Vec<LunarMonth>,
    pub jieqi: Vec<JieqiEvent>,
}

#[derive(Deserialize)]
struct LunarFile {
    version: String,
    status: String,
    reference_tz: String,
    months: Vec<LunarEntry>,
}

#[derive(Deserialize)]
struct LunarEntry {
    start: String,
    year: i32,
    month: u32,
    leap: bool,
    days: u32,
}

#[derive(Deserialize)]
struct JieqiFile {
    version: String,
    status: String,
    events: Vec<JieqiEntry>,
}

#[derive(Deserialize)]
struct JieqiEntry {
    term: u8,
    name: String,
    utc: String,
    local: String,
    jd_ut: f64,
}

impl CalData {
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, CalError> {
        let dir = dir.as_ref();
        let lunar_path = dir.join("lunar_months.json");
        let jieqi_path = dir.join("jieqi.json");

        let lunar_text = std::fs::read_to_string(&lunar_path)
            .map_err(|e| CalError::DataUnavailable(format!("{}: {e}", lunar_path.display())))?;
        let jieqi_text = std::fs::read_to_string(&jieqi_path)
            .map_err(|e| CalError::DataUnavailable(format!("{}: {e}", jieqi_path.display())))?;

        let lunar: LunarFile = serde_json::from_str(&lunar_text)
            .map_err(|e| CalError::DataInvalid(format!("lunar_months.json: {e}")))?;
        let jieqi: JieqiFile = serde_json::from_str(&jieqi_text)
            .map_err(|e| CalError::DataInvalid(format!("jieqi.json: {e}")))?;

        if lunar.version != jieqi.version {
            return Err(CalError::DataInvalid(format!(
                "lunar_months.json 与 jieqi.json 版本不一致: {} vs {}",
                lunar.version, jieqi.version
            )));
        }
        if jieqi.status.trim().is_empty() {
            return Err(CalError::DataInvalid("jieqi.json 缺少 status".into()));
        }

        let mut months = Vec::with_capacity(lunar.months.len());
        for entry in lunar.months {
            let date = NaiveDate::parse_from_str(&entry.start, "%Y-%m-%d")
                .map_err(|e| CalError::DataInvalid(format!("月份起点 {}: {e}", entry.start)))?;
            use chrono::Datelike;
            let start_jdn = jdn_from_ymd(date.year(), date.month(), date.day());
            if entry.days < 29 || entry.days > 30 {
                return Err(CalError::DataInvalid(format!(
                    "农历月 {} 天数异常: {}",
                    entry.start, entry.days
                )));
            }
            months.push(LunarMonth {
                start_jdn,
                year: entry.year,
                month: entry.month,
                leap: entry.leap,
                days: entry.days,
            });
        }
        months.sort_by_key(|m| m.start_jdn);
        for pair in months.windows(2) {
            if pair[0].end_jdn_exclusive() != pair[1].start_jdn {
                return Err(CalError::DataInvalid(format!(
                    "农历月表不连续: {} 结束 {}，下一月开始 {}",
                    pair[0].start_jdn,
                    pair[0].end_jdn_exclusive(),
                    pair[1].start_jdn
                )));
            }
        }

        let mut events = Vec::with_capacity(jieqi.events.len());
        for entry in jieqi.events {
            if entry.term >= 24 {
                return Err(CalError::DataInvalid(format!(
                    "非法节气索引: {}",
                    entry.term
                )));
            }
            let utc = DateTime::parse_from_rfc3339(&entry.utc)
                .map_err(|e| CalError::DataInvalid(format!("节气时刻 {}: {e}", entry.utc)))?
                .with_timezone(&Utc);
            events.push(JieqiEvent {
                term: entry.term,
                name: entry.name,
                utc,
                local: entry.local,
                jd_ut: entry.jd_ut,
            });
        }
        events.sort_by_key(|e| e.utc);

        Ok(Self {
            version: lunar.version,
            status: lunar.status,
            reference_tz: lunar.reference_tz,
            months,
            jieqi: events,
        })
    }

    pub fn lunar_for_jdn(&self, jdn: i64) -> Option<&LunarMonth> {
        let idx = self
            .months
            .partition_point(|m| m.start_jdn <= jdn)
            .checked_sub(1)?;
        let month = &self.months[idx];
        if jdn < month.end_jdn_exclusive() {
            Some(month)
        } else {
            None
        }
    }

    /// 全部节气（24 个/年）中，不晚于 `instant` 的最后一个。
    pub fn last_term_at(&self, instant: DateTime<Utc>) -> Option<&JieqiEvent> {
        let idx = self
            .jieqi
            .partition_point(|e| e.utc <= instant)
            .checked_sub(1)?;
        self.jieqi.get(idx)
    }

    pub fn next_term_after(&self, instant: DateTime<Utc>) -> Option<&JieqiEvent> {
        let idx = self.jieqi.partition_point(|e| e.utc <= instant);
        self.jieqi.get(idx)
    }

    /// 不晚于 `instant` 的最后一个“节”（用于月柱）。
    pub fn last_jie_at(&self, instant: DateTime<Utc>) -> Option<&JieqiEvent> {
        let mut idx = self.jieqi.partition_point(|e| e.utc <= instant);
        while idx > 0 {
            idx -= 1;
            if self.jieqi[idx].is_jie() {
                return Some(&self.jieqi[idx]);
            }
        }
        None
    }

    pub fn lichun(&self, year: i32) -> Option<&JieqiEvent> {
        self.jieqi
            .iter()
            .find(|e| e.term == 2 && e.utc.format("%Y").to_string() == year.to_string())
    }

    pub fn next_jie_after(&self, instant: DateTime<Utc>) -> Option<&JieqiEvent> {
        let mut idx = self.jieqi.partition_point(|e| e.utc <= instant);
        while idx < self.jieqi.len() {
            if self.jieqi[idx].is_jie() {
                return Some(&self.jieqi[idx]);
            }
            idx += 1;
        }
        None
    }
}
