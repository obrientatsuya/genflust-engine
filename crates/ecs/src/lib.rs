//! ECS mínimo (ENGINE_VISION2 §3.7): identidade geracional, ciclo de vida
//! com barreiras e componentes opcionais em armazenamento contíguo.

mod entity;
mod world;

pub use entity::{EntityId, Lifecycle};
pub use error::to_engine_error;
pub use world::{ComponentError, Op, Sprite, Transform, World};

/// Erros do ECS como `EngineError` (R13: sempre estruturado).
pub mod error {
    use api::EngineError;

    pub fn stale(id: crate::EntityId) -> EngineError {
        EngineError::new(
            "E010",
            format!("handle obsoleto ou inexistente: {id}"),
            "use um handle devolvido por ent.new no ciclo de vida atual",
        )
    }

    pub fn not_active(id: crate::EntityId) -> EngineError {
        EngineError::new(
            "E011",
            format!("entidade não está ativa: {id}"),
            "criação pendente só ativa na barreira do próximo tick",
        )
    }

    pub fn missing_component(entity: crate::EntityId, component: &str) -> EngineError {
        EngineError::new(
            "E012",
            format!("entidade {entity} não tem componente {component}"),
            format!("adicione {component} antes de usar"),
        )
    }

    pub fn dependency(component: &str, required: &str) -> EngineError {
        EngineError::new(
            "E013",
            format!("componente {component} exige {required}"),
            format!("adicione {required} à entidade antes de {component}"),
        )
    }

    pub fn duplicate_key(key: &str) -> EngineError {
        EngineError::new(
            "E014",
            format!("chave duplicada na cena: {key}"),
            "keys de entidade são únicas na cena",
        )
    }

    /// Converte erro interno do mundo em `EngineError` estruturado (R13).
    pub fn to_engine_error(e: crate::world::ComponentError, id: crate::EntityId) -> EngineError {
        match e {
            crate::world::ComponentError::Stale => stale(id),
            crate::world::ComponentError::NotActive => not_active(id),
            crate::world::ComponentError::Missing(c) => missing_component(id, c),
            crate::world::ComponentError::Dependency {
                component,
                required,
            } => dependency(component, required),
            crate::world::ComponentError::DuplicateKey(k) => duplicate_key(&k),
        }
    }
}
