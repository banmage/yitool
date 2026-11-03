use serde::{Deserialize, Serialize};

use crate::error::{DomainError, Result};
use crate::wuxing::WuXing;

/// 先天八卦。索引按先天数 1..=8 对应 0..=7：乾兑离震巽坎艮坤。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Trigram {
    Qian,
    Dui,
    Li,
    Zhen,
    Xun,
    Kan,
    Gen,
    Kun,
}

impl Trigram {
    pub const ALL: [Trigram; 8] = [
        Trigram::Qian,
        Trigram::Dui,
        Trigram::Li,
        Trigram::Zhen,
        Trigram::Xun,
        Trigram::Kan,
        Trigram::Gen,
        Trigram::Kun,
    ];

    /// 先天数 1..=8。
    pub fn from_number(number: u8) -> Result<Self> {
        if (1..=8).contains(&number) {
            Ok(Self::ALL[(number - 1) as usize])
        } else {
            Err(DomainError::InvalidTrigram(number))
        }
    }

    pub fn number(self) -> u8 {
        Self::ALL.iter().position(|t| *t == self).unwrap() as u8 + 1
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Qian => "乾",
            Self::Dui => "兑",
            Self::Li => "离",
            Self::Zhen => "震",
            Self::Xun => "巽",
            Self::Kan => "坎",
            Self::Gen => "艮",
            Self::Kun => "坤",
        }
    }

    pub fn element(self) -> WuXing {
        match self {
            Self::Qian | Self::Dui => WuXing::Metal,
            Self::Li => WuXing::Fire,
            Self::Zhen | Self::Xun => WuXing::Wood,
            Self::Kan => WuXing::Water,
            Self::Gen | Self::Kun => WuXing::Earth,
        }
    }

    /// 三爻，自下而上；true = 阳。
    pub fn lines(self) -> [bool; 3] {
        match self {
            Self::Qian => [true, true, true],
            Self::Dui => [true, true, false],
            Self::Li => [true, false, true],
            Self::Zhen => [true, false, false],
            Self::Xun => [false, true, true],
            Self::Kan => [false, true, false],
            Self::Gen => [false, false, true],
            Self::Kun => [false, false, false],
        }
    }

    pub fn from_lines(lines: [bool; 3]) -> Self {
        Self::ALL
            .into_iter()
            .find(|t| t.lines() == lines)
            .expect("三爻只有 8 种组合")
    }
}

/// 六十四卦几何结构。上下卦均为三爻卦；六爻自下而上。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hexagram {
    pub upper: Trigram,
    pub lower: Trigram,
}

impl Hexagram {
    pub fn new(upper: Trigram, lower: Trigram) -> Self {
        Self { upper, lower }
    }

    pub fn from_lines(lines: [bool; 6]) -> Self {
        let lower = Trigram::from_lines([lines[0], lines[1], lines[2]]);
        let upper = Trigram::from_lines([lines[3], lines[4], lines[5]]);
        Self { upper, lower }
    }

    /// 六爻，自下而上；true = 阳。
    pub fn lines(self) -> [bool; 6] {
        let l = self.lower.lines();
        let u = self.upper.lines();
        [l[0], l[1], l[2], u[0], u[1], u[2]]
    }

    pub fn name(self) -> &'static str {
        HEX_NAMES[self.upper as usize][self.lower as usize]
    }

    pub fn upper_name(self) -> &'static str {
        self.upper.name()
    }

    pub fn lower_name(self) -> &'static str {
        self.lower.name()
    }

    /// 形如 `QIAN/DU,LI` 的稳定键，用于资源交叉校验。
    pub fn key(self) -> String {
        format!("{}/{}", trigram_key(self.upper), trigram_key(self.lower))
    }

    pub fn binary_bottom_up(self) -> String {
        self.lines()
            .iter()
            .map(|y| if *y { '1' } else { '0' })
            .collect()
    }
}

