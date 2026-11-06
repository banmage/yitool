use cal::{CalQuery, CalTime, DayBound};
use chrono::DateTime;
use domain::{Engine, RuleLevel, StepLogBuilder};
use meihua::MeihuaInput;

fn context() -> app::AppContext {
    app::AppContext::load(app::resources_dir()).expect("load resources")
}

fn query() -> CalQuery {
    CalQuery {
        time: CalTime::Instant(DateTime::parse_from_rfc3339("2026-11-03T12:00:00+08:00").unwrap()),
        timezone: "Asia/Shanghai".into(),
        day_bound: DayBound::Zichu,
    }
}

#[test]
fn meihua_report_contains_full_process_and_links() {
    let ctx = context();
    let result = ctx
        .meihua(
            MeihuaInput::NineColor {
                color: 1,
                aux: 3,
                q_mod: 0,
                b_mod: 0,
            },
            &query(),
        )
        .unwrap();
    let evidence = ctx.evidence_index().unwrap();
    let markdown = app::render_meihua_report(
        &result,
        &ctx.versions(),
        &evidence,
        "run-report-test",
        "2026-11-03T12:00:00+08:00",
        "FIXED",
    )
    .unwrap();

    assert!(markdown.starts_with("# 梅花推演记录 · run-report-test"));
    assert!(markdown.contains("## 推理/演算过程"));
    assert!(markdown.contains("## 依据与附件"));
    assert!(markdown.contains("本报告完整包含 6 个步骤"));
    assert!(markdown.contains(&result.disclaimer));

    // 每一步的完整字段都在报告中，不依赖重新计算。
    for step in &result.steps {
        assert!(markdown.contains(&format!("`{}`", step.step_id)));
        assert!(markdown.contains(&format!("| 步骤序号 | {} |", step.step_index)));
    }
    // 表格中的竖线会被转义，但公式内容仍完整保留。
    assert!(markdown.contains("U=C mod 8"));
    assert!(markdown.contains("E_ji=克体之行"));
    assert!(markdown.contains("卦五行=E_ji"));

    // 依据、附件、规则均有可点击链接。
    assert!(markdown.contains("yitool://source/SRC-MEIHUA-001"));
    assert!(markdown.contains("yitool://resource/HEX-01"));
    assert!(markdown.contains("yitool://rule/MEIHUA-QIGUA-001"));
    assert!(markdown.contains("#evidence-source-SRC-MEIHUA-001"));
    assert!(markdown.contains("#evidence-resource-HEX-01"));
    assert!(markdown.contains("#evidence-rule-MEIHUA-QIGUA-001"));

    // 所有索引条目都进入报告，且没有断链占位。
    for entry in evidence.all() {
        assert!(
            markdown.contains(&format!("<a id=\"{}\"></a>", entry.anchor)),
            "missing anchor for {}",
            entry.id
        );
        assert!(
            markdown.contains(&entry.href),
            "missing link for {}",
            entry.id
        );
    }
    assert!(!markdown.contains("断链"));

    // 可保存为 .md，并能逐字读回（复制到剪贴板使用同一字符串）。
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("report.md");
    app::write_markdown(&path, &markdown).unwrap();
    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(saved, markdown);
}

#[test]
fn evidence_index_covers_every_manifest_entry_and_source() {
    let ctx = context();
    let evidence = ctx.evidence_index().unwrap();

    for entry in &ctx.manifest.entries {
        let found = evidence
            .get(app::EvidenceKind::Resource, &entry.id)
            .unwrap_or_else(|| panic!("missing resource link {}", entry.id));
        assert_eq!(found.path.as_deref(), Some(entry.path.as_str()));
        assert!(found.href.starts_with("yitool://resource/"));
    }

    for source in ctx.rules.all_sources() {
        let found = evidence
            .get(app::EvidenceKind::Source, &source.source_ref)
            .unwrap_or_else(|| panic!("missing source link {}", source.source_ref));
        assert!(found.href.starts_with("yitool://source/"));
    }

    for rule in ctx.rules.all_rules() {
        let found = evidence
            .get(app::EvidenceKind::Rule, &rule.rule_id)
            .unwrap_or_else(|| panic!("missing rule link {}", rule.rule_id));
        assert!(found.href.starts_with("yitool://rule/"));
    }

    for entry in evidence.all() {
        assert_eq!(
            evidence.resolve_href(&entry.href).map(|e| e.id.as_str()),
            Some(entry.id.as_str())
        );
    }
}

#[test]
fn report_rejects_unresolvable_step_references() {
    let ctx = context();
    let evidence = ctx.evidence_index().unwrap();

    let missing_rule = StepLogBuilder::new(0, "TEST-MISSING-RULE", Engine::Meihua)
        .rule_id("RULE-DOES-NOT-EXIST")
        .data_ref("G-01")
        .source("SRC-MEIHUA-001")
        .level(RuleLevel::L2)
        .formula("test")
        .build()
        .unwrap();
    assert!(matches!(
        evidence.assert_step_coverage(&[missing_rule]),
        Err(app::EvidenceError::MissingRule { .. })
    ));

    let missing_source = StepLogBuilder::new(0, "TEST-MISSING-SOURCE", Engine::Meihua)
        .rule_id("MEIHUA-QIGUA-001")
        .data_ref("G-01")
        .source("SRC-DOES-NOT-EXIST")
        .level(RuleLevel::L2)
        .formula("test")
        .build()
        .unwrap();
    assert!(matches!(
        evidence.assert_step_coverage(&[missing_source]),
        Err(app::EvidenceError::MissingSource { .. })
    ));

    let missing_resource = StepLogBuilder::new(0, "TEST-MISSING-RESOURCE", Engine::Meihua)
        .rule_id("MEIHUA-QIGUA-001")
        .data_ref("DATA-DOES-NOT-EXIST")
        .source("SRC-MEIHUA-001")
        .level(RuleLevel::L2)
        .formula("test")
        .build()
        .unwrap();
    assert!(matches!(
        evidence.assert_step_coverage(&[missing_resource]),
        Err(app::EvidenceError::MissingResource { .. })
    ));
}
