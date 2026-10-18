//! Componentes opcionais (dados) e acessos com validação de dependência
//! (ENGINE_VISION2 §3.7). Nenhum ponteiro interno é exposto ao script.

use crate::EntityId;
use crate::world::{ComponentError, World};

/// Posição no mundo (persistente).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
}

/// Referência a um asset por nome (persistente).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    pub asset: String,
}

/// Intenção de movimento (persistente); o system aplica no tick.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl World {
    pub fn add_transform(&mut self, entity: EntityId, t: Transform) -> Result<(), ComponentError> {
        self.slot_mut(entity)?;
        self.transforms.insert(entity.index(), t);
        Ok(())
    }

    pub fn transform(&self, entity: EntityId) -> Result<Transform, ComponentError> {
        self.slot(entity)?;
        self.transforms
            .get(entity.index())
            .copied()
            .ok_or(ComponentError::Missing("Transform"))
    }

    pub fn set_transform(&mut self, entity: EntityId, t: Transform) -> Result<(), ComponentError> {
        self.slot_mut(entity)?;
        if self.transforms.get(entity.index()).is_none() {
            return Err(ComponentError::Missing("Transform"));
        }
        self.transforms.insert(entity.index(), t);
        Ok(())
    }

    /// Leitura simples da posição (ENG-01-12-S02): nunca revela o armazenamento.
    pub fn pos(&self, entity: EntityId) -> Result<(f32, f32), ComponentError> {
        let t = self.transform(entity)?;
        Ok((t.x, t.y))
    }

    /// `Sprite` exige `Transform` (§3.7: dependência explícita).
    pub fn add_sprite(&mut self, entity: EntityId, sprite: Sprite) -> Result<(), ComponentError> {
        self.slot_mut(entity)?;
        if self.transforms.get(entity.index()).is_none() {
            return Err(ComponentError::Dependency {
                component: "Sprite",
                required: "Transform",
            });
        }
        self.sprites.insert(entity.index(), sprite);
        Ok(())
    }

    pub fn sprite(&self, entity: EntityId) -> Result<&Sprite, ComponentError> {
        self.slot(entity)?;
        self.sprites
            .get(entity.index())
            .ok_or(ComponentError::Missing("Sprite"))
    }

    /// `Velocity` exige `Transform` (intenção de movimento sobre uma posição).
    pub fn set_velocity(
        &mut self,
        entity: EntityId,
        vx: f32,
        vy: f32,
    ) -> Result<(), ComponentError> {
        self.slot_mut(entity)?;
        if self.transforms.get(entity.index()).is_none() {
            return Err(ComponentError::Dependency {
                component: "Velocity",
                required: "Transform",
            });
        }
        self.velocities
            .insert(entity.index(), Velocity { x: vx, y: vy });
        Ok(())
    }

    pub fn velocity(&self, entity: EntityId) -> Result<Velocity, ComponentError> {
        self.slot(entity)?;
        self.velocities
            .get(entity.index())
            .copied()
            .ok_or(ComponentError::Missing("Velocity"))
    }

    /// Contagens de presença por componente (contiguidade observável).
    pub fn component_counts(&self) -> (usize, usize, usize) {
        (
            self.transforms.len(),
            self.sprites.len(),
            self.velocities.len(),
        )
    }
}
