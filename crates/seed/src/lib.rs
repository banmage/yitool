//! 种子链、HMAC 与无偏随机。
//!
//! 固定模式不调用 OS 熵；摇卦模式的基础种子来自 OS CSPRNG，并写入随机快照。
//! 问事 HMAC 不参与共同随机条件，保证 A/B 统一对照。

pub mod chain;
pub mod hmac;
pub mod rng;

pub use chain::{
    derive_subseed, generate, DrawResult, InputSnapshot, Mode, SeedChain, SeedError, SeedInput,
    SeedScheme,
};
pub use hmac::{question_hmac, HmacError, KeyProvider, MockKeyProvider};
pub use rng::{uniform_below, SeedRng};
