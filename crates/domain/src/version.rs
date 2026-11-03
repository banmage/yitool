use serde::{Deserialize, Serialize};

/// 全系统版本集合，随每次推演记录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionSet {
    pub srs: String,
    pub ars: String,
    pub ir: String,
    pub cal: String,
    pub rule_set: String,
    pub data: String,
    pub cal_data: String,
    pub tzdata: String,
    pub seed_algo: String,
    pub rng: String,
    pub app: String,
    pub db_schema: String,
}

impl Default for VersionSet {
    fn default() -> Self {
        Self {
            srs: "SRS-v0.4".into(),
            ars: "ARS-v0.2".into(),
            ir: "IR-v0.1".into(),
            cal: "CAL-v0.1".into(),
            rule_set: "RULE-v0.2".into(),
            data: "DATA-v0.2".into(),
            cal_data: "CAL-DATA-v0.1".into(),
            tzdata: chrono_tz_version(),
            seed_algo: "SEED-v2".into(),
            rng: "RNG-v1".into(),
            app: "APP-v0.1".into(),
            db_schema: "DB-v0.1".into(),
        }
    }
}

fn chrono_tz_version() -> String {
    // domain 不依赖 chrono-tz；由调用方在需要时覆盖。
    "2026a".into()
}

impl VersionSet {
    /// 仅覆盖历法数据版本（资源加载后调用）。
    pub fn with_cal_data(mut self, version: impl Into<String>) -> Self {
        self.cal_data = version.into();
        self
    }
}
