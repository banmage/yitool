//! Q/B 扰动材料（ARS 3.2，ADR-005）。
//!
//! - `Q` 来自问事 HMAC 的前 16 个十六进制字符；
//! - `B` 来自生辰材料摘要的前 16 个十六进制字符；
//! - 默认 `NONE`：两者均为 0；只有显式传入材料才启用；
//! - 材料缺失或格式非法时显式报错，不得静默回退为 0；
//! - Q/B 为 L3 自定义增强，不参与共同随机条件，也不得持久化材料摘要。

use domain::BaziMaterial;
use serde::{Deserialize, Serialize};

use crate::chain::SeedError;

/// Q/B 扰动的合法取值上限（`mod 64`）。
pub const QB_MOD_MAX: u32 = 63;

/// 一次推演使用的 Q/B 扰动值；`None` 材料对应 0。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QbMods {
    pub q_mod: u32,
    pub b_mod: u32,
}

/// 由问事 HMAC（64 位十六进制）派生 `Q_mod`。
///
/// 公式：`int(question_hmac[0:16], 16) mod 64`。
pub fn q_mod_from_question_hmac(hmac_hex: &str) -> Result<u32, SeedError> {
    if hmac_hex.len() != 64 || !hmac_hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(SeedError::InvalidInput(
            "question_hmac 必须是 64 位十六进制字符串".into(),
        ));
    }
    let prefix = u64::from_str_radix(&hmac_hex[..16], 16)
        .map_err(|_| SeedError::InvalidInput("question_hmac 前缀无法解析".into()))?;
    Ok((prefix % (QB_MOD_MAX as u64 + 1)) as u32)
}

/// 由生辰材料派生 `B_mod`。
///
/// 材料只在内存中存在；返回值为 `0..=63`，摘要本身不得持久化。
pub fn b_mod_from_bazi(material: &BaziMaterial) -> u32 {
    let digest = material.digest();
    let prefix = u64::from_be_bytes(digest[..8].try_into().expect("8 字节"));
    (prefix % (QB_MOD_MAX as u64 + 1)) as u32
}

/// 组合 Q/B 扰动：未提供材料即视为未启用，对应分量为 0。
pub fn qb_mods(
    question_hmac: Option<&str>,
    bazi: Option<&BaziMaterial>,
) -> Result<QbMods, SeedError> {
    Ok(QbMods {
        q_mod: question_hmac
            .map(q_mod_from_question_hmac)
            .transpose()?
            .unwrap_or(0),
        b_mod: bazi.map(b_mod_from_bazi).unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_means_zero() {
        assert_eq!(qb_mods(None, None).unwrap(), QbMods::default());
    }

    #[test]
    fn q_mod_uses_first_16_hex_chars() {
        let zero = "0".repeat(64);
        assert_eq!(q_mod_from_question_hmac(&zero).unwrap(), 0);

        let one = format!("0000000000000001{}", "0".repeat(48));
        assert_eq!(q_mod_from_question_hmac(&one).unwrap(), 1);

        let max = "f".repeat(64);
        assert_eq!(q_mod_from_question_hmac(&max).unwrap(), 63);
    }

    #[test]
    fn q_mod_rejects_non_hmac_input() {
        assert!(q_mod_from_question_hmac("").is_err());
        assert!(q_mod_from_question_hmac("aa").is_err());
        assert!(q_mod_from_question_hmac(&"z".repeat(64)).is_err());
        assert!(q_mod_from_question_hmac(&"问".repeat(64)).is_err());
    }

    #[test]
    fn b_mod_is_in_range_and_deterministic() {
        let material = BaziMaterial::new(1990, 1, 1, 12, 0);
        let first = b_mod_from_bazi(&material);
        assert!(first <= QB_MOD_MAX);
        assert_eq!(first, b_mod_from_bazi(&material));

        let other = BaziMaterial::new(1990, 1, 1, 12, 1);
        assert_ne!(first, b_mod_from_bazi(&other));
    }

    #[test]
    fn qb_mods_combines_independent_materials() {
        let hmac = format!("00000000000000ff{}", "0".repeat(48));
        let material = BaziMaterial::new(1990, 1, 1, 12, 0);
        let mods = qb_mods(Some(&hmac), Some(&material)).unwrap();
        assert_eq!(mods.q_mod, 63);
        assert_eq!(mods.b_mod, b_mod_from_bazi(&material));
    }
}
