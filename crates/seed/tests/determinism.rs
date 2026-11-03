use seed::{generate, Mode, SeedInput, SeedScheme};

fn input(question: Option<String>) -> SeedInput {
    SeedInput {
        profile_id: "family-a".into(),
        run_time: "2026-11-03T12:00:00+08:00".into(),
        timezone: "Asia/Shanghai".into(),
        day_bound: "ZICHU".into(),
        mode: Mode::Fixed,
        scheme: SeedScheme::NineColor,
        color: Some(1),
        aux: Some(3),
        traditional: None,
        question_hmac: question,
        bazi_present: false,
        base_seed: None,
    }
}

#[test]
fn hundred_runs_are_identical() {
    let first = generate(&input(None)).unwrap();
    for _ in 0..100 {
        let next = generate(&input(None)).unwrap();
        assert_eq!(first.base_seed, next.base_seed);
        assert_eq!(first.draw_seed, next.draw_seed);
        assert_eq!(first.snapshot_digest, next.snapshot_digest);
        assert_eq!(first.run_seed, next.run_seed);
        assert_eq!(first.draw_result, next.draw_result);
    }
}

#[test]
fn question_does_not_change_common_random_conditions() {
    let a = generate(&input(Some("11".repeat(32)))).unwrap();
    let b = generate(&input(Some("22".repeat(32)))).unwrap();
    assert_eq!(a.base_seed, b.base_seed);
    assert_eq!(a.draw_seed, b.draw_seed);
    assert_eq!(a.snapshot_digest, b.snapshot_digest);
    assert_eq!(a.run_seed, b.run_seed);
    assert_ne!(
        a.input_snapshot.question_hmac,
        b.input_snapshot.question_hmac
    );
}

#[test]
fn traditional_mode_uses_no_draw() {
    let mut i = input(None);
    i.scheme = SeedScheme::TraditionalNumber;
    i.color = None;
    i.aux = None;
    i.traditional = Some([9, 16]);
    let chain = generate(&i).unwrap();
    assert_eq!(chain.draw_result.traditional_numbers, Some([9, 16]));
    assert!(chain.draw_result.color_id.is_none());
}

#[test]
fn snapshot_contains_no_bazi_material() {
    let chain = generate(&input(None)).unwrap();
    let json = serde_json::to_string(&chain.input_snapshot).unwrap();
    assert!(json.contains("\"bazi_present\":false"));
    assert!(!json.contains("1990-01-01"));
    assert!(!json.contains("\"year\":"));
}
