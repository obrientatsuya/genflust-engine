//! Registro de entidades: slots com geração, free list, nomes/tags e
//! componentes contíguos em SoA (Transform, Sprite, Velocity).
//!
//! Comandos estruturais (`Op`) só mudam o mundo na **barreira** do tick
//! (ENGINE_VISION2 §3.2: criar/remover nunca invalidam a iteração em curso).

use std::collections::{HashMap, VecDeque};

use crate::soa::SoA;
use crate::{EntityId, Lifecycle};

use crate::components::{Sprite, Transform, Velocity};

/// Operação estrutural pendente, aplicada na barreira do próximo tick.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// `ent.new(tag, [x], [y])`: reserva ID já no tick, ativa na barreira.
    Spawn { tag: String, x: Option<f32>, y: Option<f32> },
    /// Remove na barreira (marca `PendingRemoval` já).
    Kill { entity: EntityId },
}

/// Erros internos do mundo (convertidos em `EngineError` nos bindings).
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentError {
    Stale,
    NotActive,
    Missing(&'static str),
    Dependency { component: &'static str, required: &'static str },
    DuplicateKey(String),
}

pub(crate) struct Slot {
    pub(crate) generation: u32,
    pub(crate) lifecycle: Lifecycle,
    pub(crate) name: Option<String>,
    pub(crate) tags: Vec<String>,
}

impl Slot {
    fn vacant() -> Self {
        Self {
            generation: 0,
            lifecycle: Lifecycle::Removed,
            name: None,
            tags: Vec::new(),
        }
    }
}

/// O mundo: identidade, ciclo de vida e componentes.
pub struct World {
    pub(crate) slots: Vec<Slot>,
    free: Vec<u32>,
    /// nome → índice (chaves de cena).
    pub(crate) names: HashMap<String, u32>,
    tags: HashMap<String, Vec<u32>>,
    pending: VecDeque<Op>,
    pub(crate) transforms: SoA<Transform>,
    pub(crate) sprites: SoA<Sprite>,
    pub(crate) velocities: SoA<Velocity>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            names: HashMap::new(),
            tags: HashMap::new(),
            pending: VecDeque::new(),
            transforms: SoA::new(),
            sprites: SoA::new(),
            velocities: SoA::new(),
        }
    }

    /// Reserva um ID pendente durante o tick; ativa na barreira.
    pub fn spawn(&mut self, tag: impl Into<String>, x: Option<f32>, y: Option<f32>) -> EntityId {
        let id = self.alloc();
        let index = id.index();
        let slot = &mut self.slots[index as usize];
        slot.lifecycle = Lifecycle::Pending;
        slot.tags = vec![tag.into()];
        if x.is_some() || y.is_some() {
            self.transforms.insert(
                index,
                Transform {
                    x: x.unwrap_or(0.0),
                    y: y.unwrap_or(0.0),
                },
            );
        }
        self.pending.push_back(Op::Spawn {
            tag: self.slots[index as usize].tags[0].clone(),
            x,
            y,
        });
        id
    }

    /// Agenda remoção na próxima barreira; já marca e bloqueia comandos.
    /// Matar uma pendente cancela a criação na barreira (§3.7: a mesma fila
    /// pode montar e desmontar o ID pendente).
    pub fn kill(&mut self, entity: EntityId) -> Result<(), ComponentError> {
        let slot = self.slot_mut(entity)?;
        if !matches!(slot.lifecycle, Lifecycle::Active | Lifecycle::Pending) {
            return Err(ComponentError::NotActive);
        }
        slot.lifecycle = Lifecycle::PendingRemoval;
        self.pending.push_back(Op::Kill { entity });
        Ok(())
    }

    /// Aplica todos os comandos estruturais pendentes (barreira do tick).
    pub fn barrier(&mut self) {
        // Coleto os alvos antes de mutar (R8: sem invalidar iteração).
        let ops: Vec<Op> = self.pending.drain(..).collect();
        for op in ops {
            if let Op::Kill { entity } = op {
                let pending = matches!(
                    self.slot(entity),
                    Ok(s) if s.lifecycle == Lifecycle::PendingRemoval
                );
                if pending {
                    self.release(entity);
                }
            }
        }
        // Ativa todas as pendentes cuja reserva sobreviveu.
        let to_activate: Vec<(usize, Vec<String>)> = self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.lifecycle == Lifecycle::Pending)
            .map(|(i, s)| (i, s.tags.clone()))
            .collect();
        for (index, tags) in to_activate {
            self.slots[index].lifecycle = Lifecycle::Active;
            for tag in tags {
                self.tags.entry(tag).or_default().push(index as u32);
            }
        }
    }

    /// Nomeia/renomeia uma entidade (chaves de cena); montagem pode nomear
    /// uma pendente antes da barreira (§3.7).
    pub fn set_name(&mut self, entity: EntityId, name: &str) -> Result<(), ComponentError> {
        if matches!(self.lifecycle(entity)?, Lifecycle::Removed) {
            return Err(ComponentError::Stale);
        }
        let index = entity.index() as usize;
        if let Some(prev) = self.slots[index].name.take() {
            self.names.remove(&prev);
        }
        if self
            .names
            .insert(name.to_string(), entity.index())
            .is_some()
        {
            return Err(ComponentError::DuplicateKey(name.to_string()));
        }
        self.slots[index].name = Some(name.to_string());
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<EntityId> {
        let index = *self.names.get(name)?;
        let slot = &self.slots[index as usize];
        (slot.lifecycle == Lifecycle::Active).then(|| EntityId::new(index, slot.generation))
    }

    pub fn lifecycle(&self, entity: EntityId) -> Result<Lifecycle, ComponentError> {
        Ok(self.slot(entity)?.lifecycle)
    }

    /// Entidades ativas com a tag, em ordem estável de criação.
    pub fn by_tag(&self, tag: &str) -> Vec<EntityId> {
        self.tags
            .get(tag)
            .map(|v| {
                v.iter()
                    .filter_map(|&i| {
                        let s = &self.slots[i as usize];
                        (s.lifecycle == Lifecycle::Active).then(|| EntityId::new(i, s.generation))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Todas as entidades ativas (iteração em ordem estável de slot).
    pub fn active(&self) -> Vec<EntityId> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| s.lifecycle == Lifecycle::Active)
            .map(|(i, s)| EntityId::new(i as u32, s.generation))
            .collect()
    }

    fn alloc(&mut self) -> EntityId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.generation = slot.generation.wrapping_add(1);
            EntityId::new(index, slot.generation)
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 1,
                ..Slot::vacant()
            });
            let slots = self.slots.len();
            self.transforms.grow(slots);
            self.sprites.grow(slots);
            self.velocities.grow(slots);
            EntityId::new(index, 1)
        }
    }

    fn release(&mut self, entity: EntityId) {
        let index = entity.index();
        let slot = &mut self.slots[index as usize];
        if let Some(name) = slot.name.take() {
            self.names.remove(&name);
        }
        for tag in slot.tags.drain(..) {
            if let Some(v) = self.tags.get_mut(&tag) {
                v.retain(|&i| i != index);
            }
        }
        // Componentes saem das colunas contíguas (transitório limpo aqui).
        self.transforms.remove(index);
        self.sprites.remove(index);
        self.velocities.remove(index);
        *slot = Slot {
            generation: entity.generation(),
            ..Slot::vacant()
        };
        self.free.push(index);
    }

    pub(crate) fn slot(&self, entity: EntityId) -> Result<&Slot, ComponentError> {
        self.slots
            .get(entity.index() as usize)
            .filter(|s| s.generation == entity.generation() && s.lifecycle != Lifecycle::Removed)
            .ok_or(ComponentError::Stale)
    }

    pub(crate) fn slot_mut(&mut self, entity: EntityId) -> Result<&mut Slot, ComponentError> {
        self.slots
            .get_mut(entity.index() as usize)
            .filter(|s| s.generation == entity.generation() && s.lifecycle != Lifecycle::Removed)
            .ok_or(ComponentError::Stale)
    }
}
