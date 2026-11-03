use serde::{Deserialize, Serialize};

use crate::error::{DomainError, Result};

pub const STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
pub const BRANCHES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// 六十甲子中的一项。`index` 为 0..60，`stem` 0..10，`branch` 0..12。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GanZhi {
    pub stem: u8,
    pub branch: u8,
}

impl GanZhi {
    pub fn new(stem: u8, branch: u8) -> Result<Self> {
        if stem >= 10 {
            return Err(DomainError::InvalidStem(stem));
        }
        if branch >= 12 {
            return Err(DomainError::InvalidBranch(branch));
        }
        Ok(Self { stem, branch })
    }

    /// 由六十甲子序号构造（0 = 甲子）。
    pub fn from_cycle_index(index: u8) -> Self {
        let i = index % 60;
        Self {
            stem: i % 10,
            branch: i % 12,
        }
    }

    pub fn cycle_index(self) -> u8 {
        // 解同余：i ≡ stem (mod 10), i ≡ branch (mod 12)。
        for i in 0..60u8 {
            if i % 10 == self.stem && i % 12 == self.branch {
                return i;
            }
        }
        // 非法组合（如甲丑）不会出现在 60 甲子中；回退到天干序号。
        self.stem
    }

    pub fn stem_name(self) -> &'static str {
        STEMS[self.stem as usize]
    }

    pub fn branch_name(self) -> &'static str {
        BRANCHES[self.branch as usize]
    }

    pub fn name(self) -> String {
        format!("{}{}", self.stem_name(), self.branch_name())
    }

    pub fn stem_element(self) -> crate::wuxing::WuXing {
        crate::wuxing::WuXing::from_stem(self.stem)
    }

    pub fn branch_element(self) -> crate::wuxing::WuXing {
        crate::wuxing::WuXing::from_branch(self.branch)
    }

    /// 旬首（甲某）与旬空两地支。
    pub fn xun(self) -> (GanZhi, [u8; 2]) {
        let index = self.cycle_index() as i32;
        let start = index - (index % 10);
        let head = GanZhi {
            stem: 0,
            branch: (start as u8) % 12,
        };
        let kong = [((start + 10) as u8) % 12, ((start + 11) as u8) % 12];
        (head, kong)
    }
}

impl std::fmt::Display for GanZhi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.stem_name(), self.branch_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_roundtrip() {
        for i in 0..60u8 {
            let gz = GanZhi::from_cycle_index(i);
            assert_eq!(gz.cycle_index(), i);
        }
    }

    #[test]
    fn known_xun() {
        let xin_si = GanZhi::new(7, 5).unwrap();
        assert_eq!(xin_si.name(), "辛巳");
        let (head, kong) = xin_si.xun();
        assert_eq!(head.name(), "甲戌");
        assert_eq!(
            [BRANCHES[kong[0] as usize], BRANCHES[kong[1] as usize]],
            ["申", "酉"]
        );
    }
}
