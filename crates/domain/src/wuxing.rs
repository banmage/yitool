use serde::{Deserialize, Serialize};

/// 五行。生成：木→火→土→金→水→木；克制：木→土→水→火→金→木。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WuXing {
    Wood,
    Fire,
    Earth,
    Metal,
    Water,
}

impl WuXing {
    pub fn name(self) -> &'static str {
        match self {
            Self::Wood => "木",
            Self::Fire => "火",
            Self::Earth => "土",
            Self::Metal => "金",
            Self::Water => "水",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "木" => Some(Self::Wood),
            "火" => Some(Self::Fire),
            "土" => Some(Self::Earth),
            "金" => Some(Self::Metal),
            "水" => Some(Self::Water),
            _ => None,
        }
    }

    pub fn from_stem(stem: u8) -> Self {
        match stem % 10 {
            0 | 1 => Self::Wood,
            2 | 3 => Self::Fire,
            4 | 5 => Self::Earth,
            6 | 7 => Self::Metal,
            _ => Self::Water,
        }
    }

    pub fn from_branch(branch: u8) -> Self {
        match branch % 12 {
            2 | 3 => Self::Wood,
            5 | 6 => Self::Fire,
            8 | 9 => Self::Metal,
            11 | 0 => Self::Water,
            _ => Self::Earth,
        }
    }

    pub fn generates(self) -> Self {
        match self {
            Self::Wood => Self::Fire,
            Self::Fire => Self::Earth,
            Self::Earth => Self::Metal,
            Self::Metal => Self::Water,
            Self::Water => Self::Wood,
        }
    }

    pub fn controls(self) -> Self {
        match self {
            Self::Wood => Self::Earth,
            Self::Earth => Self::Water,
            Self::Water => Self::Fire,
            Self::Fire => Self::Metal,
            Self::Metal => Self::Wood,
        }
    }

    /// 以月建五行衡量某卦五行：同我旺、生我相、我生休、我克囚、克我死。
    pub fn relation_to(self, month: Self) -> Relation {
        if self == month {
            Relation::Wang
        } else if month.generates() == self {
            Relation::Xiang
        } else if self.generates() == month {
            Relation::Xiu
        } else if self.controls() == month {
            Relation::Qiu
        } else {
            Relation::Si
        }
    }
}

impl std::fmt::Display for WuXing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Relation {
    #[serde(rename = "旺")]
    Wang,
    #[serde(rename = "相")]
    Xiang,
    #[serde(rename = "休")]
    Xiu,
    #[serde(rename = "囚")]
    Qiu,
    #[serde(rename = "死")]
    Si,
}

impl Relation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Wang => "旺",
            Self::Xiang => "相",
            Self::Xiu => "休",
            Self::Qiu => "囚",
            Self::Si => "死",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relations() {
        assert_eq!(WuXing::Fire.relation_to(WuXing::Fire), Relation::Wang);
        assert_eq!(WuXing::Fire.relation_to(WuXing::Wood), Relation::Xiang);
        assert_eq!(WuXing::Fire.relation_to(WuXing::Earth), Relation::Xiu);
        assert_eq!(WuXing::Fire.relation_to(WuXing::Metal), Relation::Qiu);
        assert_eq!(WuXing::Fire.relation_to(WuXing::Water), Relation::Si);
    }
}
