//! HEX-01 六十四卦八宫表交叉校验：金标准 ↔ 资源 ↔ domain 卦象/八宫生成规则。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use domain::{Hexagram, Trigram};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Golden {
    id: String,
    version: String,
    status: String,
    source: String,
    types: Vec<String>,
    shi_yao: Vec<u8>,
    ying_yao: Vec<u8>,
    palaces: Vec<Palace>,
}

#[derive(Debug, Deserialize)]
struct Palace {
    palace: String,
    palace_index: u8,
    palace_element: String,
    members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct HexFile {
    version: String,
    status: String,
    source: String,
    hexagrams: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    palace: String,
    palace_index: u8,
    palace_element: String,
    palace_seq: u8,
    name: String,
    upper: String,
    lower: String,
    palace_kind: String,
    shi_yao: u8,
    ying_yao: u8,
}

fn load<T: for<'de> Deserialize<'de>>(relative: &str) -> T {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("解析 {} 失败: {e}", path.display()))
}

fn trigram(name: &str) -> Trigram {
    match name {
        "乾" => Trigram::Qian,
        "兑" => Trigram::Dui,
        "离" => Trigram::Li,
        "震" => Trigram::Zhen,
        "巽" => Trigram::Xun,
        "坎" => Trigram::Kan,
        "艮" => Trigram::Gen,
        "坤" => Trigram::Kun,
        other => panic!("未知卦名: {other}"),
    }
}

fn golden() -> Golden {
    load("../../tests/golden/hex01_bagong.json")
}

#[test]
fn golden_is_complete() {
    let golden = golden();
    assert_eq!(golden.id, "HEX-01");
    assert_eq!(golden.version, "HEX-01-v0.2");
    assert_eq!(golden.status, "VERIFIED");
    assert_eq!(golden.palaces.len(), 8);
    assert_eq!(golden.types.len(), 8);
    assert_eq!(golden.shi_yao.len(), 8);
    assert_eq!(golden.ying_yao.len(), 8);

    let mut names = HashSet::new();
    let mut palaces = HashSet::new();
    for palace in &golden.palaces {
        assert!(palaces.insert(palace.palace.clone()), "重复宫位");
        assert_eq!(palace.members.len(), 8, "{} 宫成员数错误", palace.palace);
        for name in &palace.members {
            assert!(names.insert(name.clone()), "重复卦名: {name}");
        }
    }
    assert_eq!(names.len(), 64);
    assert_eq!(palaces.len(), 8);
}

#[test]
fn resource_matches_golden() {
    let golden = golden();
    let resource: HexFile = load("../../resources/data/hex64.json");

    assert_eq!(resource.version, golden.version);
    assert_eq!(resource.status, golden.status);
    assert_eq!(resource.source, golden.source);
    assert_eq!(resource.hexagrams.len(), 64);

    let by_name: HashMap<_, _> = resource
        .hexagrams
        .iter()
        .map(|entry| (entry.name.as_str(), entry))
        .collect();
    assert_eq!(by_name.len(), 64, "卦名必须唯一");

    for palace in &golden.palaces {
        for (offset, name) in palace.members.iter().enumerate() {
            let entry = by_name
                .get(name.as_str())
                .unwrap_or_else(|| panic!("资源缺少卦: {name}"));
            assert_eq!(entry.palace, palace.palace, "{name} 宫位错误");
            assert_eq!(entry.palace_index, palace.palace_index, "{name} 宫序号错误");
            assert_eq!(
                entry.palace_element, palace.palace_element,
                "{name} 宫五行错误"
            );
            assert_eq!(entry.palace_seq, offset as u8 + 1, "{name} 宫内序错误");
            assert_eq!(entry.palace_kind, golden.types[offset], "{name} 卦类型错误");
            assert_eq!(entry.shi_yao, golden.shi_yao[offset], "{name} 世爻错误");
            assert_eq!(entry.ying_yao, golden.ying_yao[offset], "{name} 应爻错误");
        }
    }
}

#[test]
fn resource_upper_lower_matches_domain_names() {
    let resource: HexFile = load("../../resources/data/hex64.json");
    for entry in &resource.hexagrams {
        let hexagram = Hexagram::new(trigram(&entry.upper), trigram(&entry.lower));
        assert_eq!(
            hexagram.name(),
            entry.name,
            "上{} 下{} 的卦名与 domain 不一致",
            entry.upper,
            entry.lower
        );
    }
}

#[test]
fn eight_palace_generation_matches_golden() {
    let golden = golden();
    let resource: HexFile = load("../../resources/data/hex64.json");
    let by_name: HashMap<_, _> = resource
        .hexagrams
        .iter()
        .map(|entry| (entry.name.as_str(), entry))
        .collect();

    for palace in &golden.palaces {
        let pure = by_name
            .get(palace.members[0].as_str())
            .unwrap_or_else(|| panic!("资源缺少本宫卦: {}", palace.members[0]));
        let mut lines = Hexagram::new(trigram(&pure.upper), trigram(&pure.lower)).lines();

        // 八宫生成：初爻起逐爻变，五世后游魂回变四爻，归魂恢复内三爻。
        for offset in 1..=5 {
            lines[offset - 1] = !lines[offset - 1];
            assert_eq!(
                Hexagram::from_lines(lines).name(),
                palace.members[offset],
                "{} 宫第 {} 卦生成结果不符",
                palace.palace,
                offset + 1
            );
        }
        lines[3] = !lines[3];
        assert_eq!(
            Hexagram::from_lines(lines).name(),
            palace.members[6],
            "{} 宫游魂卦生成结果不符",
            palace.palace
        );
        for line in &mut lines[0..3] {
            *line = !*line;
        }
        assert_eq!(
            Hexagram::from_lines(lines).name(),
            palace.members[7],
            "{} 宫归魂卦生成结果不符",
            palace.palace
        );
    }
}
