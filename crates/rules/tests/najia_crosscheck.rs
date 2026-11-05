//! NJ-01 纳甲干支表交叉校验：金标准 ↔ 资源 ↔ 天干/地支装配规则。

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use serde::Deserialize;

const STEMS: [char; 10] = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
const BRANCHES: [char; 12] = [
    '子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥',
];

#[derive(Debug, Deserialize)]
struct Golden {
    id: String,
    version: String,
    status: String,
    source: String,
    stems: BTreeMap<String, Stems>,
    trigrams: Vec<TrigramLines>,
}

#[derive(Debug, Deserialize)]
struct Stems {
    inner: String,
    outer: String,
}

#[derive(Debug, Deserialize)]
struct TrigramLines {
    trigram: String,
    lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Resource {
    id: String,
    version: String,
    status: String,
    source: String,
    trigrams: Vec<TrigramLines>,
}

fn load<T: for<'de> Deserialize<'de>>(relative: &str) -> T {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("解析 {} 失败: {e}", path.display()))
}

fn golden() -> Golden {
    load("../../tests/golden/nj01_najia.json")
}

fn split(line: &str) -> (char, char) {
    let mut chars = line.chars();
    let stem = chars.next().unwrap_or_else(|| panic!("空干支: {line}"));
    let branch = chars.next().unwrap_or_else(|| panic!("干支缺地支: {line}"));
    assert!(chars.next().is_none(), "干支长度错误: {line}");
    (stem, branch)
}

#[test]
fn golden_is_complete() {
    let golden = golden();
    assert_eq!(golden.id, "NJ-01");
    assert_eq!(golden.version, "NJ-01-v0.2");
    assert_eq!(golden.status, "VERIFIED");
    assert_eq!(golden.trigrams.len(), 8);
    assert_eq!(golden.stems.len(), 8);

    let mut lines = HashSet::new();
    for trigram in &golden.trigrams {
        assert_eq!(trigram.lines.len(), 6, "{} 爻数错误", trigram.trigram);
        for line in &trigram.lines {
            split(line);
            assert!(lines.insert(line.clone()), "重复干支: {line}");
        }
    }
    assert_eq!(lines.len(), 48);
}

#[test]
fn resource_matches_golden() {
    let golden = golden();
    let resource: Resource = load("../../resources/data/najia.json");

    assert_eq!(resource.id, golden.id);
    assert_eq!(resource.version, golden.version);
    assert_eq!(resource.status, golden.status);
    assert_eq!(resource.source, golden.source);
    assert_eq!(resource.trigrams.len(), golden.trigrams.len());

    for (actual, expected) in resource.trigrams.iter().zip(&golden.trigrams) {
        assert_eq!(actual.trigram, expected.trigram);
        assert_eq!(actual.lines, expected.lines);
    }
}

#[test]
fn stems_follow_inner_outer_rule() {
    let golden = golden();
    for trigram in &golden.trigrams {
        let stems = golden
            .stems
            .get(&trigram.trigram)
            .unwrap_or_else(|| panic!("缺少 {} 纳干", trigram.trigram));
        for (index, line) in trigram.lines.iter().enumerate() {
            let (stem, _) = split(line);
            let expected = if index < 3 {
                stems.inner.chars().next().unwrap()
            } else {
                stems.outer.chars().next().unwrap()
            };
            assert_eq!(
                stem,
                expected,
                "{} 第 {} 爻天干错误：期望 {expected}，实际 {stem}",
                trigram.trigram,
                index + 1
            );
        }
    }
}

#[test]
fn branches_follow_yang_forward_yin_backward_rule() {
    let golden = golden();
    for trigram in &golden.trigrams {
        let (forward, start) = match trigram.trigram.as_str() {
            "乾" => (true, '子'),
            "震" => (true, '子'),
            "坎" => (true, '寅'),
            "艮" => (true, '辰'),
            "坤" => (false, '未'),
            "巽" => (false, '丑'),
            "离" => (false, '卯'),
            "兑" => (false, '巳'),
            other => panic!("未知纳甲卦: {other}"),
        };
        let mut index = BRANCHES.iter().position(|b| *b == start).unwrap() as isize;
        for (offset, line) in trigram.lines.iter().enumerate() {
            let (_, branch) = split(line);
            let expected = BRANCHES[index as usize];
            assert_eq!(
                branch,
                expected,
                "{} 第 {} 爻地支错误：期望 {expected}，实际 {branch}",
                trigram.trigram,
                offset + 1
            );
            index = if forward {
                (index + 2) % 12
            } else {
                (index - 2).rem_euclid(12)
            };
        }
    }
}

#[test]
fn all_stems_and_branches_are_valid() {
    let golden = golden();
    for trigram in &golden.trigrams {
        for line in &trigram.lines {
            let (stem, branch) = split(line);
            assert!(STEMS.contains(&stem), "非法天干: {stem}");
            assert!(BRANCHES.contains(&branch), "非法地支: {branch}");
        }
    }
}
