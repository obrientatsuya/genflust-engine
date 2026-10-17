//! Identidade geracional (R10): índice + geração. Slot reutilizado nunca
//! revalida um handle antigo.

use std::fmt;

/// ID opaco: índice no registro + geração do slot (ENGINE_VISION2 §3.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    index: u32,
    generation: u32,
}

impl EntityId {
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Handle cru para a fronteira Luau: `(geração << 32) | índice`.
    pub fn raw(self) -> u64 {
        ((self.generation as u64) << 32) | self.index as u64
    }

    /// Reconstrói um handle da fronteira Luau.
    pub fn from_raw(raw: u64) -> Self {
        Self {
            index: raw as u32,
            generation: (raw >> 32) as u32,
        }
    }

    pub fn index(self) -> u32 {
        self.index
    }

    pub fn generation(self) -> u32 {
        self.generation
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}.{}", self.index, self.generation)
    }
}

/// Estado de ciclo de vida (ENGINE_VISION2 §3.7):
/// pendente → ativa → remoção pendente → removida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    /// Reservado no tick, ativa na barreira.
    Pending,
    Active,
    /// Marcada para remoção; limpa na próxima barreira.
    PendingRemoval,
    /// Slot livre para reuso (com geração incrementada).
    Removed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_differ_by_generation() {
        let a = EntityId::new(7, 1);
        let b = EntityId::new(7, 2);
        assert_ne!(a, b);
        assert_eq!(a.index(), b.index());
    }
}
