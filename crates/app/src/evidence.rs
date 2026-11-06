//! 依据与附件链接索引。
//!
//! 每个规则、典籍出处、数据/附件资源都有一个稳定的 `yitool://` 深层链接，
//! 报告与 UI 只能通过这些链接打开只读详情；链接不存在时必须报错，不能编造。

use std::collections::HashMap;

use domain::StepLog;
use rules::manifest::{Manifest, ResourceReport};
use rules::{RuleRegistry, SourceRecord};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EvidenceError {
    #[error("依据链接重复: {0}")]
    DuplicateHref(String),
    #[error("步骤 {step_id} 引用了不存在的规则: {ref_id}")]
    MissingRule { step_id: String, ref_id: String },
    #[error("步骤 {step_id} 引用了不存在的典籍出处: {ref_id}")]
    MissingSource { step_id: String, ref_id: String },
    #[error("步骤 {step_id} 引用了不存在的附件/数据资源: {ref_id}")]
    MissingResource { step_id: String, ref_id: String },
    #[error("无法解析的依据链接: {0}")]
    InvalidHref(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EvidenceKind {
    Rule,
    Source,
    Resource,
}

impl EvidenceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rule => "规则",
            Self::Source => "典籍出处",
            Self::Resource => "附件/数据资源",
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Rule => "rule",
            Self::Source => "source",
            Self::Resource => "resource",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceEntry {
    pub kind: EvidenceKind,
    pub id: String,
    pub title: String,
    pub href: String,
    pub anchor: String,
    /// 资源文件相对 `resources/` 的路径；规则/出处为 `None`。
    pub path: Option<String>,
    pub version: String,
    pub status: String,
    pub source_ref: Option<String>,
    pub summary: String,
    pub fields: Vec<(String, String)>,
}

impl EvidenceEntry {
    pub fn link_label(&self) -> String {
        format!("{}（{}）", self.title, self.id)
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceIndex {
    entries: Vec<EvidenceEntry>,
    by_href: HashMap<String, usize>,
}

impl EvidenceIndex {
    pub fn build(
        manifest: &Manifest,
        reports: &[ResourceReport],
        registry: &RuleRegistry,
    ) -> Result<Self, EvidenceError> {
        let checksums: HashMap<&str, &ResourceReport> =
            reports.iter().map(|r| (r.id.as_str(), r)).collect();
        let mut entries = Vec::new();

        for rule in registry.all_rules() {
            let source = registry.source(&rule.source_ref);
            entries.push(EvidenceEntry {
                kind: EvidenceKind::Rule,
                id: rule.rule_id.clone(),
                title: format!("{} · {}", rule.engine, rule.category),
                href: href(EvidenceKind::Rule, &rule.rule_id),
                anchor: anchor(EvidenceKind::Rule, &rule.rule_id),
                path: None,
                version: rule.version.clone(),
                status: rule.status.clone(),
                source_ref: Some(rule.source_ref.clone()),
                summary: source
                    .map(source_summary)
                    .unwrap_or_else(|| "来源记录缺失".into()),
                fields: vec![
                    ("引擎".into(), rule.engine.clone()),
                    ("类别".into(), rule.category.clone()),
                    ("层级".into(), rule.rule_level.clone()),
                    ("来源".into(), rule.source_ref.clone()),
                    ("条件".into(), compact_json(&rule.condition)),
                    (
                        "结果类型".into(),
                        rule.result_type.clone().unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "测试样例".into(),
                        if rule.test_refs.is_empty() {
                            "—".into()
                        } else {
                            rule.test_refs.join("、")
                        },
                    ),
                ],
            });
        }

        for source in registry.all_sources() {
            entries.push(source_entry(source));
        }

        for entry in &manifest.entries {
            let report = checksums.get(entry.id.as_str());
            entries.push(EvidenceEntry {
                kind: EvidenceKind::Resource,
                id: entry.id.clone(),
                title: entry.id.clone(),
                href: href(EvidenceKind::Resource, &entry.id),
                anchor: anchor(EvidenceKind::Resource, &entry.id),
                path: Some(entry.path.clone()),
                version: entry.version.clone(),
                status: entry.status.clone(),
                source_ref: None,
                summary: entry.source.clone(),
                fields: vec![
                    ("文件".into(), entry.path.clone()),
                    (
                        "校验和".into(),
                        report
                            .map(|r| {
                                if r.checksum_ok {
                                    "通过".into()
                                } else {
                                    "失败/不可用".into()
                                }
                            })
                            .unwrap_or_else(|| "未校验".into()),
                    ),
                    ("SHA-256".into(), entry.sha256.clone()),
                ],
            });
        }

        entries.sort_by(|a, b| {
            kind_order(a.kind)
                .cmp(&kind_order(b.kind))
                .then_with(|| a.id.cmp(&b.id))
        });

        let mut by_href = HashMap::new();
        for (index, entry) in entries.iter().enumerate() {
            if let Some(previous) = by_href.insert(entry.href.clone(), index) {
                return Err(EvidenceError::DuplicateHref(entries[previous].href.clone()));
            }
        }

        Ok(Self { entries, by_href })
    }

    pub fn all(&self) -> &[EvidenceEntry] {
        &self.entries
    }

    pub fn by_kind(&self, kind: EvidenceKind) -> impl Iterator<Item = &EvidenceEntry> {
        self.entries.iter().filter(move |entry| entry.kind == kind)
    }

    pub fn resolve_href(&self, href: &str) -> Option<&EvidenceEntry> {
        self.by_href.get(href).map(|index| &self.entries[*index])
    }

    pub fn get(&self, kind: EvidenceKind, id: &str) -> Option<&EvidenceEntry> {
        self.resolve_href(&href(kind, id))
    }

    /// 校验每个步骤的规则、出处、附件引用都能解析为链接。
    pub fn assert_step_coverage(&self, steps: &[StepLog]) -> Result<(), EvidenceError> {
        for step in steps {
            for rule_id in &step.rule_ids {
                if self.get(EvidenceKind::Rule, rule_id).is_none() {
                    return Err(EvidenceError::MissingRule {
                        step_id: step.step_id.clone(),
                        ref_id: rule_id.clone(),
                    });
                }
            }
            if self.get(EvidenceKind::Source, &step.source_ref).is_none() {
                return Err(EvidenceError::MissingSource {
                    step_id: step.step_id.clone(),
                    ref_id: step.source_ref.clone(),
                });
            }
            for data_ref in &step.data_refs {
                if self.get(EvidenceKind::Resource, data_ref).is_none() {
                    return Err(EvidenceError::MissingResource {
                        step_id: step.step_id.clone(),
                        ref_id: data_ref.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

fn source_entry(source: &SourceRecord) -> EvidenceEntry {
    EvidenceEntry {
        kind: EvidenceKind::Source,
        id: source.source_ref.clone(),
        title: format!("{} · {}", source.book, source.chapter),
        href: href(EvidenceKind::Source, &source.source_ref),
        anchor: anchor(EvidenceKind::Source, &source.source_ref),
        path: None,
        version: source.edition.clone(),
        status: source.collation_status.clone(),
        source_ref: None,
        summary: source.original_excerpt.clone(),
        fields: vec![
            ("书名".into(), source.book.clone()),
            ("版本".into(), source.edition.clone()),
            ("篇目".into(), source.chapter.clone()),
            ("原文/摘要".into(), source.original_excerpt.clone()),
            ("现代释义".into(), source.modern_note.clone()),
            ("层级".into(), source.rule_level.clone()),
            ("版权状态".into(), source.copyright_status.clone()),
        ],
    }
}

fn source_summary(source: &SourceRecord) -> String {
    format!(
        "{}（{}·{}）：{}",
        source.book, source.edition, source.chapter, source.original_excerpt
    )
}

fn compact_json(value: &serde_json::Value) -> String {
    if value.is_null() {
        "—".into()
    } else {
        serde_json::to_string(value).unwrap_or_else(|_| "—".into())
    }
}

pub fn href(kind: EvidenceKind, id: &str) -> String {
    format!("yitool://{}/{}", kind.slug(), id)
}

pub fn anchor(kind: EvidenceKind, id: &str) -> String {
    // Markdown 锚点不使用 yitool://，保证导出的 .md 在任意阅读器中可跳转。
    format!("evidence-{}-{}", kind.slug(), id)
}

fn kind_order(kind: EvidenceKind) -> u8 {
    match kind {
        EvidenceKind::Rule => 0,
        EvidenceKind::Source => 1,
        EvidenceKind::Resource => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn href_round_trip_format_is_stable() {
        assert_eq!(
            href(EvidenceKind::Source, "SRC-MEIHUA-001"),
            "yitool://source/SRC-MEIHUA-001"
        );
        assert_eq!(
            anchor(EvidenceKind::Resource, "HEX-01"),
            "evidence-resource-HEX-01"
        );
    }
}
