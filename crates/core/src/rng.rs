//! RNG determinístico (ENGINE_VISION2 §4.3: seed fixa = mesma sequência em
//! qualquer SO). xoshiro256** — sem dependências, sem `HashMap` (ordem fixa).

pub struct Rng {
    state: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        // SplitMix64 para espalhar a seed nos quatro estados.
        let mut sm = seed;
        let mut next = || {
            sm = sm.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        };
        let state = [next(), next(), next(), next()];
        Self { state }
    }

    /// Próximo u64 da sequência.
    pub fn next_u64(&mut self) -> u64 {
        let [a, b, c, d] = self.state;
        let result = b
            .rotate_left(27)
            .wrapping_mul(5)
            .rotate_left(7)
            .wrapping_mul(9);
        let t = b << 17;
        let c = c ^ a;
        let d = d ^ b;
        let b = b ^ c;
        let a = a ^ d;
        let c = c ^ t;
        let d = d.rotate_left(45);
        self.state = [a, b, c, d];
        result
    }

    /// Inteiro em `[0, n)` — `rnd(n)` do Luau.
    pub fn below(&mut self, n: u64) -> u64 {
        if n <= 1 {
            return 0;
        }
        self.next_u64() % n
    }

    /// Estado atual, para snapshot: restaurar retoma a sequência do ponto
    /// salvo (contrato de replay, §3.9).
    pub fn state(&self) -> [u64; 4] {
        self.state
    }

    /// Reconstrói o RNG de um estado salvo (`state()`).
    pub fn from_state(state: [u64; 4]) -> Self {
        Self { state }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seed_differs() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn state_restore_resumes_sequence() {
        let mut a = Rng::new(42);
        let _ = a.next_u64();
        let saved = a.state();
        let expected = [a.next_u64(), a.next_u64(), a.next_u64()];
        let mut restored = Rng::from_state(saved);
        assert_eq!(
            [
                restored.next_u64(),
                restored.next_u64(),
                restored.next_u64()
            ],
            expected
        );
    }
}
