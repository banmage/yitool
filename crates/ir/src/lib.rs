//! 最小合规模块：免责声明与禁用绝对断语扫描。阶段 1 扩展解释模板。

use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

pub const DEFAULT_DISCLAIMER: &str =
    "本工具仅供个人学习、文化研究与娱乐，不构成任何决策、医疗、法律或投资建议；请勿据此替代专业意见。";

#[derive(Debug, Error)]
pub enum ComplianceError {
    #[error("合规词表不可读: {0}")]
    Read(String),
    #[error("合规词表格式错误: {0}")]
    Parse(String),
    #[error("输出缺少免责声明")]
    MissingDisclaimer,
    #[error("输出含绝对断语: {0}")]
    ForbiddenPhrase(String),
}

#[derive(Debug, Clone, Deserialize)]
struct ComplianceFile {
    version: String,
    disclaimer: String,
    forbidden_phrases: Vec<ForbiddenEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ForbiddenEntry {
    phrase: String,
    reason: String,
}

#[derive(Debug, Clone)]
pub struct Compliance {
    pub version: String,
    pub disclaimer: String,
    phrases: Vec<ForbiddenEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub phrase: String,
    pub offset: usize,
    pub reason: String,
}

impl Compliance {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ComplianceError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .map_err(|e| ComplianceError::Read(format!("{}: {e}", path.display())))?;
        let file: ComplianceFile =
            serde_json::from_str(&text).map_err(|e| ComplianceError::Parse(e.to_string()))?;
        Ok(Self {
            version: file.version,
            disclaimer: file.disclaimer,
            phrases: file.forbidden_phrases,
        })
    }

    pub fn scan(&self, text: &str) -> Vec<Hit> {
        let mut hits = Vec::new();
        for entry in &self.phrases {
            let mut start = 0;
            while let Some(pos) = text[start..].find(&entry.phrase) {
                hits.push(Hit {
                    phrase: entry.phrase.clone(),
                    offset: start + pos,
                    reason: entry.reason.clone(),
                });
                start += pos + entry.phrase.len();
            }
        }
        hits.sort_by_key(|h| h.offset);
        hits
    }

    pub fn check_output(&self, text: &str) -> Result<(), ComplianceError> {
        if !text.contains(&self.disclaimer) {
            return Err(ComplianceError::MissingDisclaimer);
        }
        if let Some(hit) = self.scan(text).into_iter().next() {
            return Err(ComplianceError::ForbiddenPhrase(hit.phrase));
        }
        Ok(())
    }
}

/// 未冻结规则的统一标记。
pub fn pending_marker(rule_id: &str) -> String {
    format!("规则 {rule_id} 待校核/待冻结，结果不得作为正式结论。")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compliance() -> Compliance {
        Compliance {
            version: "IR-v0.1".into(),
            disclaimer: DEFAULT_DISCLAIMER.into(),
            phrases: vec![ForbiddenEntry {
                phrase: "必定".into(),
                reason: "绝对断语".into(),
            }],
        }
    }

    #[test]
    fn detects_forbidden() {
        let c = compliance();
        assert_eq!(c.scan("此事必定成功").len(), 1);
        assert!(c.check_output("此事必定成功").is_err());
        assert!(c.check_output(DEFAULT_DISCLAIMER).is_ok());
    }
}
