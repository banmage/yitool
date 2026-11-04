//! 领域类型：版本、规则层级、干支、五行、卦象、步骤日志、生辰内存对象。
//!
//! 本 crate 不依赖历法、种子、存储或 UI；所有引擎共享这些类型。

pub mod bazi;
pub mod error;
pub mod ganzhi;
pub mod hexagram;
pub mod step;
pub mod version;
pub mod wuxing;

pub use bazi::BaziMaterial;
pub use error::{DomainError, Result};
pub use ganzhi::{GanZhi, BRANCHES, STEMS};
pub use hexagram::{trigram_key, Hexagram, Trigram};
pub use step::{Engine, RuleLevel, Status, StepLog, StepLogBuilder};
pub use version::VersionSet;
pub use wuxing::{Relation, WuXing};
