//! Contratos e tipos públicos da engine (AGENTS.md §2, ENGINE_VISION2 §3.3).
//!
//! Zero implementação de capacidade: só traits e tipos compartilhados.

mod command;
mod error;
mod formatter;

pub use command::Command;
pub use error::EngineError;
pub use formatter::{Formatter, Value};

/// Resolução interna fixa (ENGINE_VISION2 §2.2): 320×180, paleta de 32 cores.
pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 180;
pub const PALETTE_SIZE: usize = 32;

/// Timestep fixo: 60Hz (ENGINE_VISION2 §3.2).
pub const TICK_DT: f64 = 1.0 / 60.0;
