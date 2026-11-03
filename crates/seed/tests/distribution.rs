use seed::SeedRng;

#[test]
fn nine_color_distribution_is_uniform() {
    let mut rng = SeedRng::new([3u8; 32]);
    let mut counts = [0u32; 9];
    let n = 90_000;
    for _ in 0..n {
        counts[rng.uniform_1_to(9) as usize - 1] += 1;
    }
    let expected = n as f64 / 9.0;
    for (i, c) in counts.iter().enumerate() {
        let deviation = (*c as f64 - expected).abs() / expected;
        assert!(
            deviation < 0.05,
            "九色 {} 偏差 {:.2}%",
            i + 1,
            deviation * 100.0
        );
    }
}

#[test]
fn liuyao_distribution_matches_coin_probabilities() {
    let mut rng = SeedRng::new([5u8; 32]);
    let mut counts = [0u32; 4]; // 6,7,8,9
    let n = 120_000;
    for _ in 0..n {
        let mut sum = 0u32;
        for _ in 0..3 {
            sum += rng.uniform_below(2);
        }
        counts[sum as usize] += 1; // 0..3 -> 6..9
    }
    let expected = [
        n as f64 / 8.0,
        3.0 * n as f64 / 8.0,
        3.0 * n as f64 / 8.0,
        n as f64 / 8.0,
    ];
    for (i, c) in counts.iter().enumerate() {
        let deviation = (*c as f64 - expected[i]).abs() / expected[i];
        assert!(
            deviation < 0.05,
            "爻值 {} 偏差 {:.2}%",
            i + 6,
            deviation * 100.0
        );
    }
}
