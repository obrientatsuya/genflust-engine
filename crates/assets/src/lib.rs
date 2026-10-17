//! Assets (ENGINE_VISION2 §4.8): PNG → sprites indexados, registro por nome
//! e placeholders automáticos. Fonte da verdade: PNG + TOML; `.pak` é futuro.

use std::collections::HashMap;
use std::path::Path;

use api::EngineError;
use gfx::{IndexedSprite, Palette};

/// Registro de sprites: índice estável por asset (usado pelos comandos).
#[derive(Default)]
pub struct Registry {
    sprites: Vec<IndexedSprite>,
    names: HashMap<String, u32>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Índice de um asset por nome (None = ainda não carregado/desenhado).
    pub fn resolve(&self, name: &str) -> Option<u32> {
        self.names.get(name).copied()
    }

    /// Índice real ou de placeholder: nome ausente NUNCA é erro silencioso
    /// (R18) — devolve placeholder + o nome para rótulo (§4.8).
    pub fn resolve_or_placeholder(&mut self, name: &str) -> (u32, Option<String>) {
        if let Some(i) = self.resolve(name) {
            return (i, None);
        }
        let sprite = IndexedSprite::placeholder8(name);
        let index = self.sprites.len() as u32;
        self.sprites.push(sprite);
        (index, Some(name.to_string()))
    }

    pub fn sprite(&self, index: u32) -> Option<&IndexedSprite> {
        self.sprites.get(index as usize)
    }

    pub fn sprites(&self) -> &[IndexedSprite] {
        &self.sprites
    }

    /// Nomes conhecidos (para `check`: referência não-placeholder).
    pub fn known(&self) -> impl Iterator<Item = &str> {
        self.names.keys().map(|s| s.as_str())
    }

    /// Carrega `assets/*.png` de um diretório; erro estruturado por arquivo
    /// corrompido (ENG-01-13-S04), nunca silencioso.
    pub fn load_dir(
        &mut self,
        dir: &Path,
        palette: &Palette,
    ) -> Result<Vec<EngineError>, EngineError> {
        let mut errors = Vec::new();
        let entries = std::fs::read_dir(dir).map_err(|e| {
            EngineError::new(
                "E100",
                format!("não consegui ler {}: {e}", dir.display()),
                "crie a pasta assets/ ou corrija o caminho em engine.toml",
            )
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            match load_png(&path, palette) {
                Ok(sprite) => {
                    self.add(name, sprite);
                }
                Err(e) => errors.push(e.at(path.display().to_string(), 0)),
            }
        }
        Ok(errors)
    }

    fn add(&mut self, name: &str, sprite: IndexedSprite) -> u32 {
        let index = self.sprites.len() as u32;
        self.sprites.push(sprite);
        self.names.insert(name.to_string(), index);
        index
    }
}

/// PNG RGBA → sprite indexado pela paleta (aproximação por cor mais próxima).
pub fn load_png(path: &Path, palette: &Palette) -> Result<IndexedSprite, EngineError> {
    let file = std::fs::File::open(path).map_err(|e| {
        EngineError::new(
            "E100",
            format!("não consegui abrir {}: {e}", path.display()),
            "verifique o caminho e as permissões do arquivo",
        )
    })?;
    let file = std::io::BufReader::new(file);
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().map_err(|e| {
        EngineError::new(
            "E100",
            format!("PNG corrompido {}: {e}", path.display()),
            "re-exporte o arquivo como PNG válido",
        )
    })?;
    let mut buf = vec![0; reader.output_buffer_size().unwrap_or(1 << 26)];
    let info = reader.next_frame(&mut buf).map_err(|e| {
        EngineError::new(
            "E100",
            format!("PNG ilegível {}: {e}", path.display()),
            "re-exporte o arquivo como PNG válido",
        )
    })?;
    let rgba = &buf[..info.buffer_size()];
    let (w, h) = (info.width, info.height);
    let bpp = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        other => {
            return Err(EngineError::new(
                "E100",
                format!("{}: formato {other:?} não suportado", path.display()),
                "use PNG 8-bit RGB ou RGBA",
            ));
        }
    };
    let mut pixels = Vec::with_capacity((w * h) as usize);
    for i in 0..(w * h) as usize {
        let o = i * bpp;
        let c = gfx::Rgba {
            r: rgba[o],
            g: rgba[o + 1],
            b: rgba[o + 2],
            a: 255,
        };
        pixels.push(palette.nearest(c));
    }
    Ok(IndexedSprite {
        width: w,
        height: h,
        pixels,
    })
}
