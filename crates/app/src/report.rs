//! 推演过程 Markdown 报告。
//!
//! 报告只消费结构化 `StepLog` 与 [`EvidenceIndex`]，不重新计算卦象；
//! 同一份字符串可直接用于窗口展示、复制到剪贴板或保存为 `.md` 文件。

use std::path::Path;

use domain::{StepLog, VersionSet};
use meihua::MeihuaResult;
use seed::SeedChain;
use thiserror::Error;

use crate::evidence::{EvidenceError, EvidenceIndex, EvidenceKind};

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("报告不含任何步骤，拒绝生成“完整”报告")]
    EmptySteps,
    #[error(transparent)]
    Evidence(#[from] EvidenceError),
    #[error("报告写入失败: {0}")]
    Io(String),
}

/// 报告元数据；由调用方提供，避免渲染层读取系统时间或隐式状态。
#[derive(Debug, Clone)]
pub struct ReportMeta {
    pub title: String,
    pub run_id: String,
    pub created_at: String,
    pub engine: String,
    pub mode: String,
    pub scheme: String,
    pub status: String,
    pub formal_allowed: bool,
    pub summary: Vec<(String, String)>,
}

/// 渲染完整推演过程 Markdown。
///
/// 渲染前会校验每一步的 `rule_ids[]`、`source_ref`、`data_refs[]` 都能在
/// 依据索引中解析；缺失时返回错误，不输出带断链的报告。
pub fn render_markdown(
    meta: &ReportMeta,
    steps: &[StepLog],
    evidence: &EvidenceIndex,
    versions: &VersionSet,
    disclaimer: &str,
) -> Result<String, ReportError> {
    // SRS FR-13：空步骤不得输出“完整”报告或可下载的空壳 Markdown。
    if steps.is_empty() {
        return Err(ReportError::EmptySteps);
    }
    evidence.assert_step_coverage(steps)?;

    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", meta.title));
    out.push_str(&format!("> **免责声明：** {}\n\n", disclaimer.trim()));
    out.push_str("| 项目 | 内容 |\n|---|---|\n");
    out.push_str(&format!("| 记录 ID | `{}` |\n", cell(&meta.run_id)));
    out.push_str(&format!("| 生成时间 | {} |\n", cell(&meta.created_at)));
    out.push_str(&format!("| 引擎 | {} |\n", cell(&meta.engine)));
    out.push_str(&format!("| 模式 | {} |\n", cell(&meta.mode)));
    out.push_str(&format!("| 输入方案 | {} |\n", cell(&meta.scheme)));
    out.push_str(&format!("| 状态 | {} |\n", cell(&meta.status)));
    out.push_str(&format!(
        "| 是否允许正式结论 | {} |\n\n",
        if meta.formal_allowed {
            "是"
        } else {
            "否（待校核）"
        }
    ));

    out.push_str("## 结果摘要\n\n");
    out.push_str("| 项目 | 内容 |\n|---|---|\n");
    for (key, value) in &meta.summary {
        out.push_str(&format!("| {} | {} |\n", cell(key), cell(value)));
    }
    out.push('\n');

    out.push_str("## 推理/演算过程\n\n");
    out.push_str(&format!(
        "本报告完整包含 {} 个步骤；每个步骤均列出规则、数据/附件、出处、层级、公式与输出。\n\n",
        steps.len()
    ));
    for (index, step) in steps.iter().enumerate() {
        render_step(&mut out, index + 1, step, evidence);
    }

    out.push_str("## 依据与附件\n\n");
    out.push_str(&format!(
        "本报告共链接 {} 条规则、出处与附件。链接使用 `yitool://` 本地协议，只读打开，不访问网络。\n\n",
        evidence.all().len()
    ));
    for kind in [
        EvidenceKind::Rule,
        EvidenceKind::Source,
        EvidenceKind::Resource,
    ] {
        render_evidence_group(&mut out, kind, evidence);
    }

    out.push_str("## 版本\n\n");
    out.push_str("| 项目 | 版本 |\n|---|---|\n");
    for (key, value) in version_rows(versions) {
        out.push_str(&format!("| {key} | `{}` |\n", cell(&value)));
    }
    out.push('\n');

    out.push_str("## 复现与合规说明\n\n");
    out.push_str(
        "1. 固定模式在相同输入、相同规则/数据/历法版本下应逐项可复现；历史记录按原版本渲染，不静默重算。\n",
    );
    out.push_str(
        "2. 报告中不含生辰原始值/派生值；问事原文默认不持久化。若记录状态为 `PENDING`，相关结果不得作为正式结论。\n",
    );
    out.push_str(
        "3. 本报告为传统易学文化学习与娱乐参考，不构成医疗、法律、投资、安全、婚姻等专业建议。\n",
    );
    if meta.scheme == "NINE_COLOR" {
        out.push_str(
            "4. 九色为系统自定义扩展象数（L3），不是传统梅花易数原典标准；不得冒充 L1 典籍结论。\n",
        );
    }
    Ok(out)
}

