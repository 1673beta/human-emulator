//! 依存クレートなしの決定論的な擬似乱数生成器 (xorshift64*)。
//!
//! 同じシードからは必ず同じ一日が再現される。人間の「気まぐれ」は
//! 再現可能でなければデバッグできない。

pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // 状態 0 は xorshift の不動点なので避ける。
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// 0.0 以上 1.0 未満。
    pub fn ratio(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + self.ratio() * (hi - lo)
    }

    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        debug_assert!(!xs.is_empty(), "空スライスからは選べない");
        &xs[(self.next_u64() % xs.len() as u64) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 同じシードは同じ列を生む() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn 異なるシードは異なる列を生む() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn ratio_は単位区間に収まる() {
        let mut r = Rng::new(7);
        for _ in 0..10_000 {
            let v = r.ratio();
            assert!((0.0..1.0).contains(&v), "{v} が範囲外");
        }
    }

    #[test]
    fn シードゼロでも縮退しない() {
        let mut r = Rng::new(0);
        let first = r.next_u64();
        assert_ne!(first, r.next_u64());
    }
}