pub fn trigram_key(t: Trigram) -> &'static str {
    match t {
        Trigram::Qian => "QIAN",
        Trigram::Dui => "DUI",
        Trigram::Li => "LI",
        Trigram::Zhen => "ZHEN",
        Trigram::Xun => "XUN",
        Trigram::Kan => "KAN",
        Trigram::Gen => "GEN",
        Trigram::Kun => "KUN",
    }
}

/// `HEX-01` 名称表；行 = 上卦（乾兑离震巽坎艮坤），列 = 下卦。
/// 与 `resources/data/hex64.json` 由测试交叉校验。
pub const HEX_NAMES: [[&str; 8]; 8] = [
    // 上乾
    [
        "乾为天",
        "天泽履",
        "天火同人",
        "天雷无妄",
        "天风姤",
        "天水讼",
        "天山遁",
        "天地否",
    ],
    // 上兑
    [
        "泽天夬",
        "兑为泽",
        "泽火革",
        "泽雷随",
        "泽风大过",
        "泽水困",
        "泽山咸",
        "泽地萃",
    ],
    // 上离
    [
        "火天大有",
        "火泽睽",
        "离为火",
        "火雷噬嗑",
        "火风鼎",
        "火水未济",
        "火山旅",
        "火地晋",
    ],
    // 上震
    [
        "雷天大壮",
        "雷泽归妹",
        "雷火丰",
        "震为雷",
        "雷风恒",
        "雷水解",
        "雷山小过",
        "雷地豫",
    ],
    // 上巽
    [
        "风天小畜",
        "风泽中孚",
        "风火家人",
        "风雷益",
        "巽为风",
        "风水涣",
        "风山渐",
        "风地观",
    ],
    // 上坎
    [
        "水天需",
        "水泽节",
        "水火既济",
        "水雷屯",
        "水风井",
        "坎为水",
        "水山蹇",
        "水地比",
    ],
    // 上艮
    [
        "山天大畜",
        "山泽损",
        "山火贲",
        "山雷颐",
        "山风蛊",
        "山水蒙",
        "艮为山",
        "山地剥",
    ],
    // 上坤
    [
        "地天泰",
        "地泽临",
        "地火明夷",
        "地雷复",
        "地风升",
        "地水师",
        "地山谦",
        "坤为地",
    ],
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn trigram_roundtrip() {
        for n in 1..=8u8 {
            assert_eq!(Trigram::from_number(n).unwrap().number(), n);
        }
        assert!(Trigram::from_number(9).is_err());
    }

    #[test]
    fn sixty_four_unique() {
        let names: HashSet<_> = (0..8)
            .flat_map(|u| (0..8).map(move |l| HEX_NAMES[u][l]))
            .collect();
        assert_eq!(names.len(), 64);
    }

    #[test]
    fn golden_hexagrams() {
        let tong_ren = Hexagram::new(Trigram::Qian, Trigram::Li);
        assert_eq!(tong_ren.name(), "天火同人");

        // 同人互卦：上乾下巽 = 天风姤
        let lines = tong_ren.lines();
        let hu = Hexagram::from_lines([lines[1], lines[2], lines[3], lines[2], lines[3], lines[4]]);
        assert_eq!(hu.name(), "天风姤");

        // 动四爻：上巽下离 = 风火家人
        let mut changed = lines;
        changed[3] = !changed[3];
        assert_eq!(Hexagram::from_lines(changed).name(), "风火家人");

        let pi = Hexagram::new(Trigram::Qian, Trigram::Kun);
        assert_eq!(pi.name(), "天地否");
        let mut changed = pi.lines();
        changed[0] = !changed[0];
        assert_eq!(Hexagram::from_lines(changed).name(), "天雷无妄");
    }

    #[test]
    fn binary_roundtrip() {
        for u in Trigram::ALL {
            for l in Trigram::ALL {
                let h = Hexagram::new(u, l);
                assert_eq!(Hexagram::from_lines(h.lines()), h);
            }
        }
    }
}
