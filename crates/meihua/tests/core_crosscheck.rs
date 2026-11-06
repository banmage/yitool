//! 梅花核心规则校核：金标准逐项比对 + 全 64 卦 / 六动爻独立 oracle 属性测试。
//!
//! 本测试不复用 `meihua::cast` 的中间结果：原始 64 卦名称取自已校核的
//! `HEX-01-v0.2` 资源，互卦/变卦/体用由测试内的三爻位运算独立推导。

use std::collections::BTreeMap;
use std::path::PathBuf;

use meihua::{cast, MeihuaInput};
use serde_json::Value;

/// 三爻自下而上；true = 阳。顺序与先天数 1..=8 一致。
const TRIGRAMS: [(&str, [bool; 3], &str); 8] = [
    ("乾", [true, true, true], "金"),
    ("兑", [true, true, false], "金"),
    ("离", [true, false, true], "火"),
    ("震", [true, false, false], "木"),
    ("巽", [false, true, true], "木"),
    ("坎", [false, true, false], "水"),
    ("艮", [false, false, true], "土"),
    ("坤", [false, false, false], "土"),
];

fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn hexagram_names() -> BTreeMap<(String, String), String> {
    let path = repo_path("../../resources/data/hex64.json");
    let data: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("读取 hex64.json")).unwrap();
    assert_eq!(data["version"], "HEX-01-v0.2");
    let mut names = BTreeMap::new();
    for hex in data["hexagrams"].as_array().unwrap() {
        let upper = hex["upper"].as_str().unwrap().to_string();
        let lower = hex["lower"].as_str().unwrap().to_string();
        let name = hex["name"].as_str().unwrap().to_string();
        assert!(
            names.insert((upper, lower), name).is_none(),
            "HEX-01 存在重复上下卦组合"
        );
    }
    assert_eq!(names.len(), 64, "HEX-01 必须覆盖 64 卦");
    names
}

fn bits_to_name(names: &BTreeMap<(String, String), String>, bits: [bool; 6]) -> String {
    let mut from_lines = BTreeMap::new();
    for (name, lines, _) in TRIGRAMS {
        from_lines.insert(lines, name);
    }
    let lower = from_lines[&[bits[0], bits[1], bits[2]]];
    let upper = from_lines[&[bits[3], bits[4], bits[5]]];
    names
        .get(&(upper.to_string(), lower.to_string()))
        .unwrap_or_else(|| panic!("HEX-01 缺少 {upper}上{lower}下"))
        .clone()
}

fn bits_string(bits: [bool; 6]) -> String {
    bits.iter().map(|b| if *b { '1' } else { '0' }).collect()
}

fn element(trigram: &str) -> &'static str {
    TRIGRAMS
        .iter()
        .find(|(name, _, _)| *name == trigram)
        .map(|(_, _, element)| *element)
        .expect("未知三爻卦")
}

fn mod_1_to(value: u32, modulus: u32) -> u32 {
    let r = value % modulus;
    if r == 0 {
        modulus
    } else {
        r
    }
}

