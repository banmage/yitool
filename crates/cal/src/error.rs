use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CalError {
    #[error("CAL_OUT_OF_RANGE: 日期超出支持范围 1900-01-31..=2100-12-31")]
    OutOfRange,
    #[error("CAL_INVALID_DATE: {0}")]
    InvalidDate(String),
    #[error("CAL_INVALID_TIMEZONE: {0}")]
    InvalidTimezone(String),
    #[error("CAL_DATA_UNAVAILABLE: {0}")]
    DataUnavailable(String),
    #[error("CAL_DATA_INVALID: {0}")]
    DataInvalid(String),
    #[error("CAL_DST_GAP: 本地时刻不存在（夏令时跳变），已按冻结策略向后平移")]
    DstGap,
    #[error("CAL_DST_AMBIGUOUS: 本地时刻重复（夏令时回拨）")]
    DstAmbiguous,
    #[error("CAL_DOMAIN: 干支计算失败: {0}")]
    Domain(#[from] domain::DomainError),
}
