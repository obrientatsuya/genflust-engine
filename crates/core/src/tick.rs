//! Orquestração do tick (ENGINE_VISION2 §3.2), ordem fixa e parte do contrato
//! de replay:
//!
//! 1. barreira (feita pelo `app` antes de `frame()`)
//! 2. input (vazio na fase 1)
//! 3. `update(dt)` + controladores
//! 4. systems Rust (nenhum habilitado na fase 1)
//! 5. `draw()` → command buffer
//! 6. raster → framebuffer → PNG | hash (feito pelo `app` depois de `frame()`)
//!
//! Os passos 3/5 são injetados pelo `app` (o core não importa backends, R4).

use std::cell::RefCell;
use std::rc::Rc;

use api::EngineError;

pub type Step = Rc<RefCell<dyn FnMut() -> Result<(), EngineError>>>;

/// Estrutura do tick: passos opcionais injetados na ordem do contrato.
pub struct Tick {
    /// `update(dt)` do Luau + controladores ativos.
    pub update: Option<Step>,
    /// `draw()` do Luau (gera comandos, nunca altera a simulação — R9).
    pub draw: Option<Step>,
}

impl Default for Tick {
    fn default() -> Self {
        Self::new()
    }
}

impl Tick {
    pub fn new() -> Self {
        Self {
            update: None,
            draw: None,
        }
    }

    pub fn set_update(&mut self, f: Step) {
        self.update = Some(f);
    }

    pub fn set_draw(&mut self, f: Step) {
        self.draw = Some(f);
    }

    /// Executa um frame completo, na ordem fixa do contrato, registrando as
    /// fases em `trace` (observabilidade §3.9: trace de um tick).
    pub fn frame(&mut self, trace: &mut Vec<&'static str>) -> Result<(), EngineError> {
        if let Some(update) = &self.update {
            trace.push("update");
            (update.borrow_mut())()?;
        }
        if let Some(draw) = &self.draw {
            trace.push("draw");
            (draw.borrow_mut())()?;
        }
        Ok(())
    }
}