#[test]
fn golden_cases_match_independent_recomputation() {
    let path = repo_path("../../tests/golden/meihua_core.json");
    let data: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("读取梅花核心金标准")).unwrap();
    assert_eq!(data["version"], "MEIHUA-CORE-v0.2");
    let cases = data["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 17, "金标准案例数变化须同步校核记录");

    // 校核覆盖：双模式、六种动爻、Q 与 B 扰动均须至少出现一次。
    let mut schemes = BTreeMap::new();
    let mut moving_lines = BTreeMap::new();
    let mut nonzero_q = 0;
    let mut nonzero_b = 0;
    for case in cases {
        *schemes
            .entry(case["scheme"].as_str().unwrap().to_string())
            .or_insert(0) += 1;
        *moving_lines
            .entry(case["expected"]["moving_line"].as_u64().unwrap())
            .or_insert(0) += 1;
        if case["input"]["q_mod"].as_u64().unwrap() != 0 {
            nonzero_q += 1;
        }
        if case["input"]["b_mod"].as_u64().unwrap() != 0 {
            nonzero_b += 1;
        }
    }
    assert_eq!(
        schemes.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["NINE_COLOR", "TRADITIONAL_NUMBER"],
        "金标准必须同时覆盖九色与传统数字"
    );
    assert_eq!(
        moving_lines.keys().copied().collect::<Vec<_>>(),
        vec![1u64, 2, 3, 4, 5, 6],
        "金标准必须覆盖初爻至上爻全部动爻"
    );
    assert!(nonzero_q > 0, "金标准必须覆盖 Q 扰动");
    assert!(nonzero_b > 0, "金标准必须覆盖 B 扰动");

    let names = hexagram_names();
    for case in cases {
        let case_id = case["case_id"].as_str().unwrap();
        let scheme = case["scheme"].as_str().unwrap();
        let input = &case["input"];
        let q_mod = input["q_mod"].as_u64().unwrap() as u32;
        let b_mod = input["b_mod"].as_u64().unwrap() as u32;

        let (meihua_input, upper_num, lower_num, total) = if scheme == "NINE_COLOR" {
            let color = input["color"].as_u64().unwrap() as u32;
            let aux = input["aux"].as_u64().unwrap() as u32;
            (
                MeihuaInput::NineColor {
                    color: color as u8,
                    aux: aux as u8,
                    q_mod,
                    b_mod,
                },
                mod_1_to(color, 8),
                mod_1_to(aux, 8),
                color + aux + q_mod + b_mod,
            )
        } else {
            let u0 = input["u0"].as_u64().unwrap() as u32;
            let d0 = input["d0"].as_u64().unwrap() as u32;
            (
                MeihuaInput::Traditional {
                    u0,
                    d0,
                    q_mod,
                    b_mod,
                },
                mod_1_to(u0 + q_mod, 8),
                mod_1_to(d0 + b_mod, 8),
                u0 + d0 + q_mod + b_mod,
            )
        };
        let moving = mod_1_to(total, 6) as usize;
        assert_eq!(
            moving as u64,
            case["expected"]["moving_line"].as_u64().unwrap(),
            "{case_id} 金标准动爻与独立复算不一致"
        );

        let upper = TRIGRAMS[(upper_num - 1) as usize].0;
        let lower = TRIGRAMS[(lower_num - 1) as usize].0;
        let upper_lines = TRIGRAMS[(upper_num - 1) as usize].1;
        let lower_lines = TRIGRAMS[(lower_num - 1) as usize].1;
        let original_bits = [
            lower_lines[0],
            lower_lines[1],
            lower_lines[2],
            upper_lines[0],
            upper_lines[1],
            upper_lines[2],
        ];

        let mut changed_bits = original_bits;
        changed_bits[moving - 1] = !changed_bits[moving - 1];

        // 互卦：2/3/4 爻为下互，3/4/5 爻为上互（六爻自下而上）。
        let mutual_bits = [
            original_bits[1],
            original_bits[2],
            original_bits[3],
            original_bits[2],
            original_bits[3],
            original_bits[4],
        ];

        let (expected_ti, expected_yong) = if moving <= 3 {
            (upper, lower)
        } else {
            (lower, upper)
        };

        let expected = &case["expected"];
        let assert_field = |field: &str, actual: &str, expected_value: &str| {
            assert_eq!(actual, expected_value, "{case_id} {field} 与手工复算不一致");
        };

        assert_field(
            "original",
            &names
                .get(&(upper.to_string(), lower.to_string()))
                .unwrap()
                .clone(),
            expected["original"].as_str().unwrap(),
        );
        assert_field(
            "changed",
            &bits_to_name(&names, changed_bits),
            expected["changed"].as_str().unwrap(),
        );
        assert_field(
            "mutual",
            &bits_to_name(&names, mutual_bits),
            expected["mutual"].as_str().unwrap(),
        );
        assert_field("ti", expected_ti, expected["ti"].as_str().unwrap());
        assert_field("yong", expected_yong, expected["yong"].as_str().unwrap());

        let result = cast(meihua_input, 10).unwrap();
        assert_eq!(result.moving_line as usize, moving, "{case_id} 动爻不一致");
        assert_eq!(result.original.name, expected["original"].as_str().unwrap());
        assert_eq!(
            result.original.binary_bottom_up,
            bits_string(original_bits),
            "{case_id} 本卦六爻位不一致"
        );
        assert_eq!(result.changed.name, expected["changed"].as_str().unwrap());
        assert_eq!(result.mutual.name, expected["mutual"].as_str().unwrap());
        assert_eq!(result.ti, expected_ti);
        assert_eq!(result.yong, expected_yong);
        assert_eq!(result.ti_element, element(expected_ti));
        assert_eq!(result.yong_element, element(expected_yong));
    }
}

