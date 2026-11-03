use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("非法八卦序号: {0}（应为 1..=8 或 0..=7 的索引）")]
    InvalidTrigram(u8),
    #[error("非法爻序列：必须为 6 爻")]
    InvalidHexagramLines,
    #[error("步骤日志至少需要一条 rule_id")]
    EmptyRuleIds,
    #[error("步骤日志缺少 source_ref 或 rule_level")]
    MissingTraceability,
    #[error("非法天干序号: {0}")]
    InvalidStem(u8),
    #[error("非法地支序号: {0}")]
    InvalidBranch(u8),
    #[error("JSON 序列化失败: {0}")]
    Json(String),
}

pub type Result<T> = std::result::Result<T, DomainError>;
