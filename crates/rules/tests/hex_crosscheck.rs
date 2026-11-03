use std::collections::HashSet;
use std::path::PathBuf;

use domain::{Hexagram, Trigram};
use serde::Deserialize;

#[derive(Deserialize)]
struct HexFile {
    hexagrams: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    name: String,
    upper: String,
    lower: String,
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

#[test]
fn hex64_resource_matches_domain_table() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../resources/data/hex64.json");
    let text = std::fs::read_to_string(path).unwrap();
    let file: HexFile = serde_json::from_str(&text).unwrap();
    assert_eq!(file.hexagrams.len(), 64);

    let mut seen = HashSet::new();
    for entry in file.hexagrams {
        let h = Hexagram::new(trigram(&entry.upper), trigram(&entry.lower));
        assert_eq!(
            h.name(),
            entry.name,
            "HEX-01 与 domain 名称表不一致: 上{} 下{}",
            entry.upper,
            entry.lower
        );
        assert!(seen.insert(entry.name.clone()), "重复卦名: {}", entry.name);
    }
    assert_eq!(seen.len(), 64);
}
