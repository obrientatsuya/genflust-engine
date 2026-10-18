//! Projeto carregado: composition root das capacidades (ENGINE_VISION2
//! §3.3 — só o `app` conhece impls concretas; aqui elas ficam reunidas).

use std::cell::RefCell;
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
    pub root: PathBuf,
    pub config: EngineConfig,
    pub palette: Palette,
    pub world: Arc<Mutex<World>>,
    pub rng: Arc<Mutex<Rng>>,
    pub registry: Arc<Mutex<Registry>>,
    pub host: Rc<ScriptHost>,
    pub clock: Clock,
    pub tick: Tick,
    pub framebuffer: Framebuffer,
    pub raster: Raster,
    pub commands: Arc<Mutex<Vec<Command>>>,
    /// Hash do último frame rasterizado (contrato de replay, R7).
    pub frame_hash: u64,
    /// Fases do último tick, na ordem executada (trace de um tick).
    pub trace: Vec<&'static str>,
    pub seed: u64,
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

        let mut registry = Registry::new();
        let assets_dir = root.join("assets");
        if assets_dir.is_dir() {
            let errors = registry.load_dir(&assets_dir, &palette)?;
            if let Some(e) = errors.into_iter().next() {
                return Err(e);
            }
        }

        let world = Arc::new(Mutex::new(World::new()));
        if let Some(scene_path) = &config.scene {
            let scene = Scene::load(&project_path(root, scene_path)?)?;
            scene.mount(&mut world.lock().expect("poisoned"))?;
        }

        let rng = Arc::new(Mutex::new(Rng::new(seed)));
        let commands: Arc<Mutex<Vec<Command>>> = Arc::new(Mutex::new(Vec::new()));
        let registry = Arc::new(Mutex::new(registry));
        let in_draw = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let main = root.join("main.luau");
        let host = Rc::new(ScriptHost::new(
            &main,
            world.clone(),
            rng.clone(),
            commands.clone(),
            registry.clone(),
            in_draw.clone(),
        )?);

        let mut tick = Tick::new();
        {
            let host = host.clone();
            tick.set_update(Rc::new(RefCell::new(move || host.call_update())));
        }
        {
            let host = host.clone();
            let commands = commands.clone();
            tick.set_draw(Rc::new(RefCell::new(move || {
                commands.lock().expect("poisoned").clear();
                host.call_draw()
            })));
        }

        Ok(Self {
            root: root.to_path_buf(),
            config,
            palette,
            world,
            rng,
            registry,
            host,
            clock: Clock::new(),
            tick,
            framebuffer: Framebuffer::new(),
            raster: Raster::new(),
            commands,
            frame_hash: 0,
            trace: Vec::new(),
            seed,
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
