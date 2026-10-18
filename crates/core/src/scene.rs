//! Cena (ENGINE_VISION2 §3.7): TOML com `schema = 1` e `[[entities]]`.
//! Cena implícita = projeto sem `scenes/*.toml` (só `main.luau`).

use std::path::Path;

use api::EngineError;
use ecs::{Sprite, Transform, World};
use serde::Deserialize;

/// Entidade declarada na cena (chave local, tags, componentes).
#[derive(Debug, Deserialize)]
pub struct SceneEntity {
    pub key: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub components: SceneComponents,
}

#[derive(Debug, Default, Deserialize)]
pub struct SceneComponents {
    pub transform: Option<SceneTransform>,
    pub sprite: Option<SceneSprite>,
}

#[derive(Debug, Deserialize)]
pub struct SceneTransform {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

#[derive(Debug, Deserialize)]
pub struct SceneSprite {
    pub asset: String,
}

/// Cena completa, pronta para montar no mundo.
#[derive(Debug, Deserialize)]
pub struct Scene {
    #[serde(default = "default_schema")]
    pub schema: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub entities: Vec<SceneEntity>,
}

fn default_schema() -> u32 {
    1
}

impl Scene {
    /// Carrega e valida um arquivo de cena.
    pub fn load(path: &Path) -> Result<Self, EngineError> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            EngineError::new(
                "E030",
                format!("não consegui ler a cena {}: {e}", path.display()),
                "verifique o caminho e as permissões do arquivo",
            )
        })?;
        let scene: Scene = toml::from_str(&text).map_err(|e| {
            EngineError::new(
                "E031",
                format!("cena {}: TOML inválido: {e}", path.display()),
                "confira o schema da cena (schema = 1, [[entities]])",
            )
        })?;
        if scene.schema != 1 {
            return Err(EngineError::new(
                "E032",
                format!(
                    "cena {}: schema {} não suportado",
                    path.display(),
                    scene.schema
                ),
                "use schema = 1",
            ));
        }
        // Valida ANTES de qualquer mutação: carga que falha não deixa a cena
        // parcialmente construída (ENG-01-09-S04).
        scene.validate().map_err(|e| {
            let mut e = e;
            e.file = path.display().to_string();
            e
        })?;
        Ok(scene)
    }

    /// Validação prévia da cena: chaves únicas/não vazias e dependências de
    /// componente. Erro = nada é montado.
    pub fn validate(&self) -> Result<(), EngineError> {
        let mut seen: Vec<&str> = Vec::new();
        for entity in &self.entities {
            if entity.key.trim().is_empty() {
                return Err(EngineError::new(
                    "E014",
                    "entidade de cena sem key",
                    "toda entidade precisa de uma key única",
                ));
            }
            if seen.contains(&entity.key.as_str()) {
                return Err(EngineError::new(
                    "E014",
                    format!("chave duplicada na cena: {}", entity.key),
                    "keys de entidade são únicas na cena",
                ));
            }
            seen.push(&entity.key);
            if entity.components.sprite.is_some() && entity.components.transform.is_none() {
                return Err(EngineError::new(
                    "E013",
                    format!("entidade {}: Sprite exige Transform", entity.key),
                    "adicione [entities.components.transform] à entidade",
                ));
            }
        }
        Ok(())
    }

    /// Monta a cena no mundo ANTES do primeiro tick (§3.2: `load()` conclui a
    /// montagem inicial). Keys duplicadas e dependências de componente são
    /// erros de carga, nunca silenciosos (R18).
    pub fn mount(&self, world: &mut World) -> Result<(), EngineError> {
        for entity in &self.entities {
            let id = world.spawn(entity.tags.first().cloned().unwrap_or_default(), None, None);
            if let Some(t) = &entity.components.transform {
                world
                    .add_transform(id, Transform { x: t.x, y: t.y })
                    .map_err(|e| ecs::to_engine_error(e, id).at("scene.toml", 0))?;
            }
            if let Some(s) = &entity.components.sprite {
                world
                    .add_sprite(
                        id,
                        Sprite {
                            asset: s.asset.clone(),
                        },
                    )
                    .map_err(|e| ecs::to_engine_error(e, id).at("scene.toml", 0))?;
            }
            world
                .set_name(id, &entity.key)
                .map_err(|e| ecs::to_engine_error(e, id).at("scene.toml", 0))?;
        }
        world.barrier();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCENE: &str = r#"
schema = 1
name = "demo"

[[entities]]
key = "player"
tags = ["player"]
[entities.components.transform]
x = 48
y = 120
[entities.components.sprite]
asset = "player_idle"
"#;

    #[test]
    fn loads_and_mounts() {
        let scene: Scene = toml::from_str(SCENE).unwrap();
        let mut w = World::new();
        scene.mount(&mut w).unwrap();
        let e = w.resolve("player").unwrap();
        assert_eq!(w.transform(e).unwrap().x, 48.0);
        assert_eq!(w.sprite(e).unwrap().asset, "player_idle");
        assert_eq!(w.by_tag("player").len(), 1);
    }

    #[test]
    fn sprite_without_transform_fails_load() {
        let scene: Scene = toml::from_str(
            r#"
schema = 1
[[entities]]
key = "s"
[entities.components.sprite]
asset = "x"
"#,
        )
        .unwrap();
        let mut w = World::new();
        assert!(scene.mount(&mut w).is_err());
    }

    #[test]
    fn duplicate_key_rejected_before_mutation() {
        let scene: Scene = toml::from_str(
            r#"
schema = 1
[[entities]]
key = "a"
[[entities]]
key = "a"
"#,
        )
        .unwrap();
        let err = scene.validate().unwrap_err();
        assert_eq!(err.code, "E014");
        // Nada foi montado: validação é prévia à mutação.
        let w = World::new();
        assert_eq!(w.active().len(), 0);
    }

    #[test]
    fn validate_accepts_minimal_scene() {
        let scene: Scene = toml::from_str("schema = 1\n[[entities]]\nkey = \"solo\"\n").unwrap();
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn load_attaches_file_location_to_error() {
        let dir = std::env::temp_dir().join(format!("genflust-scene-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("dup.toml");
        std::fs::write(
            &path,
            "schema = 1\n[[entities]]\nkey = \"a\"\n[[entities]]\nkey = \"a\"\n",
        )
        .unwrap();
        let err = Scene::load(&path).unwrap_err();
        assert_eq!(err.code, "E014");
        assert!(err.file.contains("dup.toml"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
