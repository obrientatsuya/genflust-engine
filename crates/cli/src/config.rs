//! `engine.toml` (ENGINE_VISION2 §2.2: projeto mínimo). Fase 1: nome,
//! paleta e cena opcionais.

use std::path::Path;

use api::EngineError;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct EngineConfig {
    pub name: String,
    /// Paleta de 32 cores em texto (`i hex` por linha). Opcional.
    pub palette: Option<String>,
    /// Cena TOML (`schema = 1`). Sem cena = cena implícita (só main.luau).
    pub scene: Option<String>,
}

impl EngineConfig {
    pub fn load(root: &Path) -> Result<Self, EngineError> {
        let path = root.join("engine.toml");
        let text = std::fs::read_to_string(&path).map_err(|e| {
            EngineError::new(
                "E020",
                format!("não consegui ler {}: {e}", path.display()),
                "rode engine new <nome> ou crie engine.toml com name = \"...\"",
            )
        })?;
        let cfg: EngineConfig = toml::from_str(&text).map_err(|e| {
            EngineError::new(
                "E021",
                format!("engine.toml inválido: {e}"),
                "confira o schema: name = \"...\", palette/scene opcionais",
            )
        })?;
        if cfg.name.trim().is_empty() {
            return Err(EngineError::new(
                "E022",
                "engine.toml sem name",
                "adicione name = \"nome-do-projeto\"",
            ));
        }
        Ok(cfg)
    }
}
