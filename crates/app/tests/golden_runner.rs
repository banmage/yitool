use std::path::PathBuf;

use meihua::{cast, MeihuaInput};
use serde::Deserialize;

#[derive(Deserialize)]
struct Golden {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    case_id: String,
    rule_id: String,
    input: Input,
    month_branch: u8,
    expected: Expected,
}

#[derive(Deserialize)]
struct Input {
    scheme: String,
    color: Option<u8>,
    aux: Option<u8>,
    u0: Option<u32>,
    d0: Option<u32>,
    q_mod: u32,
    b_mod: u32,
}

#[derive(Deserialize)]
struct Expected {
    original: String,
    moving_line: u8,
    changed: String,
    mutual: String,
    ti: String,
    yong: String,
}

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/meihua.json")
}

#[test]
fn golden_meihua_cases() {
    let text = std::fs::read_to_string(golden_path()).unwrap();
    let golden: Golden = serde_json::from_str(&text).unwrap();
    assert!(!golden.cases.is_empty());

    for case in golden.cases {
        let input = match case.input.scheme.as_str() {
            "NINE_COLOR" => MeihuaInput::NineColor {
                color: case.input.color.unwrap(),
                aux: case.input.aux.unwrap(),
                q_mod: case.input.q_mod,
                b_mod: case.input.b_mod,
            },
            "TRADITIONAL_NUMBER" => MeihuaInput::Traditional {
                u0: case.input.u0.unwrap(),
                d0: case.input.d0.unwrap(),
                q_mod: case.input.q_mod,
                b_mod: case.input.b_mod,
            },
            other => panic!("未知 scheme: {other}"),
        };
        let r = cast(input, case.month_branch).unwrap();
        assert_eq!(
            r.original.name, case.expected.original,
            "{} / {}: 本卦",
            case.case_id, case.rule_id
        );
        assert_eq!(
            r.moving_line, case.expected.moving_line,
            "{} / {}: 动爻",
            case.case_id, case.rule_id
        );
        assert_eq!(
            r.changed.name, case.expected.changed,
            "{}: 变卦",
            case.case_id
        );
        assert_eq!(
            r.mutual.name, case.expected.mutual,
            "{}: 互卦",
            case.case_id
        );
        assert_eq!(r.ti, case.expected.ti, "{}: 体", case.case_id);
        assert_eq!(r.yong, case.expected.yong, "{}: 用", case.case_id);
    }
}
