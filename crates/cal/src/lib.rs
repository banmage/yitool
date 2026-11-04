//! CAL 历法服务：预计算农历月表 + 节气时刻表，纯 Rust 查询。
//!
//! 数据来源与校核状态随结果输出；`CAL-DATA-v0.2` 已按 HKO 1901–2100 日期级校核，
//! 但 CAL 规则集本身冻结前仍须保留校核提示。

pub mod data;
pub mod engine;
pub mod error;
pub mod jdn;

pub use data::{CalData, JieqiEvent, LunarMonth};
pub use engine::{query, CalQuery, CalResult, CalTime, DayBound};
pub use error::CalError;
pub use jdn::{jdn_from_ymd, ymd_from_jdn};
