//! Serialização canônica do estado persistente (ENG-01-07-S04).
//!
//! Dados persistentes (Transform, Sprite, Velocity) serializam em ordem
//! canônica de slot — independente da ordem de consulta. Transitórios
//! (filas estruturais, remoção pendente) NÃO entram no snapshot.
//! Nada de ponteiro/ID de runtime: o snapshot usa as chaves locais.

use serde::{Deserialize, Serialize};

use crate::Lifecycle;
use crate::world::World;

/// Estado persistente de uma entidade, em campos serializáveis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub key: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transform: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sprite: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub velocity: Option<[f32; 2]>,
}

/// Snapshot completo: entidades persistentes em ordem canônica de slot.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub entities: Vec<EntitySnapshot>,
}

impl Snapshot {
    /// Representação TOML determinística (mesma ordem sempre).
    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_else(|_| String::new())
    }

    /// Reconstrói de TOML (roundtrip).
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }
}

impl World {
    /// Snapshot do estado persistente em ordem canônica: por `key` (lexicográfica;
    /// sem key ficam na frente, em ordem de slot). Transitórios (fila
    /// estrutural, remoção pendente) ficam fora. Ordem independente da ordem
    /// de criação/consulta (ENG-01-07-S04).
    pub fn snapshot(&self) -> Snapshot {
        let mut entities = Vec::new();
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.lifecycle != Lifecycle::Active {
                continue;
            }
            let idx = i as u32;
            entities.push(EntitySnapshot {
                key: slot.name.clone().unwrap_or_default(),
                tags: slot.tags.clone(),
                transform: self.transforms.get(idx).map(|t| [t.x, t.y]),
                sprite: self.sprites.get(idx).map(|s| s.asset.clone()),
                velocity: self.velocities.get(idx).map(|v| [v.x, v.y]),
            });
        }
        // sort estável por key: empates (sem key) preservam a ordem de slot.
        entities.sort_by(|a, b| a.key.cmp(&b.key));
        Snapshot { entities }
    }
}
