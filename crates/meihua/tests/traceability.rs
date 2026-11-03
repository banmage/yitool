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
fn pending_rules_do_not_produce_formal_result() {
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
    assert_eq!(result.jishen, "PENDING");
    assert!(result
        .steps
        .iter()
        .any(|s| s.step_id == "MEIHUA-JISHEN" && s.status == Status::Pending));
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
