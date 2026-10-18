//! Bindings Luau (ENGINE_VISION2 §5, fase 1): `gfx.*`, `ent.*`, `rnd`.
//!
//! Instalados uma vez; estado compartilhado via `Arc<Mutex<...>>`. Erros de
//! binding carregam `code`/`msg` estruturados (R13) e a guarda R9 bloqueia
//! mutação de mundo dentro de `draw()`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use api::{Command, EngineError};
use assets::Registry;
use core::Rng;
use ecs::{EntityId, World};
use mlua::{Lua, Result};

pub fn install(
    lua: &Lua,
    world: Arc<Mutex<World>>,
    rng: Arc<Mutex<Rng>>,
    commands: Arc<Mutex<Vec<Command>>>,
    registry: Arc<Mutex<Registry>>,
    last_error: Arc<Mutex<Option<EngineError>>>,
    in_draw: Arc<AtomicBool>,
) -> Result<()> {
    let globals = lua.globals();

    let ent = lua.create_table()?;
    {
        let world = world.clone();
        let in_draw = in_draw.clone();
        let last_error = last_error.clone();
        let new =
            lua.create_function(move |_, (tag, x, y): (String, Option<f32>, Option<f32>)| {
                if in_draw.load(Ordering::Relaxed) {
                    return Err(guarded(
                        &last_error,
                        "E016",
                        "ent.new em draw() alteraria a simulação",
                        "crie entidades em update()/load()",
                    ));
                }
                let mut w = world.lock().expect("poisoned");
                Ok(w.spawn(tag, x, y).raw())
            })?;
        ent.set("new", new)?;
    }
    {
        let world = world.clone();
        let each = lua.create_function(move |lua, tag: String| {
            let w = world.lock().expect("poisoned");
            let table = lua.create_table()?;
            for (i, e) in w.by_tag(&tag).into_iter().enumerate() {
                table.set(i + 1, e.raw())?;
            }
            Ok(table)
        })?;
        ent.set("each", each)?;
    }
    {
        let world = world.clone();
        let in_draw = in_draw.clone();
        let last_error = last_error.clone();
        let kill = lua.create_function(move |_, raw: u64| {
            if in_draw.load(Ordering::Relaxed) {
                return Err(guarded(
                    &last_error,
                    "E016",
                    "ent.kill em draw() alteraria a simulação",
                    "remova entidades em update()/load()",
                ));
            }
            let mut w = world.lock().expect("poisoned");
            let id = EntityId::from_raw(raw);
            w.kill(id).map_err(|e| {
                let err = ecs::to_engine_error(e, id);
                *last_error.lock().expect("poisoned") = Some(err.clone());
                mlua::Error::external(err.msg)
            })
        })?;
        ent.set("kill", kill)?;
    }
    globals.set("ent", ent)?;

    let gfx = lua.create_table()?;
    let color = |c: i64| c.clamp(0, 31) as u8;
    {
        let commands = commands.clone();
        let clear = lua.create_function(move |_, c: i64| {
            commands
                .lock()
                .expect("poisoned")
                .push(Command::Clear { color: color(c) });
            Ok(())
        })?;
        gfx.set("clear", clear)?;
    }
    {
        let commands = commands.clone();
        let rect = lua.create_function(move |_, (x, y, w, h, c): (i32, i32, i32, i32, i64)| {
            commands.lock().expect("poisoned").push(Command::Rect {
                x,
                y,
                w,
                h,
                color: color(c),
            });
            Ok(())
        })?;
        gfx.set("rect", rect)?;
    }
    {
        let commands = commands.clone();
        let rectb = lua.create_function(move |_, (x, y, w, h, c): (i32, i32, i32, i32, i64)| {
            commands.lock().expect("poisoned").push(Command::RectB {
                x,
                y,
                w,
                h,
                color: color(c),
            });
            Ok(())
        })?;
        gfx.set("rectb", rectb)?;
    }
    {
        let commands = commands.clone();
        let registry = registry.clone();
        let spr = lua.create_function(
            move |_, (name, x, y, flipx, flipy): (String, i32, i32, Option<bool>, Option<bool>)| {
                let (index, label) = registry
                    .lock()
                    .expect("poisoned")
                    .resolve_or_placeholder(&name);
                commands.lock().expect("poisoned").push(Command::Spr {
                    index,
                    x,
                    y,
                    flipx: flipx.unwrap_or(false),
                    flipy: flipy.unwrap_or(false),
                    label,
                });
                Ok(())
            },
        )?;
        gfx.set("spr", spr)?;
    }
    {
        let commands = commands.clone();
        let text = lua.create_function(move |_, (s, x, y, c): (String, i32, i32, i64)| {
            commands.lock().expect("poisoned").push(Command::Text {
                s,
                x,
                y,
                color: color(c),
            });
            Ok(())
        })?;
        gfx.set("text", text)?;
    }
    {
        let commands = commands.clone();
        let cam = lua.create_function(move |_, (x, y): (i32, i32)| {
            commands
                .lock()
                .expect("poisoned")
                .push(Command::Cam { x, y });
            Ok(())
        })?;
        gfx.set("cam", cam)?;
    }
    globals.set("gfx", gfx)?;

    let rnd =
        lua.create_function(move |_, n: u64| Ok(rng.lock().expect("poisoned").below(n.max(1))))?;
    globals.set("rnd", rnd)?;

    Ok(())
}

fn guarded(
    last_error: &Arc<Mutex<Option<EngineError>>>,
    code: &'static str,
    msg: &str,
    fix: &str,
) -> mlua::Error {
    *last_error.lock().expect("poisoned") = Some(EngineError::new(code, msg, fix));
    mlua::Error::external(msg)
}
