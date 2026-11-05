//! W-01 四时旺衰表交叉校验：金标准 ↔ 资源 ↔ domain 生克实现。

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use domain::{Relation, WuXing};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct WangShuaiFile {
    id: String,
    version: String,
    status: String,
    source: String,
    relation_rule: String,
    elements: Vec<String>,
    branches: Vec<BranchEntry>,
    table: Vec<MonthEntry>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct BranchEntry {
    branch: String,
    branch_id: u8,
    element: String,
    season: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct MonthEntry {
    month_branch: String,
    branch_id: u8,
    month_element: String,
    states: BTreeMap<String, String>,
}

fn load(relative: &str) -> WangShuaiFile {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("解析 {} 失败: {e}", path.display()))
}

fn element(name: &str) -> WuXing {
    WuXing::from_name(name).unwrap_or_else(|| panic!("未知五行: {name}"))
}

#[test]
fn golden_shape_is_complete() {
    let golden = load("../../tests/golden/w01_wangshuai.json");
    assert_eq!(golden.id, "W-01");
    assert_eq!(golden.version, "W-01-v0.2");
    assert_eq!(golden.status, "VERIFIED");
    assert_eq!(golden.elements.len(), 5);
    assert_eq!(golden.branches.len(), 12);
    assert_eq!(golden.table.len(), 12);

    let elements: HashSet<_> = golden.elements.iter().cloned().collect();
    assert_eq!(elements.len(), 5);
    for month in &golden.table {
        assert_eq!(
            month.states.keys().cloned().collect::<HashSet<_>>(),
            elements,
            "{} 月缺五行状态",
            month.month_branch
        );
    }
}

#[test]
fn resource_matches_golden() {
    let golden = load("../../tests/golden/w01_wangshuai.json");
    let resource = load("../../resources/data/wuxing_wangshuai.json");

    assert_eq!(resource.id, golden.id);
    assert_eq!(resource.version, golden.version);
    assert_eq!(resource.status, golden.status);
    assert_eq!(resource.source, golden.source);
    assert_eq!(resource.relation_rule, golden.relation_rule);
    assert_eq!(resource.elements, golden.elements);
    assert_eq!(resource.branches, golden.branches);
    assert_eq!(resource.table, golden.table);
}

#[test]
fn table_matches_classical_relation_rule() {
    let golden = load("../../tests/golden/w01_wangshuai.json");

    for (index, month) in golden.table.iter().enumerate() {
        assert_eq!(month.branch_id as usize, index, "月建顺序必须固定");
        assert_eq!(month.month_branch, golden.branches[index].branch);
        assert_eq!(month.month_element, golden.branches[index].element);

        let month_element = element(&month.month_element);
        assert_eq!(
            WuXing::from_branch(month.branch_id),
            month_element,
            "{} 月建五行不一致",
            month.month_branch
        );

        for (element_name, expected) in &month.states {
            let actual = element(element_name).relation_to(month_element);
            assert_eq!(
                actual,
                relation(expected),
                "{} 月 / {} 行：期望 {}，生克实现得到 {}",
                month.month_branch,
                element_name,
                expected,
                actual.name()
            );
        }
    }
}

fn relation(name: &str) -> Relation {
    match name {
        "旺" => Relation::Wang,
        "相" => Relation::Xiang,
        "休" => Relation::Xiu,
        "囚" => Relation::Qiu,
        "死" => Relation::Si,
        other => panic!("未知旺衰状态: {other}"),
    }
}
