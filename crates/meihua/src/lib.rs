//! 梅花易数：九色与传统数字两套独立起卦、互卦、变卦、体用、旺衰。

use domain::{
    Engine, GanZhi, Hexagram, Relation, RuleLevel, Status, StepLog, StepLogBuilder, Trigram, WuXing,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MeihuaError {
    #[error("九色 color_id 越界: {0}（应为 1..=9）")]
    InvalidColor(u8),
    #[error("生肖 aux_id 越界: {0}（应为 1..=12）")]
    InvalidAux(u8),
    #[error("传统数字必须为正整数")]
    InvalidTraditional,
    #[error("步骤日志构建失败: {0}")]
    Step(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeihuaInput {
    NineColor {
        color: u8,
        aux: u8,
        q_mod: u32,
        b_mod: u32,
    },
    Traditional {
        u0: u32,
        d0: u32,
        q_mod: u32,
        b_mod: u32,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct HexOut {
    pub name: String,
    pub upper: String,
    pub lower: String,
    pub binary_bottom_up: String,
}

impl From<Hexagram> for HexOut {
    fn from(h: Hexagram) -> Self {
        Self {
            name: h.name().to_string(),
            upper: h.upper.name().to_string(),
            lower: h.lower.name().to_string(),
            binary_bottom_up: h.binary_bottom_up(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WangShuaiEntry {
    pub trigram: String,
    pub element: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MeihuaResult {
    pub scheme: String,
    pub original: HexOut,
    pub mutual: HexOut,
    pub changed: HexOut,
    pub moving_line: u8,
    pub ti: String,
    pub yong: String,
    pub ti_element: String,
    pub yong_element: String,
    pub wangshuai: Vec<WangShuaiEntry>,
    pub jishen: String,
    pub steps: Vec<StepLog>,
    pub status: Status,
    pub formal_allowed: bool,
    pub disclaimer: String,
}

pub const DISCLAIMER: &str = "本工具仅供个人学习与娱乐，不构成任何决策建议；卦象解释存在流派差异。";

pub fn cast(input: MeihuaInput, month_branch: u8) -> Result<MeihuaResult, MeihuaError> {
    let (upper, lower, moving, scheme, rule_id, level, formula) = match input {
        MeihuaInput::NineColor {
            color,
            aux,
            q_mod,
            b_mod,
        } => {
            if !(1..=9).contains(&color) {
                return Err(MeihuaError::InvalidColor(color));
            }
            if !(1..=12).contains(&aux) {
                return Err(MeihuaError::InvalidAux(aux));
            }
            let u_num = mod_1_to(color as u32, 8);
            let d_num = mod_1_to(aux as u32, 8);
            let total = color as u32 + aux as u32 + q_mod + b_mod;
            let m = mod_1_to(total, 6);
            (
                Trigram::from_number(u_num as u8).expect("1..=8"),
                Trigram::from_number(d_num as u8).expect("1..=8"),
                m as u8,
                "NINE_COLOR",
                "MEIHUA-QIGUA-001",
                RuleLevel::L3,
                format!("U=C mod 8, D=A mod 8, M=(C+A+Q+B) mod 6；本例 C={color}, A={aux}, Q={q_mod}, B={b_mod}"),
            )
        }
        MeihuaInput::Traditional {
            u0,
            d0,
            q_mod,
            b_mod,
        } => {
            if u0 == 0 || d0 == 0 {
                return Err(MeihuaError::InvalidTraditional);
            }
            let u_num = mod_1_to(u0 + q_mod, 8);
            let d_num = mod_1_to(d0 + b_mod, 8);
            let m = mod_1_to(u0 + d0 + q_mod + b_mod, 6);
            (
                Trigram::from_number(u_num as u8).expect("1..=8"),
                Trigram::from_number(d_num as u8).expect("1..=8"),
                m as u8,
                "TRADITIONAL_NUMBER",
                "MEIHUA-QIGUA-002",
                RuleLevel::L2,
                format!("U=U0 mod 8, D=D0 mod 8, M=(U0+D0) mod 6；本例 U0={u0}, D0={d0}"),
            )
        }
    };

    let original = Hexagram::new(upper, lower);
    let lines = original.lines();

    let mutual = Hexagram::from_lines([lines[1], lines[2], lines[3], lines[2], lines[3], lines[4]]);

    let mut changed_lines = lines;
    changed_lines[(moving - 1) as usize] = !changed_lines[(moving - 1) as usize];
    let changed = Hexagram::from_lines(changed_lines);

    let (ti, yong) = if moving <= 3 {
        (upper, lower)
    } else {
        (lower, upper)
    };
    let month_element = WuXing::from_branch(month_branch);
    let ti_relation = ti.element().relation_to(month_element);
    let yong_relation = yong.element().relation_to(month_element);

    let mut steps = Vec::new();

    steps.push(
        StepLogBuilder::new(0, "MEIHUA-QIGUA", Engine::Meihua)
            .rule_id(rule_id)
            .data_ref(if scheme == "NINE_COLOR" {
                "COLOR-NUM-02"
            } else {
                "G-01"
            })
            .data_ref("G-01")
            .source("SRC-MEIHUA-001")
            .level(level)
            .formula(formula)
            .output(serde_json::json!({
                "upper": original.upper.name(),
                "lower": original.lower.name(),
                "moving_line": moving,
            }))
            .build()
            .map_err(|e| MeihuaError::Step(e.to_string()))?,
    );

    steps.push(
        StepLogBuilder::new(1, "MEIHUA-HUGUA", Engine::Meihua)
            .rule_id("MEIHUA-HUGUA-001")
            .data_ref("HEX-01")
            .source("SRC-MEIHUA-002")
            .level(RuleLevel::L2)
            .formula("lower_hu=lines[1..4], upper_hu=lines[2..5]")
            .output(serde_json::json!({
                "name": mutual.name(),
                "upper": mutual.upper.name(),
                "lower": mutual.lower.name(),
            }))
            .build()
            .map_err(|e| MeihuaError::Step(e.to_string()))?,
    );

    steps.push(
        StepLogBuilder::new(2, "MEIHUA-BIANGUA", Engine::Meihua)
            .rule_id("MEIHUA-BIANGUA-001")
            .data_ref("HEX-01")
            .source("SRC-MEIHUA-003")
            .level(RuleLevel::L2)
            .formula(format!("翻转第 {moving} 爻"))
            .output(serde_json::json!({
                "name": changed.name(),
                "upper": changed.upper.name(),
                "lower": changed.lower.name(),
                "moving_line": moving,
            }))
            .build()
            .map_err(|e| MeihuaError::Step(e.to_string()))?,
    );

    steps.push(
        StepLogBuilder::new(3, "MEIHUA-TIYONG", Engine::Meihua)
            .rule_id("MEIHUA-TIYONG-001")
            .data_ref("G-01")
            .source("SRC-MEIHUA-004")
            .level(RuleLevel::L2)
            .formula("动爻 1—3 下用上体；4—6 上用下体")
            .output(serde_json::json!({
                "ti": ti.name(),
                "yong": yong.name(),
                "ti_element": ti.element().name(),
                "yong_element": yong.element().name(),
            }))
            .build()
            .map_err(|e| MeihuaError::Step(e.to_string()))?,
    );

    steps.push(
        StepLogBuilder::new(4, "MEIHUA-WANGSHUAI", Engine::Meihua)
            .rule_id("MEIHUA-WANGSHUAI-001")
            .data_ref("W-01")
            .source("SRC-MEIHUA-005")
            .level(RuleLevel::L2)
            .formula("同我旺、生我相、我生休、我克囚、克我死")
            .status(Status::Pending)
            .output(serde_json::json!({
                "month_branch": month_branch,
                "month_element": month_element.name(),
                "ti": ti_relation.name(),
                "yong": yong_relation.name(),
                "note": "W-01 待校核"
            }))
            .build()
            .map_err(|e| MeihuaError::Step(e.to_string()))?,
    );

    // FR-012：忌神/弱势卦位在 ARS 未定义算法，阶段 0 阻断正式结论。
    steps.push(
        StepLogBuilder::new(5, "MEIHUA-JISHEN", Engine::Meihua)
            .rule_id("MEIHUA-JISHEN-001")
            .data_ref("W-01")
            .source("SRC-MEIHUA-006")
            .level(RuleLevel::Provisional)
            .formula("待冻结：ARS 未定义忌神/弱势卦位算法")
            .status(Status::Pending)
            .output(serde_json::json!({
                "jishen": "PENDING",
                "formal_allowed": false,
                "note": "忌神/弱势卦位规则待冻结，方位象意不得发布正式结果"
            }))
            .build()
            .map_err(|e| MeihuaError::Step(e.to_string()))?,
    );

    Ok(MeihuaResult {
        scheme: scheme.into(),
        original: original.into(),
        mutual: mutual.into(),
        changed: changed.into(),
        moving_line: moving,
        ti: ti.name().into(),
        yong: yong.name().into(),
        ti_element: ti.element().name().into(),
        yong_element: yong.element().name().into(),
        wangshuai: vec![
            WangShuaiEntry {
                trigram: ti.name().into(),
                element: ti.element().name().into(),
                relation: ti_relation.name().into(),
            },
            WangShuaiEntry {
                trigram: yong.name().into(),
                element: yong.element().name().into(),
                relation: yong_relation.name().into(),
            },
        ],
        jishen: "PENDING".into(),
        steps,
        status: Status::Pending,
        formal_allowed: false,
        disclaimer: DISCLAIMER.into(),
    })
}

fn mod_1_to(value: u32, modulus: u32) -> u32 {
    let r = value % modulus;
    if r == 0 {
        modulus
    } else {
        r
    }
}

/// 梅花结果中的干支仅用于展示月度旺衰上下文，不参与卦象数字。
pub fn month_context(gz: GanZhi) -> (u8, WuXing, Relation) {
    let branch = gz.branch;
    let element = WuXing::from_branch(branch);
    (branch, element, element.relation_to(element))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_nine_color() {
        let r = cast(
            MeihuaInput::NineColor {
                color: 1,
                aux: 3,
                q_mod: 0,
                b_mod: 0,
            },
            10, // 戌月
        )
        .unwrap();
        assert_eq!(r.original.name, "天火同人");
        assert_eq!(r.moving_line, 4);
        assert_eq!(r.changed.name, "风火家人");
        assert_eq!(r.mutual.name, "天风姤");
        assert_eq!(r.ti, "离");
        assert_eq!(r.yong, "乾");
        assert_eq!(r.status, Status::Pending);
        assert!(!r.formal_allowed);
    }

    #[test]
    fn golden_traditional() {
        let r = cast(
            MeihuaInput::Traditional {
                u0: 9,
                d0: 16,
                q_mod: 0,
                b_mod: 0,
            },
            10,
        )
        .unwrap();
        assert_eq!(r.original.name, "天地否");
        assert_eq!(r.moving_line, 1);
        assert_eq!(r.changed.name, "天雷无妄");
        assert_eq!(r.ti, "乾");
        assert_eq!(r.yong, "坤");
    }

    #[test]
    fn q_b_perturbation_changes_moving_line() {
        let a = cast(
            MeihuaInput::NineColor {
                color: 1,
                aux: 3,
                q_mod: 0,
                b_mod: 0,
            },
            10,
        )
        .unwrap();
        let b = cast(
            MeihuaInput::NineColor {
                color: 1,
                aux: 3,
                q_mod: 1,
                b_mod: 0,
            },
            10,
        )
        .unwrap();
        assert_ne!(a.moving_line, b.moving_line);
    }
}
