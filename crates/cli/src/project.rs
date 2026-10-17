//! Projeto carregado: composition root das capacidades (ENGINE_VISION2
//! §3.3 — só o `app` conhece impls concretas; aqui elas ficam reunidas).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use api::{Command, EngineError, Formatter, Value};
use assets::Registry;
use core::{Clock, Rng, Scene, Tick};
use ecs::World;
use gfx::{Framebuffer, Palette, Raster};
use script::ScriptHost;

use crate::EngineConfig;

/// Erro de montagem do projeto (config/assets/cena/script).
#[derive(Debug)]
pub enum ProjectError {
    Missing(EngineError),
}

impl ProjectError {
    pub fn into_engine(self) -> EngineError {
        match self {
            Self::Missing(e) => e,
        }
    }
}

impl From<EngineError> for ProjectError {
    fn from(e: EngineError) -> Self {
        Self::Missing(e)
    }
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
    pub seed: u64,
}

impl Project {
    /// Monta o projeto em `root` (cwd). Seed fixa por padrão (R7).
    pub fn build(root: &Path, seed: u64) -> Result<Self, ProjectError> {
        let config = EngineConfig::load(root)?;

        let palette = match &config.palette {
            Some(p) => Palette::load(&root.join(p))?,
            None => Palette::default(),
        };

        let mut registry = Registry::new();
        let assets_dir = root.join("assets");
        if assets_dir.is_dir() {
            let errors = registry.load_dir(&assets_dir, &palette)?;
            if let Some(e) = errors.into_iter().next() {
                return Err(ProjectError::Missing(e));
            }
        }

        let world = Arc::new(Mutex::new(World::new()));
        if let Some(scene_path) = &config.scene {
            let scene = Scene::load(&root.join(scene_path))?;
            scene.mount(&mut world.lock().expect("poisoned"))?;
        }

        let rng = Arc::new(Mutex::new(Rng::new(seed)));
        let commands: Arc<Mutex<Vec<Command>>> = Arc::new(Mutex::new(Vec::new()));
        let registry = Arc::new(Mutex::new(registry));
        let in_draw = Arc::new(Mutex::new(false));

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
            seed,
        })
    }

    /// `load()` + N frames + raster + hash (contrato de replay, R7).
    pub fn run_frames(&mut self, frames: u64) -> Result<(), EngineError> {
        self.host.call_load()?;
        for _ in 0..frames {
            self.world.lock().expect("poisoned").barrier();
            self.tick.frame()?;
            {
                let commands = self.commands.lock().expect("poisoned");
                let registry = self.registry.lock().expect("poisoned");
                self.raster
                    .exec(&commands, registry.sprites(), &mut self.framebuffer);
            }
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

    /// Saída estruturada do estado (para testes/verificação).
    pub fn state_value(&self) -> Value {
        let w = self.world.lock().expect("poisoned");
        let rows: Vec<Vec<Value>> = w
            .active()
            .into_iter()
            .map(|e| {
                let t = w
                    .transform(e)
                    .map(|t| format!("{:.1},{:.1}", t.x, t.y))
                    .unwrap_or_default();
                let s = w.sprite(e).map(|s| s.asset.clone()).unwrap_or_default();
                vec![Value::i(e.raw() as i64), Value::s(t), Value::s(s)]
            })
            .collect();
        Value::table(vec!["entity".into(), "pos".into(), "sprite".into()], rows)
    }

    /// Imprime valor com o formatter pedido.
    pub fn print(&self, title: &str, value: &Value, fmt: &dyn Formatter) {
        print!("{}", fmt.format(title, value));
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
