use domain::BaziMaterial;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::rng::SeedRng;

pub const SEED_BASE_RULE: &str = "SEED-BASE-001";
pub const SEED_DRAW_RULE: &str = "SEED-DRAW-001";
pub const SEED_SNAPSHOT_RULE: &str = "SEED-SNAPSHOT-001";
pub const SEED_RUN_RULE: &str = "SEED-RUN-001";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SeedError {
    #[error("固定模式缺少输入: {0}")]
    MissingInput(String),
    #[error("非法输入: {0}")]
    InvalidInput(String),
    #[error("OS 随机源不可用: {0}")]
    EntropyUnavailable(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Mode {
    Fixed,
    Cast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeedScheme {
    NineColor,
    TraditionalNumber,
}

#[derive(Debug, Clone)]
pub struct SeedInput {
    pub profile_id: String,
    pub run_time: String,
    pub timezone: String,
    pub day_bound: String,
    pub mode: Mode,
    pub scheme: SeedScheme,
    pub color: Option<u8>,
    pub aux: Option<u8>,
    pub traditional: Option<[u32; 2]>,
    /// 仅 HMAC；原文不得进入本结构。
    pub question_hmac: Option<String>,
    pub bazi_present: bool,
    /// 显式提供时用于重放；固定模式忽略。
    pub base_seed: Option<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawResult {
    pub color_id: Option<u8>,
    pub aux_type: Option<String>,
    pub aux_id: Option<u8>,
    pub traditional_numbers: Option<[u32; 2]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputSnapshot {
    pub color_scheme: SeedScheme,
    pub color_id: Option<u8>,
    pub aux_type: String,
    pub aux_id: Option<u8>,
    pub traditional_numbers: Option<[u32; 2]>,
    pub question_hmac: Option<String>,
    pub bazi_present: bool,
    pub liuyao_input_type: String,
    pub random_snapshot_digest: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SeedChain {
    pub base_seed: [u8; 32],
    pub draw_seed: [u8; 32],
    pub draw_result: DrawResult,
    pub input_snapshot: InputSnapshot,
    pub snapshot_digest: [u8; 32],
    pub run_seed: [u8; 32],
}

impl SeedChain {
    pub fn base_seed_hex(&self) -> String {
        hex::encode(self.base_seed)
    }
    pub fn draw_seed_hex(&self) -> String {
        hex::encode(self.draw_seed)
    }
    pub fn snapshot_digest_hex(&self) -> String {
        hex::encode(self.snapshot_digest)
    }
    pub fn run_seed_hex(&self) -> String {
        hex::encode(self.run_seed)
    }

    /// 子种子：同一推演内按引擎/用途分离，避免串用。
    pub fn subseed(&self, label: &str) -> [u8; 32] {
        derive_subseed(self.run_seed, label)
    }
}

pub fn derive_subseed(run_seed: [u8; 32], label: &str) -> [u8; 32] {
    hash_parts(&[b"SEED-SUB-v1\0", &run_seed, label.as_bytes()])
}

pub fn generate(input: &SeedInput) -> Result<SeedChain, SeedError> {
    validate(input)?;

    let base_seed = match (input.mode, input.base_seed) {
        (Mode::Cast, Some(seed)) => seed,
        (Mode::Fixed, Some(seed)) => seed,
        (Mode::Fixed, None) => hash_parts(&[
            b"SEED-BASE-001\0",
            input.profile_id.as_bytes(),
            input.run_time.as_bytes(),
            input.timezone.as_bytes(),
            input.day_bound.as_bytes(),
            format!("{:?}", input.mode).as_bytes(),
            format!("{:?}", input.scheme).as_bytes(),
        ]),
        (Mode::Cast, None) => {
            let mut seed = [0u8; 32];
            getrandom::fill(&mut seed).map_err(|e| SeedError::EntropyUnavailable(e.to_string()))?;
            seed
        }
    };

    let draw_seed = hash_parts(&[b"SEED-DRAW-001\0", &base_seed]);
    let mut draw_rng = SeedRng::new(draw_seed);

    let draw_result = match input.scheme {
        SeedScheme::NineColor => DrawResult {
            color_id: Some(match input.color {
                Some(c) => c,
                None => draw_rng.uniform_1_to(9) as u8,
            }),
            aux_type: Some("ZODIAC".into()),
            aux_id: Some(match input.aux {
                Some(a) => a,
                None => draw_rng.uniform_1_to(12) as u8,
            }),
            traditional_numbers: None,
        },
        SeedScheme::TraditionalNumber => DrawResult {
            color_id: None,
            aux_type: None,
            aux_id: None,
            traditional_numbers: input.traditional,
        },
    };

    let input_snapshot = InputSnapshot {
        color_scheme: input.scheme,
        color_id: draw_result.color_id,
        aux_type: draw_result.aux_type.clone().unwrap_or_default(),
        aux_id: draw_result.aux_id,
        traditional_numbers: draw_result.traditional_numbers,
        question_hmac: input.question_hmac.clone(),
        bazi_present: input.bazi_present,
        liuyao_input_type: match input.mode {
            Mode::Fixed => "FIXED_DERIVED".into(),
            Mode::Cast => "COIN_CAST".into(),
        },
        random_snapshot_digest: None,
    };

    // 共同随机条件排除问事 HMAC 与生辰布尔，保证 A/B 对照不被问事文本改变。
    let mut common = input_snapshot.clone();
    common.question_hmac = None;
    common.bazi_present = false;
    let snapshot_digest = hash_parts(&[
        b"SEED-SNAPSHOT-001\0",
        &serde_json::to_vec(&common).map_err(|e| SeedError::InvalidInput(e.to_string()))?,
    ]);

    let run_seed = hash_parts(&[
        b"SEED-RUN-001\0",
        &base_seed,
        &snapshot_digest,
        format!("{:?}", input.mode).as_bytes(),
    ]);

    Ok(SeedChain {
        base_seed,
        draw_seed,
        draw_result,
        input_snapshot,
        snapshot_digest,
        run_seed,
    })
}

/// 便捷入口：由生辰材料生成 Q/B 扰动的 B 分量（不持久化摘要）。
pub fn bazi_b_mod(material: &BaziMaterial) -> u64 {
    let digest = material.digest();
    let mut v = u64::from_be_bytes(digest[..8].try_into().expect("8 字节"));
    v %= 64;
    v
}

fn validate(input: &SeedInput) -> Result<(), SeedError> {
    if input.profile_id.trim().is_empty() {
        return Err(SeedError::MissingInput("profile_id".into()));
    }
    if input.run_time.trim().is_empty() {
        return Err(SeedError::MissingInput("run_time".into()));
    }
    match input.scheme {
        SeedScheme::NineColor => {
            if let Some(c) = input.color {
                if !(1..=9).contains(&c) {
                    return Err(SeedError::InvalidInput(format!("color_id 越界: {c}")));
                }
            }
            if let Some(a) = input.aux {
                if !(1..=12).contains(&a) {
                    return Err(SeedError::InvalidInput(format!("aux_id 越界: {a}")));
                }
            }
        }
        SeedScheme::TraditionalNumber => {
            let [u, d] = input
                .traditional
                .ok_or_else(|| SeedError::MissingInput("traditional_numbers".into()))?;
            if u == 0 || d == 0 {
                return Err(SeedError::InvalidInput("起卦数字必须为正整数".into()));
            }
        }
    }
    Ok(())
}

fn hash_parts(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> SeedInput {
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
            question_hmac: None,
            bazi_present: false,
            base_seed: None,
        }
    }

    #[test]
    fn fixed_mode_is_deterministic() {
        let a = generate(&base_input()).unwrap();
        let b = generate(&base_input()).unwrap();
        assert_eq!(a.run_seed, b.run_seed);
        assert_eq!(a.snapshot_digest, b.snapshot_digest);
    }

    #[test]
    fn question_hmac_does_not_change_common_random() {
        let mut a = base_input();
        a.question_hmac = Some("aa".repeat(32));
        let mut b = base_input();
        b.question_hmac = Some("bb".repeat(32));
        let a = generate(&a).unwrap();
        let b = generate(&b).unwrap();
        assert_eq!(a.snapshot_digest, b.snapshot_digest);
        assert_eq!(a.run_seed, b.run_seed);
        assert_eq!(a.draw_seed, b.draw_seed);
    }

    #[test]
    fn cast_replay_from_snapshot_seed() {
        let mut a = base_input();
        a.mode = Mode::Cast;
        a.base_seed = Some([42u8; 32]);
        a.color = None;
        a.aux = None;
        let first = generate(&a).unwrap();
        let second = generate(&a).unwrap();
        assert_eq!(first.base_seed, second.base_seed);
        assert_eq!(first.run_seed, second.run_seed);
        assert_eq!(first.draw_result, second.draw_result);
    }

    #[test]
    fn invalid_color_rejected() {
        let mut a = base_input();
        a.color = Some(10);
        assert!(generate(&a).is_err());
    }

    #[test]
    fn color_changes_snapshot_not_base() {
        let a = generate(&base_input()).unwrap();
        let mut other = base_input();
        other.color = Some(2);
        let b = generate(&other).unwrap();
        assert_eq!(a.base_seed, b.base_seed);
        assert_eq!(a.draw_seed, b.draw_seed);
        assert_ne!(a.snapshot_digest, b.snapshot_digest);
        assert_ne!(a.run_seed, b.run_seed);
    }
}
