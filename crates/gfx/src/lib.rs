//! Renderer por software (ENGINE_VISION2 §3.1/§3.6-D): framebuffer indexado
//! 320×180, paleta → RGBA só na saída, raster determinístico sem GPU.

mod export;
mod font;
mod framebuffer;
mod palette;
mod raster;

pub use export::{frame_hash, write_png};
pub use font::draw_text;
pub use framebuffer::Framebuffer;
pub use palette::{PALETTE_PICO8, Palette, Rgba};
pub use raster::Raster;

/// Sprite em pixels indexados (carregado de PNG ou gerado como placeholder).
#[derive(Debug, Clone, PartialEq)]
pub struct IndexedSprite {
    pub width: u32,
    pub height: u32,
    /// `width * height` índices de paleta.
    pub pixels: Vec<u8>,
}

impl IndexedSprite {
    pub fn solid(width: u32, height: u32, color: u8) -> Self {
        Self {
            width,
            height,
            pixels: vec![color; (width * height) as usize],
        }
    }

    /// Placeholder legível (ENGINE_VISION2 §4.8): caixa com borda e miolo
    /// derivados deterministicamente do nome, visível até a arte chegar.
    pub fn placeholder(name: &str, width: u32, height: u32) -> Self {
        let mut pixels = vec![0u8; (width * height) as usize];
        let hash = fnv(name);
        let border = 1 + (hash % 30) as u8;
        let fill = 1 + ((hash >> 8) % 30) as u8;
        for y in 0..height {
            for x in 0..width {
                let border_px = x == 0 || y == 0 || x == width - 1 || y == height - 1;
                let i = (y * width + x) as usize;
                pixels[i] = if border_px { border } else { fill };
            }
        }
        Self {
            width,
            height,
            pixels,
        }
    }
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