/// 从梅花结果生成报告；供 CLI、后续 UI 与历史复盘复用。
pub fn render_meihua_report(
    result: &MeihuaResult,
    versions: &VersionSet,
    evidence: &EvidenceIndex,
    run_id: &str,
    created_at: &str,
    mode: &str,
    seed: Option<&SeedChain>,
) -> Result<String, ReportError> {
    let mut summary = vec![
        ("本卦".into(), hex_summary(&result.original)),
        ("互卦".into(), hex_summary(&result.mutual)),
        ("变卦".into(), hex_summary(&result.changed)),
        ("动爻".into(), format!("第 {} 爻", result.moving_line)),
        (
            "体用".into(),
            format!(
                "体 {}（{}）；用 {}（{}）",
                result.ti, result.ti_element, result.yong, result.yong_element
            ),
        ),
    ];
    summary.push((
        "旺衰".into(),
        result
            .wangshuai
            .iter()
            .map(|entry| format!("{} {}（{}）", entry.trigram, entry.relation, entry.element))
            .collect::<Vec<_>>()
            .join("；"),
    ));
    summary.push((
        "忌神/弱势卦位".into(),
        format!(
            "忌神五行 {}；弱势方位 {}",
            result.jishen.element,
            if result.jishen.weak_positions.is_empty() {
                "无".into()
            } else {
                result
                    .jishen
                    .weak_positions
                    .iter()
                    .map(|p| format!("{} {}", p.trigram, p.direction))
                    .collect::<Vec<_>>()
                    .join("、")
            }
        ),
    ));
    if let Some(chain) = seed {
        summary.push((
            "抽卡结果".into(),
            format!(
                "颜色 {}；生肖 {}；传统数字 {}",
                chain
                    .draw_result
                    .color_id
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "—".into()),
                chain
                    .draw_result
                    .aux_id
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "—".into()),
                chain
                    .draw_result
                    .traditional_numbers
                    .map(|v| format!("{}, {}", v[0], v[1]))
                    .unwrap_or_else(|| "—".into())
            ),
        ));
        summary.push(("基础种子".into(), chain.base_seed_hex()));
        summary.push(("抽卡子种子".into(), chain.draw_seed_hex()));
        summary.push(("输入快照摘要".into(), chain.snapshot_digest_hex()));
        summary.push(("推演种子".into(), chain.run_seed_hex()));
    }

    let meta = ReportMeta {
        title: format!("梅花推演记录 · {}", run_id),
        run_id: run_id.into(),
        created_at: created_at.into(),
        engine: "MEIHUA".into(),
        mode: mode.into(),
        scheme: result.scheme.clone(),
        status: result.status.label().into(),
        formal_allowed: result.formal_allowed,
        summary,
    };
    render_markdown(&meta, &result.steps, evidence, versions, &result.disclaimer)
}

/// 保存为 UTF-8 Markdown 文件；父目录不存在时返回错误，由调用方决定是否创建。
pub fn write_markdown(path: impl AsRef<Path>, content: &str) -> Result<(), ReportError> {
    let path = path.as_ref();
    std::fs::write(path, content).map_err(|e| ReportError::Io(format!("{}: {e}", path.display())))
}

