use std::collections::HashMap;
use std::path::Path;

use domain::RuleLevel;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("规则资源不可读: {0}")]
    Read(String),
    #[error("规则资源格式错误: {0}")]
    Parse(String),
    #[error("重复的 rule_id: {0}")]
    DuplicateRule(String),
    #[error("规则缺少测试样例: {0}")]
    MissingTests(String),
    #[error("非法规则层级: {0}")]
    InvalidLevel(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleRecord {
    pub rule_id: String,
    pub engine: String,
    pub category: String,
    pub rule_level: String,
    pub condition: serde_json::Value,
    pub result_type: Option<String>,
    pub weight: Option<f64>,
    pub priority: Option<i32>,
    pub source_ref: String,
    pub version: String,
    pub status: String,
    pub test_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceRecord {
    pub source_ref: String,
    pub book: String,
    pub edition: String,
    pub chapter: String,
    pub original_excerpt: String,
    pub modern_note: String,
    pub rule_level: String,
    pub collation_status: String,
    pub copyright_status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RuleFile {
    version: String,
    rules: Vec<RuleRecord>,
    sources: Vec<SourceRecord>,
}

#[derive(Debug, Clone)]
pub struct RuleRegistry {
    pub version: String,
    rules: HashMap<String, RuleRecord>,
    sources: HashMap<String, SourceRecord>,
}

impl RuleRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, RegistryError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .map_err(|e| RegistryError::Read(format!("{}: {e}", path.display())))?;
        let file: RuleFile =
            serde_json::from_str(&text).map_err(|e| RegistryError::Parse(e.to_string()))?;

        let mut rules = HashMap::new();
        for rule in file.rules {
            rule_level(&rule.rule_level)?;
            if rule.test_refs.is_empty() {
                return Err(RegistryError::MissingTests(rule.rule_id));
            }
            if rules.insert(rule.rule_id.clone(), rule.clone()).is_some() {
                return Err(RegistryError::DuplicateRule(rule.rule_id));
            }
        }
        let mut sources = HashMap::new();
        for source in file.sources {
            sources.insert(source.source_ref.clone(), source);
        }
        Ok(Self {
            version: file.version,
            rules,
            sources,
        })
    }

    pub fn rule(&self, rule_id: &str) -> Option<&RuleRecord> {
        self.rules.get(rule_id)
    }

    pub fn source(&self, source_ref: &str) -> Option<&SourceRecord> {
        self.sources.get(source_ref)
    }

    pub fn rules_for_engine(&self, engine: &str) -> Vec<&RuleRecord> {
        self.rules.values().filter(|r| r.engine == engine).collect()
    }

    pub fn assert_traceable(&self, rule_id: &str, source_ref: &str) -> bool {
        self.rule(rule_id).is_some() && self.source(source_ref).is_some()
    }
}

pub fn rule_level(raw: &str) -> Result<RuleLevel, RegistryError> {
    match raw {
        "L1" => Ok(RuleLevel::L1),
        "L2" => Ok(RuleLevel::L2),
        "L3" => Ok(RuleLevel::L3),
        "PROVISIONAL" => Ok(RuleLevel::Provisional),
        other => Err(RegistryError::InvalidLevel(other.to_string())),
    }
}
