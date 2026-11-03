use sha2::{Digest, Sha256};

/// 生辰材料：**仅内存**。
///
/// 隐私约束（章程 / SRS）：
/// - 不实现 `Serialize`，不能进入快照、数据库、日志或导出；
/// - `Debug` 只输出 `<redacted>`；
/// - `Drop` 时覆写内部字节；
/// - 生命周期应限制在单次推演调用内。
pub struct BaziMaterial {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    present: bool,
}

impl BaziMaterial {
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            present: true,
        }
    }

    /// 仅用于确认是否提供生辰，可安全持久化。
    pub fn is_present(&self) -> bool {
        self.present
    }

    /// 派生摘要仅用于当次推演的 Q/B 扰动，不得持久化。
    pub fn digest(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"BAZI-MATERIAL-v1\0");
        hasher.update(self.year.to_be_bytes());
        hasher.update([self.month, self.day, self.hour, self.minute]);
        hasher.finalize().into()
    }
}

impl std::fmt::Debug for BaziMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BaziMaterial(<redacted>)")
    }
}

impl Drop for BaziMaterial {
    fn drop(&mut self) {
        self.year = 0;
        self.month = 0;
        self.day = 0;
        self.hour = 0;
        self.minute = 0;
        self.present = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_is_redacted() {
        let b = BaziMaterial::new(1990, 1, 1, 12, 0);
        assert_eq!(format!("{b:?}"), "BaziMaterial(<redacted>)");
        assert!(b.is_present());
    }

    #[test]
    fn digest_changes_with_input() {
        let a = BaziMaterial::new(1990, 1, 1, 12, 0);
        let b = BaziMaterial::new(1990, 1, 1, 12, 1);
        assert_ne!(a.digest(), b.digest());
    }
}
