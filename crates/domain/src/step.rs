use serde::{Deserialize, Serialize};

use crate::error::{DomainError, Result};

/// 规则层级。L3 不得冒充 L1；PROVISIONAL 仅用于未校核来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RuleLevel {
    L1,
    L2,
    L3,
    Provisional,
}

impl RuleLevel {
    pub fn name(self) -> &'static str {
        match self {
            Self::L1 => "L1",
            Self::L2 => "L2",
            Self::L3 => "L3",
            Self::Provisional => "PROVISIONAL",
        }
    }

    pub fn is_formal(self) -> bool {
        !matches!(self, Self::Provisional)
    }
}

/// 步骤/资源状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Ok,
    Pending,
    Error,
    Verified,
    Frozen,
    Deprecated,
    Readonly,
}

impl Status {
    pub fn is_formal(self) -> bool {
        matches!(self, Self::Ok | Self::Verified | Self::Frozen)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Pending => "PENDING",
            Self::Error => "ERROR",
            Self::Verified => "VERIFIED",
            Self::Frozen => "FROZEN",
            Self::Deprecated => "DEPRECATED",
            Self::Readonly => "READONLY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Engine {
    Meihua,
    Jiaolin,
    Liuyao,
    Zeji,
    Cal,
    Seed,
    Ir,
    Rng,
    Log,
    Store,
    App,
}

impl Engine {
    pub fn name(self) -> &'static str {
        match self {
            Self::Meihua => "MEIHUA",
            Self::Jiaolin => "JIAOLIN",
            Self::Liuyao => "LIUYAO",
            Self::Zeji => "ZEJI",
            Self::Cal => "CAL",
            Self::Seed => "SEED",
            Self::Ir => "IR",
            Self::Rng => "RNG",
            Self::Log => "LOG",
            Self::Store => "STORE",
            Self::App => "APP",
        }
    }
}

/// 一步推演日志。所有正式引擎步骤必须完整填写。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepLog {
    pub step_index: usize,
    pub step_id: String,
    pub engine: Engine,
    pub rule_ids: Vec<String>,
    pub data_refs: Vec<String>,
    pub source_ref: String,
    pub rule_level: RuleLevel,
    pub formula_summary: String,
    pub output: serde_json::Value,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct StepLogBuilder {
    step_index: usize,
    step_id: String,
    engine: Engine,
    rule_ids: Vec<String>,
    data_refs: Vec<String>,
    source_ref: Option<String>,
    rule_level: Option<RuleLevel>,
    formula_summary: String,
    output: serde_json::Value,
    status: Status,
}

impl StepLogBuilder {
    pub fn new(step_index: usize, step_id: impl Into<String>, engine: Engine) -> Self {
        Self {
            step_index,
            step_id: step_id.into(),
            engine,
            rule_ids: Vec::new(),
            data_refs: Vec::new(),
            source_ref: None,
            rule_level: None,
            formula_summary: String::new(),
            output: serde_json::Value::Null,
            status: Status::Ok,
        }
    }

    pub fn rule_id(mut self, id: impl Into<String>) -> Self {
        self.rule_ids.push(id.into());
        self
    }

    pub fn data_ref(mut self, id: impl Into<String>) -> Self {
        self.data_refs.push(id.into());
        self
    }

    pub fn source(mut self, source_ref: impl Into<String>) -> Self {
        self.source_ref = Some(source_ref.into());
        self
    }

    pub fn level(mut self, level: RuleLevel) -> Self {
        self.rule_level = Some(level);
        self
    }

    pub fn formula(mut self, summary: impl Into<String>) -> Self {
        self.formula_summary = summary.into();
        self
    }

    pub fn output(mut self, output: serde_json::Value) -> Self {
        self.output = output;
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn build(self) -> Result<StepLog> {
        if self.rule_ids.is_empty() {
            return Err(DomainError::EmptyRuleIds);
        }
        let source_ref = self.source_ref.ok_or(DomainError::MissingTraceability)?;
        let rule_level = self.rule_level.ok_or(DomainError::MissingTraceability)?;
        Ok(StepLog {
            step_index: self.step_index,
            step_id: self.step_id,
            engine: self.engine,
            rule_ids: self.rule_ids,
            data_refs: self.data_refs,
            source_ref,
            rule_level,
            formula_summary: self.formula_summary,
            output: self.output,
            status: self.status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_requires_traceability() {
        let err = StepLogBuilder::new(0, "X", Engine::Cal).build();
        assert_eq!(err.unwrap_err(), DomainError::EmptyRuleIds);

        let err = StepLogBuilder::new(0, "X", Engine::Cal)
            .rule_id("CAL-X")
            .build();
        assert_eq!(err.unwrap_err(), DomainError::MissingTraceability);

        let step = StepLogBuilder::new(0, "X", Engine::Cal)
            .rule_id("CAL-X")
            .source("SRC-CAL-001")
            .level(RuleLevel::L2)
            .build()
            .unwrap();
        assert_eq!(step.step_id, "X");
    }
}