#[test]
fn all_64_hexagrams_and_six_moving_lines_match_oracle() {
    let names = hexagram_names();

    for upper_num in 1..=8u32 {
        for lower_num in 1..=8u32 {
            for moving in 1..=6u32 {
                // 九色模式：color/aux 直接映射先天数；Q 只补足动爻，不改变上下卦。
                let q_mod = (moving + 6 - (upper_num + lower_num) % 6) % 6;
                let result = cast(
                    MeihuaInput::NineColor {
                        color: upper_num as u8,
                        aux: lower_num as u8,
                        q_mod,
                        b_mod: 0,
                    },
                    10,
                )
                .unwrap();
                assert_eq!(result.moving_line, moving as u8);

                let upper_lines = TRIGRAMS[(upper_num - 1) as usize].1;
                let lower_lines = TRIGRAMS[(lower_num - 1) as usize].1;
                let original_bits = [
                    lower_lines[0],
                    lower_lines[1],
                    lower_lines[2],
                    upper_lines[0],
                    upper_lines[1],
                    upper_lines[2],
                ];
                let mut changed_bits = original_bits;
                changed_bits[(moving - 1) as usize] = !changed_bits[(moving - 1) as usize];
                let mutual_bits = [
                    original_bits[1],
                    original_bits[2],
                    original_bits[3],
                    original_bits[2],
                    original_bits[3],
                    original_bits[4],
                ];

                let upper = TRIGRAMS[(upper_num - 1) as usize].0;
                let lower = TRIGRAMS[(lower_num - 1) as usize].0;
                let (ti, yong) = if moving <= 3 {
                    (upper, lower)
                } else {
                    (lower, upper)
                };

                let context = format!("上{upper}下{lower}动{moving}");
                assert_eq!(
                    result.original.name,
                    names[&(upper.to_string(), lower.to_string())],
                    "{context} 本卦错误"
                );
                assert_eq!(
                    result.original.binary_bottom_up,
                    bits_string(original_bits),
                    "{context} 本卦位错误"
                );
                assert_eq!(
                    result.mutual.name,
                    bits_to_name(&names, mutual_bits),
                    "{context} 互卦错误"
                );
                assert_eq!(
                    result.changed.name,
                    bits_to_name(&names, changed_bits),
                    "{context} 变卦错误"
                );
                assert_eq!(result.ti, ti, "{context} 体卦错误");
                assert_eq!(result.yong, yong, "{context} 用卦错误");
                assert_eq!(result.ti_element, element(ti), "{context} 体五行错误");
                assert_eq!(result.yong_element, element(yong), "{context} 用五行错误");
            }
        }
    }
}

#[test]
fn traditional_mode_matches_nine_color_when_no_perturbation() {
    for upper_num in 1..=8u8 {
        for lower_num in 1..=8u8 {
            // 两者无扰动时公式等价：U=U0/color mod 8、D=D0/aux mod 8、
            // M=(U0+D0)/(C+A) mod 6。
            let traditional = cast(
                MeihuaInput::Traditional {
                    u0: upper_num as u32,
                    d0: lower_num as u32,
                    q_mod: 0,
                    b_mod: 0,
                },
                10,
            )
            .unwrap();
            let nine = cast(
                MeihuaInput::NineColor {
                    color: upper_num,
                    aux: lower_num,
                    q_mod: 0,
                    b_mod: 0,
                },
                10,
            )
            .unwrap();
            assert_eq!(
                traditional.moving_line,
                mod_1_to(upper_num as u32 + lower_num as u32, 6) as u8
            );
            assert_eq!(nine.moving_line, traditional.moving_line);
            assert_eq!(nine.original.name, traditional.original.name);
            assert_eq!(nine.mutual.name, traditional.mutual.name);
            assert_eq!(nine.changed.name, traditional.changed.name);
            assert_eq!(nine.ti, traditional.ti);
            assert_eq!(nine.yong, traditional.yong);
        }
    }
}
