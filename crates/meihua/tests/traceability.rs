use domain::{RuleLevel, Status};
use meihua::{cast, MeihuaInput};

#[test]
fn every_step_has_full_traceability() {
    let result = cast(
        MeihuaInput::NineColor {
            color: 1,
            aux: 3,
            q_mod: 0,
            b_mod: 0,
        },
        10,
    )
    .unwrap();

    assert_eq!(result.steps.len(), 6);
    for step in &result.steps {
        assert!(!step.rule_ids.is_empty(), "{} 缺 rule_ids", step.step_id);
        assert!(!step.data_refs.is_empty(), "{} 缺 data_refs", step.step_id);
        assert!(
            !step.source_ref.is_empty(),
            "{} 缺 source_ref",
            step.step_id
        );
        assert!(matches!(
            step.rule_level,
            RuleLevel::L1 | RuleLevel::L2 | RuleLevel::L3 | RuleLevel::Provisional
        ));
        assert!(!step.formula_summary.is_empty());
    }
}

#[test]
fn verified_core_rules_do_not_release_formal_result() {
    let result = cast(
        MeihuaInput::Traditional {
            u0: 9,
            d0: 16,
            q_mod: 0,
            b_mod: 0,
        },
        10,
    )
    .unwrap();
    assert_eq!(result.status, Status::Pending);
    assert!(!result.formal_allowed);
    // 体乾金 → 忌神火；弱势卦位为同属火的离（南）。
    assert_eq!(result.jishen.element, "火");
    assert_eq!(result.jishen.weak_positions[0].direction, "南");
    // ADR-021：六步的核心规则均已校核。
    for step in &result.steps {
        assert_eq!(
            step.status,
            Status::Verified,
            "{} 应为 VERIFIED",
            step.step_id
        );
    }
    // 核心规则已校核，但解释规则与焦氏等仍未完成，整体不得转为正式结果。
}

#[test]
fn nine_color_is_l3_not_l1() {
    let result = cast(
        MeihuaInput::NineColor {
            color: 1,
            aux: 3,
            q_mod: 0,
            b_mod: 0,
        },
        10,
    )
    .unwrap();
    let qigua = result
        .steps
        .iter()
        .find(|s| s.step_id == "MEIHUA-QIGUA")
        .unwrap();
    assert_eq!(qigua.rule_level, RuleLevel::L3);
}

#[test]
fn invalid_inputs_are_rejected() {
    assert!(cast(
        MeihuaInput::NineColor {
            color: 0,
            aux: 3,
            q_mod: 0,
            b_mod: 0
        },
        10
    )
    .is_err());
    assert!(cast(
        MeihuaInput::Traditional {
            u0: 0,
            d0: 1,
            q_mod: 0,
            b_mod: 0
        },
        10
    )
    .is_err());
}
