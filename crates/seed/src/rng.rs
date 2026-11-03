use sha2::{Digest, Sha256};

/// 基于 SHA-256 计数器的确定性字节流。固定模式不触碰 OS 熵。
#[derive(Debug, Clone)]
pub struct SeedRng {
    state: [u8; 32],
    counter: u64,
    buffer: [u8; 32],
    used: usize,
}

impl SeedRng {
    pub fn new(seed: [u8; 32]) -> Self {
        Self {
            state: seed,
            counter: 0,
            buffer: [0u8; 32],
            used: 32,
        }
    }

    fn refill(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(b"SEED-RNG-v1\0");
        hasher.update(self.state);
        hasher.update(self.counter.to_be_bytes());
        self.buffer = hasher.finalize().into();
        self.counter += 1;
        self.used = 0;
    }

    pub fn next_u8(&mut self) -> u8 {
        if self.used >= self.buffer.len() {
            self.refill();
        }
        let b = self.buffer[self.used];
        self.used += 1;
        b
    }

    pub fn next_u32(&mut self) -> u32 {
        u32::from_be_bytes([
            self.next_u8(),
            self.next_u8(),
            self.next_u8(),
            self.next_u8(),
        ])
    }

    /// 无偏采样 1..=n（拒绝采样）。
    pub fn uniform_below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "n 必须大于 0");
        let zone = u32::MAX - (u32::MAX % n);
        loop {
            let v = self.next_u32();
            if v < zone {
                return v % n;
            }
        }
    }

    /// 无偏采样 1..=n。
    pub fn uniform_1_to(&mut self, n: u32) -> u32 {
        self.uniform_below(n) + 1
    }
}

/// 使用调用方提供的字节流做无偏 1..=n 采样。
pub fn uniform_below(n: u32, mut next: impl FnMut() -> u8) -> u32 {
    assert!(n > 0, "n 必须大于 0");
    let zone = u32::MAX - (u32::MAX % n);
    loop {
        let v = u32::from_be_bytes([next(), next(), next(), next()]);
        if v < zone {
            return v % n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = SeedRng::new([1u8; 32]);
        let mut b = SeedRng::new([1u8; 32]);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn range_bounds() {
        let mut rng = SeedRng::new([9u8; 32]);
        for _ in 0..10_000 {
            let v = rng.uniform_1_to(9);
            assert!((1..=9).contains(&v));
        }
    }
}
