//! VM Luau (ENGINE_VISION2 §2.5/§4.9): sandbox, `--!strict`, callbacks
//! `load()/update(dt)/draw()` e orçamento por interrupção.
//!
//! Estado compartilhado via `Arc<Mutex<...>>`: o `app` é o dono dos dados e
//! o Luau só os enxerga através dos bindings (§3.6-A: o agente nunca vê
//! buffers). Sem `io`/`os`/rede (§4.9).

mod bindings;

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use api::{Command, EngineError, TICK_DT};
use assets::Registry;
use core::Rng;
use ecs::World;
use mlua::{Lua, Value, VmState};

/// Orçamento por frame: número máximo de interrupções da VM por callback
/// (§4.9: script lento não trava o loop; loop infinito interrompe aqui).
const INTERRUPT_BUDGET: u64 = 10_000_000;

pub struct ScriptHost {
    lua: Lua,
    /// Último erro estruturado emitido por um binding (R13).
    last_error: Arc<Mutex<Option<EngineError>>>,
    /// Contador de interrupções do callback em execução.
    budget: Arc<AtomicU64>,
    /// Guarda de R9: draw() não pode alterar a simulação (atômico: os
    /// bindings leem durante o callback, sem lock reentrante).
    in_draw: Arc<AtomicBool>,
}

impl ScriptHost {
    /// Carrega `main.luau` em modo strict e instala os globals (`gfx`, `ent`,
    /// `rnd`) uma única vez. Erro de sintaxe = E050.
    pub fn new(
        main: &Path,
        world: Arc<Mutex<World>>,
        rng: Arc<Mutex<Rng>>,
        commands: Arc<Mutex<Vec<Command>>>,
        registry: Arc<Mutex<Registry>>,
        in_draw: Arc<AtomicBool>,
    ) -> Result<Self, EngineError> {
        let lua = Lua::new();
        lua.sandbox(true)
            .map_err(|e| EngineError::new("E050", format!("sandbox: {e}"), "erro interno"))?;

        let last_error = Arc::new(Mutex::new(None));
        bindings::install(
            &lua,
            world,
            rng,
            commands,
            registry,
            last_error.clone(),
            in_draw.clone(),
        )
        .map_err(|e| script_error("E051", "bindings", &e))?;

        let source = std::fs::read_to_string(main).map_err(|e| {
            EngineError::new(
                "E050",
                format!("não consegui ler {}: {e}", main.display()),
                "o projeto precisa de um main.luau",
            )
        })?;
        let strict = format!("--!strict\n{source}");
        lua.load(&strict)
            .set_name("main.luau")
            .exec()
            .map_err(|e| script_error("E050", "main.luau", &e))?;

        let budget = Arc::new(AtomicU64::new(0));
        {
            let budget = budget.clone();
            lua.set_interrupt(move |_| {
                if budget.fetch_add(1, Ordering::Relaxed) > INTERRUPT_BUDGET {
                    return Err(mlua::Error::external(
                        "script excedeu o orçamento de instruções por frame",
                    ));
                }
                Ok(VmState::Continue)
            });
        }

        Ok(Self {
            lua,
            last_error,
            budget,
            in_draw,
        })
    }

    /// `load()`: montagem inicial antes do primeiro tick (§3.2).
    pub fn call_load(&self) -> Result<(), EngineError> {
        self.call("load", false)
    }

    /// `update(dt)`.
    pub fn call_update(&self) -> Result<(), EngineError> {
        self.call("update", true)
    }

    /// `draw()`: só gera comandos visuais; ent.* que altera o mundo é
    /// rejeitado pela guarda R9.
    pub fn call_draw(&self) -> Result<(), EngineError> {
        self.in_draw.store(true, Ordering::Relaxed);
        let result = self.call("draw", false);
        self.in_draw.store(false, Ordering::Relaxed);
        result
    }

    fn call(&self, name: &str, pass_dt: bool) -> Result<(), EngineError> {
        self.budget.store(0, Ordering::Relaxed);
        let result = (|| -> mlua::Result<()> {
            let cb: Value = self.lua.globals().get(name)?;
            if cb.is_nil() {
                return Ok(());
            }
            let func = cb.as_function().ok_or(mlua::Error::RuntimeError(format!(
                "{name} deve ser uma função"
            )))?;
            if pass_dt {
                func.call::<()>(TICK_DT)?;
            } else {
                func.call::<()>(())?;
            }
            Ok(())
        })();
        result.map_err(|e| self.take_error("main.luau", e))
    }

    /// Converte erro de VM em EngineError; erro de binding mantém o código
    /// estruturado emitido (R13).
    fn take_error(&self, file: &str, e: mlua::Error) -> EngineError {
        if let Some(err) = self.last_error.lock().expect("poisoned").take() {
            return err.at(file, 0);
        }
        let msg = format!("{e}");
        let first = msg.lines().next().unwrap_or(&msg);
        EngineError::new(
            "E050",
            format!("erro de script: {first}"),
            "corrija o erro apontado no main.luau e rode engine run de novo",
        )
        .at(file, 0)
    }
}

fn script_error(code: &'static str, file: &str, e: &mlua::Error) -> EngineError {
    EngineError::new(
        code,
        format!("{e}"),
        "corrija o erro apontado e rode engine check",
    )
    .at(file, 0)
}