fn render_step(out: &mut String, number: usize, step: &StepLog, evidence: &EvidenceIndex) {
    out.push_str(&format!(
        "### {}. `{}` — {}\n\n",
        number,
        step.step_id,
        step.engine.name()
    ));
    out.push_str("| 字段 | 内容 |\n|---|---|\n");
    out.push_str(&format!("| 步骤序号 | {} |\n", step.step_index));
    out.push_str(&format!("| 状态 | {} |\n", step.status.label()));
    out.push_str(&format!("| 规则层级 | {} |\n", step.rule_level.name()));
    out.push_str(&format!(
        "| 规则 | {} |\n",
        links_for(&step.rule_ids, EvidenceKind::Rule, evidence)
    ));
    out.push_str(&format!(
        "| 数据/附件 | {} |\n",
        links_for(&step.data_refs, EvidenceKind::Resource, evidence)
    ));
    out.push_str(&format!(
        "| 出处 | {} |\n",
        link_for(&step.source_ref, EvidenceKind::Source, evidence)
    ));
    out.push_str(&format!(
        "| 公式摘要 | {} |\n\n",
        cell(&step.formula_summary)
    ));
    out.push_str("**输出**\n\n");
    out.push_str("~~~json\n");
    out.push_str(&serde_json::to_string_pretty(&step.output).unwrap_or_else(|_| "null".into()));
    out.push_str("\n~~~\n\n");
}

fn render_evidence_group(out: &mut String, kind: EvidenceKind, evidence: &EvidenceIndex) {
    out.push_str(&format!("### {}\n\n", kind.label()));
    let mut any = false;
    for entry in evidence.by_kind(kind) {
        any = true;
        out.push_str(&format!(
            "<a id=\"{}\"></a>\n#### `{}` — {}\n\n",
            entry.anchor, entry.id, entry.title
        ));
        out.push_str(&format!("- 链接：[打开]({})\n", entry.href));
        out.push_str(&format!(
            "- 版本：{}；状态：{}\n",
            entry.version, entry.status
        ));
        if let Some(path) = &entry.path {
            out.push_str(&format!("- 附件路径：`resources/{path}`\n"));
        }
        if let Some(source_ref) = &entry.source_ref {
            out.push_str(&format!(
                "- 出处：{}\n",
                link_for(source_ref, EvidenceKind::Source, evidence)
            ));
        }
        out.push_str(&format!("- 摘要：{}\n", entry.summary));
        for (key, value) in &entry.fields {
            out.push_str(&format!("- {}：{}\n", key, cell(value)));
        }
        out.push('\n');
    }
    if !any {
        out.push_str("（无）\n\n");
    }
}

fn links_for(ids: &[String], kind: EvidenceKind, evidence: &EvidenceIndex) -> String {
    if ids.is_empty() {
        return "—".into();
    }
    ids.iter()
        .map(|id| link_for(id, kind, evidence))
        .collect::<Vec<_>>()
        .join("、")
}

fn link_for(id: &str, kind: EvidenceKind, evidence: &EvidenceIndex) -> String {
    match evidence.get(kind, id) {
        Some(entry) => format!("[`{}`](#{})", entry.id, entry.anchor),
        None => format!("`{}`（断链）", id),
    }
}

fn hex_summary(hex: &meihua::HexOut) -> String {
    format!(
        "{}（上{}下{}，{}）",
        hex.name, hex.upper, hex.lower, hex.binary_bottom_up
    )
}

fn version_rows(versions: &VersionSet) -> Vec<(&'static str, String)> {
    vec![
        ("SRS", versions.srs.clone()),
        ("ARS", versions.ars.clone()),
        ("IR", versions.ir.clone()),
        ("CAL", versions.cal.clone()),
        ("规则集", versions.rule_set.clone()),
        ("数据", versions.data.clone()),
        ("历法数据", versions.cal_data.clone()),
        ("tzdata", versions.tzdata.clone()),
        ("种子算法", versions.seed_algo.clone()),
        ("随机数", versions.rng.clone()),
        ("应用", versions.app.clone()),
        ("数据库 schema", versions.db_schema.clone()),
    ]
}

fn cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "<br>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_cell_escapes_pipes_and_newlines() {
        assert_eq!(cell("a|b\nc"), "a\\|b<br>c");
    }
}
