use seed::{generate, Mode, SeedInput, SeedScheme};

fn base() -> SeedInput {
    SeedInput {
        profile_id: "family-a".into(),
        run_time: "2026-11-03T12:00:00+08:00".into(),
        timezone: "Asia/Shanghai".into(),
        day_bound: "ZICHU".into(),
        mode: Mode::Cast,
        scheme: SeedScheme::NineColor,
        color: None,
        aux: None,
        traditional: None,
        question_hmac: None,
        bazi_present: false,
        base_seed: None,
    }
}

#[test]
fn cast_snapshot_replays_exact_result() {
    let first = generate(&base()).unwrap();
    let snapshot = serde_json::to_string(&first.input_snapshot).unwrap();

    let mut replay = base();
    replay.base_seed = Some(first.base_seed);
    let second = generate(&replay).unwrap();

    assert_eq!(first.base_seed, second.base_seed);
    assert_eq!(first.draw_result, second.draw_result);
    assert_eq!(first.run_seed, second.run_seed);
    assert!(snapshot.contains("bazi_present"));
    assert!(!snapshot.contains("question_text"));
}

#[test]
fn exported_snapshot_digest_is_stable() {
    let chain = generate(&base()).unwrap();
    let a = chain.snapshot_digest_hex();
    let b = chain.snapshot_digest_hex();
    assert_eq!(a, b);
    assert_eq!(a.len(), 64);
}
