//! Relógio de simulação: timestep fixo 60Hz, desacoplado do relógio de parede
//! (ENGINE_VISION2 §3.2: `dt` constante; headless roda sem tempo real).

use api::TICK_DT;

pub struct Clock {
    /// Frame atual (começa em 0; o primeiro tick executa o frame 0).
    pub frame: u64,
    /// Delta fixo em segundos.
    pub dt: f64,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock {
    pub fn new() -> Self {
        Self {
            frame: 0,
            dt: TICK_DT,
        }
    }

    /// Avança um tick.
    pub fn advance(&mut self) {
        self.frame += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dt_is_fixed() {
        let mut c = Clock::new();
        let d = c.dt;
        c.advance();
        c.advance();
        assert_eq!(c.frame, 2);
        assert_eq!(c.dt, d);
        assert_eq!(d, TICK_DT);
    }
}
