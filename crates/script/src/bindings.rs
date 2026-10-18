//! Bindings Luau (ENGINE_VISION2 §5, fase 1): `gfx.*`, `ent.*`, `rnd`.
//!
//! Instalados uma vez. Toda validação/conversão de erro passa por helpers —
//! nenhuma cópia manual de lock/convert/map_error. Erros carregam `code`/`msg`
//! (R13) e a guarda R9 bloqueia mutação de mundo dentro de `draw()`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use api::{Command, EngineError};
use assets::Registry;
use core::Rng;
use ecs::{ComponentError, EntityId, World};
use mlua::{Lua, Result};

/// Estado compartilhado; clonável para os closures 'static do mlua.
#[derive(Clone)]
struct Shared {
    world: Arc<Mutex<World>>,
    commands: Arc<Mutex<Vec<Command>>>,
    registry: Arc<Mutex<Registry>>,
    last_error: Arc<Mutex<Option<EngineError>>>,
    in_draw: Arc<AtomicBool>,
}

impl Shared {
    /// Emite um comando de desenho (fila por frame).
    fn push(&self, cmd: Command) -> mlua::Result<()> {
        self.commands.lock().expect("poisoned").push(cmd);
        Ok(())
    }

    /// Registra o erro estruturado e devolve o erro de VM correspondente.
    fn fail(&self, err: EngineError) -> mlua::Error {
        *self.last_error.lock().expect("poisoned") = Some(err.clone());
        mlua::Error::external(err.msg)
    }

    /// Guarda R9: mutação de mundo/simulação é proibida em `draw()`.
    fn guard_draw(&self, what: &str) -> mlua::Result<()> {
        if self.in_draw.load(Ordering::Relaxed) {
            return Err(self.fail(EngineError::new(
                "E016",
                format!("{what} em draw() alteraria a simulação"),
                "mova para update()/load()",
            )));
        }
        Ok(())
    }

    /// Acesso ao mundo com handle convertido e erro estruturado (R10/R13).
    fn with_entity<R>(
        &self,
        raw: u64,
        f: impl FnOnce(&mut World, EntityId) -> std::result::Result<R, ComponentError>,
    ) -> mlua::Result<R> {
        let mut world = self.world.lock().expect("poisoned");
        let id = EntityId::from_raw(raw);
        f(&mut world, id).map_err(|e| self.fail(ecs::to_engine_error(e, id)))
    }
}

pub fn install(
    lua: &Lua,
    world: Arc<Mutex<World>>,
    rng: Arc<Mutex<Rng>>,
    commands: Arc<Mutex<Vec<Command>>>,
    registry: Arc<Mutex<Registry>>,
    last_error: Arc<Mutex<Option<EngineError>>>,
    in_draw: Arc<AtomicBool>,
) -> Result<()> {
    let shared = Shared {
        world,
        commands,
        registry,
        last_error,
        in_draw,
    };
    let globals = lua.globals();
    globals.set("ent", ent_table(lua, &shared)?)?;
    globals.set("gfx", gfx_table(lua, &shared)?)?;
    globals.set(
        "rnd",
        lua.create_function(move |_, n: u64| Ok(rng.lock().expect("poisoned").below(n.max(1))))?,
    )?;
    Ok(())
}

fn ent_table(lua: &Lua, shared: &Shared) -> Result<mlua::Table> {
    let ent = lua.create_table()?;

    let s = shared.clone();
    ent.set(
        "new",
        lua.create_function(move |_, (tag, x, y): (String, Option<f32>, Option<f32>)| {
            s.guard_draw("ent.new")?;
            Ok(s.world.lock().expect("poisoned").spawn(tag, x, y).raw())
        })?,
    )?;

    let s = shared.clone();
    ent.set(
        "each",
        lua.create_function(move |lua, tag: String| {
            let world = s.world.lock().expect("poisoned");
            let table = lua.create_table()?;
            for (i, e) in world.by_tag(&tag).into_iter().enumerate() {
                table.set(i + 1, e.raw())?;
            }
            Ok(table)
        })?,
    )?;

    let s = shared.clone();
    ent.set(
        "kill",
        lua.create_function(move |_, raw: u64| {
            s.guard_draw("ent.kill")?;
            s.with_entity(raw, |world, id| world.kill(id))
        })?,
    )?;

    let s = shared.clone();
    ent.set(
        "pos",
        lua.create_function(move |_, raw: u64| s.with_entity(raw, |world, id| world.pos(id)))?,
    )?;

    let s = shared.clone();
    ent.set(
        "set_vel",
        lua.create_function(move |_, (raw, vx, vy): (u64, f32, f32)| {
            s.guard_draw("ent.set_vel")?;
            s.with_entity(raw, |world, id| world.set_velocity(id, vx, vy))
        })?,
    )?;

    Ok(ent)
}

/// Cor de paleta validada: qualquer número vira índice 0..31.
fn color(c: i64) -> u8 {
    c.clamp(0, 31) as u8
}

fn gfx_table(lua: &Lua, shared: &Shared) -> Result<mlua::Table> {
    let gfx = lua.create_table()?;

    let s = shared.clone();
    gfx.set(
        "clear",
        lua.create_function(move |_, c: i64| s.push(Command::Clear { color: color(c) }))?,
    )?;

    let s = shared.clone();
    gfx.set(
        "rect",
        lua.create_function(move |_, (x, y, w, h, c): (i32, i32, i32, i32, i64)| {
            s.push(Command::Rect {
                x,
                y,
                w,
                h,
                color: color(c),
            })
        })?,
    )?;

    let s = shared.clone();
    gfx.set(
        "rectb",
        lua.create_function(move |_, (x, y, w, h, c): (i32, i32, i32, i32, i64)| {
            s.push(Command::RectB {
                x,
                y,
                w,
                h,
                color: color(c),
            })
        })?,
    )?;

    let s = shared.clone();
    gfx.set(
        "spr",
        lua.create_function(
            move |_, (name, x, y, flipx, flipy): (String, i32, i32, Option<bool>, Option<bool>)| {
                let (index, label) = s
                    .registry
                    .lock()
                    .expect("poisoned")
                    .resolve_or_placeholder(&name);
                s.push(Command::Spr {
                    index,
                    x,
                    y,
                    flipx: flipx.unwrap_or(false),
                    flipy: flipy.unwrap_or(false),
                    label,
                })
            },
        )?,
    )?;

    let s = shared.clone();
    gfx.set(
        "text",
        lua.create_function(move |_, (text, x, y, c): (String, i32, i32, i64)| {
            s.push(Command::Text {
                s: text,
                x,
                y,
                color: color(c),
            })
        })?,
    )?;

    let s = shared.clone();
    gfx.set(
        "cam",
        lua.create_function(move |_, (x, y): (i32, i32)| s.push(Command::Cam { x, y }))?,
    )?;

    Ok(gfx)
}
