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
    /// Capacidades exigidas pelo projeto (ENG-01-05-S03). Precisam existir no
    /// build; indisponível = erro com correção, nunca silêncio (R18).
    #[serde(default)]
    pub features: Vec<String>,
}

/// Capacidades previstas pelo doc (§3.3); nomes fora disso são erro.
pub const KNOWN_FEATURES: &[&str] = &[
    "window",
    "audio",
    "physics",
    "animation",
    "tilemap",
    "juice",
    "mcp",
    "live",
    "editor",
];

/// Capacidades compiladas nesta distribuição (fase 1: nenhuma opcional).
pub const AVAILABLE_FEATURES: &[&str] = &[];

impl EngineConfig {
    /// Valida as capacidades declaradas contra o build atual.
    pub fn validate_features(&self) -> Result<(), EngineError> {
        for f in &self.features {
            if !KNOWN_FEATURES.contains(&f.as_str()) {
                return Err(EngineError::new(
                    "E024",
                    format!("capacidade desconhecida: {f}"),
                    format!("use uma de: {}", KNOWN_FEATURES.join(", ")),
                )
                .at("engine.toml", 0));
            }
            if !AVAILABLE_FEATURES.contains(&f.as_str()) {
                return Err(EngineError::new(
                    "E025",
                    format!("capacidade indisponível nesta distribuição: {f}"),
                    format!("remova \"{f}\" de features ou rode um build com a feature {f}"),
                )
                .at("engine.toml", 0));
            }
        }
        Ok(())
    }
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
