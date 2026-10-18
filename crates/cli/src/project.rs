//! Projeto carregado: composition root das capacidades (ENGINE_VISION2
//! §3.3 — só o `app` conhece impls concretas; aqui elas ficam reunidas).

use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use api::{Command, EngineError};
use assets::Registry;
use core::{Clock, Rng, Scene, Tick};
use ecs::World;
use gfx::{Framebuffer, Palette, Raster};
use script::ScriptHost;

use crate::EngineConfig;

/// Erro de montagem do projeto é o próprio `EngineError` (R13) — sem wrapper.
pub type BuildResult = Result<Project, EngineError>;

/// Caminho de arquivo do projeto: relativo, sem escapar da raiz (ENG-01-05-S02).
/// Absolutos e `..` são rejeitados com diagnóstico.
pub fn project_path(root: &Path, rel: &str) -> Result<PathBuf, EngineError> {
    let p = Path::new(rel);
    let escapes = p.is_absolute()
        || p.components()
            .any(|c| matches!(c, std::path::Component::ParentDir));
    if escapes {
        return Err(EngineError::new(
            "E023",
            format!("caminho fora do projeto: {rel}"),
            "use caminhos relativos dentro da pasta do projeto (sem .. nem absoluto)",
        )
        .at("engine.toml", 0));
    }
    Ok(root.join(p))
}

pub struct Project {
    palette: Palette,
    pub world: Arc<Mutex<World>>,
    pub registry: Arc<Mutex<Registry>>,
    host: Rc<ScriptHost>,
    pub clock: Clock,
    tick: Tick,
    framebuffer: Framebuffer,
    raster: Raster,
    commands: Arc<Mutex<Vec<Command>>>,
    /// Hash do último frame rasterizado (contrato de replay, R7).
    pub frame_hash: u64,
    /// Fases do último tick, na ordem executada (trace de um tick).
    pub trace: Vec<&'static str>,
}

impl Project {
    /// Monta o projeto em `root` (cwd). Seed fixa por padrão (R7).
    pub fn build(root: &Path, seed: u64) -> BuildResult {
        let config = EngineConfig::load(root)?;
        config.validate_features()?;

        let palette = match &config.palette {
            Some(p) => Palette::load(&project_path(root, p)?)?,
            None => Palette::default(),
        };

        let world = mount_scene(root, &config)?;
        let registry = load_assets(root, &palette)?;

        let rng = Arc::new(Mutex::new(Rng::new(seed)));
        let commands: Arc<Mutex<Vec<Command>>> = Arc::new(Mutex::new(Vec::new()));
        let registry = Arc::new(Mutex::new(registry));
        let in_draw = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let host = Rc::new(ScriptHost::new(
            &root.join("main.luau"),
            world.clone(),
            rng.clone(),
            commands.clone(),
            registry.clone(),
            in_draw,
        )?);
        let tick = wire_tick(&host, &commands);

        Ok(Self {
            palette,
            world,
            registry,
            host,
            clock: Clock::new(),
            tick,
            framebuffer: Framebuffer::new(),
            raster: Raster::new(),
            commands,
            frame_hash: 0,
            trace: Vec::new(),
        })
    }

    /// `load()` + N frames + raster + hash (contrato de replay, R7).
    /// `trace` registra as fases do último tick, na ordem executada.
    pub fn run_frames(&mut self, frames: u64) -> Result<(), EngineError> {
        self.host.call_load()?;
        for _ in 0..frames {
            self.trace.clear();
            self.trace.push("barrier");
            self.world.lock().expect("poisoned").barrier();
            self.tick.frame(&mut self.trace)?;
            self.trace.push("raster");
            {
                let commands = self.commands.lock().expect("poisoned");
                let registry = self.registry.lock().expect("poisoned");
                self.raster
                    .exec(&commands, registry.sprites(), &mut self.framebuffer);
            }
            self.trace.push("hash");
            self.frame_hash = gfx::frame_hash(&self.framebuffer);
            self.clock.advance();
        }
        Ok(())
    }

    /// Escreve o frame atual como PNG (cria o diretório de saída se faltar).
    pub fn save_png(&self, out: &Path) -> Result<(), EngineError> {
        if let Some(parent) = out.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|e| {
                EngineError::new(
                    "E103",
                    format!("não consegui criar {}: {e}", parent.display()),
                    "verifique permissões de escrita",
                )
            })?;
        }
        gfx::write_png(&self.framebuffer, &self.palette, out)
    }
}

/// Carrega a cena (se houver) ANTES do primeiro tick; sem cena = implícita.
fn mount_scene(root: &Path, config: &EngineConfig) -> Result<Arc<Mutex<World>>, EngineError> {
    let world = Arc::new(Mutex::new(World::new()));
    if let Some(scene_path) = &config.scene {
        let path = project_path(root, scene_path)?;
        let scene = Scene::load(&path)?;
        scene.mount(&mut world.lock().expect("poisoned"), &path)?;
    }
    Ok(world)
}

/// Carrega `assets/*.png` para o runtime: falha no PRIMEIRO asset com problema
/// (fail fast); o diagnóstico completo de todos fica com o `check` (T1).
fn load_assets(root: &Path, palette: &Palette) -> Result<Registry, EngineError> {
    let mut registry = Registry::new();
    let assets_dir = root.join("assets");
    if assets_dir.is_dir() {
        let errors = registry.load_dir(&assets_dir, palette)?;
        if let Some(e) = errors.into_iter().next() {
            return Err(e);
        }
    }
    Ok(registry)
}

/// Liga os callbacks de update/draw ao tick (o core não conhece Luau, R4).
fn wire_tick(host: &Rc<ScriptHost>, commands: &Arc<Mutex<Vec<Command>>>) -> Tick {
    let mut tick = Tick::new();
    {
        let host = host.clone();
        tick.set_update(Box::new(move || host.call_update()));
    }
    {
        let host = host.clone();
        let commands = commands.clone();
        tick.set_draw(Box::new(move || {
            commands.lock().expect("poisoned").clear();
            host.call_draw()
        }));
    }
    tick
}

/// Validação rápida sem simular (usada por `check`).
pub fn validate_world(world: &World) -> Vec<EngineError> {
    let mut errors = Vec::new();
    for e in world.active() {
        if matches!(world.lifecycle(e), Ok(ecs::Lifecycle::PendingRemoval)) {
            errors.push(EngineError::new(
                "E015",
                format!("entidade {e} presa em remoção pendente"),
                "rode a barreira do tick",
            ));
        }
    }
    errors
}
