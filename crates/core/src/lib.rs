//! Núcleo (ENGINE_VISION2 §3.2/§3.7): relógio fixo, RNG semeado, cena
//! implícita + cena TOML e barreiras do tick.
//!
//! O núcleo não importa backends nem ferramentas (R4): a orquestração do tick
//! recebe os passos por injeção (o `app` conecta Luau/render).

mod clock;
mod rng;
mod scene;
mod tick;

pub use clock::Clock;
pub use rng::Rng;
pub use scene::{Scene, SceneComponents, SceneEntity, SceneSprite, SceneTransform};
pub use tick::{Step, Tick};

pub const DEFAULT_SEED: u64 = 42;
