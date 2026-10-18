//! Raster: executa comandos de desenho sobre o framebuffer (ENGINE_VISION2
//! §3.2: command buffer → raster → framebuffer). Clipping antes de rasterizar.
//!
//! Sprites são resolvidos por índice contra a tabela de sprites (assets);
//! nome ausente vira placeholder + rótulo (§4.8), nunca erro silencioso (R18).

use api::Command;

use crate::{Framebuffer, IndexedSprite};

pub struct Raster {
    cam_x: i32,
    cam_y: i32,
}

impl Default for Raster {
    fn default() -> Self {
        Self::new()
    }
}

impl Raster {
    pub fn new() -> Self {
        Self { cam_x: 0, cam_y: 0 }
    }

    /// Executa um lote de comandos na ordem emitida (ordem estável, R7).
    pub fn exec(&mut self, commands: &[Command], sprites: &[IndexedSprite], fb: &mut Framebuffer) {
        for cmd in commands {
            match cmd {
                Command::Clear { color } => fb.clear(*color),
                Command::Cam { x, y } => {
                    self.cam_x = *x;
                    self.cam_y = *y;
                }
                Command::Rect { x, y, w, h, color } => {
                    self.rect(fb, x - self.cam_x, y - self.cam_y, *w, *h, *color);
                }
                Command::RectB { x, y, w, h, color } => {
                    self.rectb(fb, x - self.cam_x, y - self.cam_y, *w, *h, *color);
                }
                Command::Spr {
                    index,
                    x,
                    y,
                    flipx,
                    flipy,
                    label,
                } => {
                    // Sprite vem SEMPRE do registro (inclui placeholders);
                    // o rótulo é só texto, sem recriar sprite no hot path.
                    if let Some(sprite) = sprites.get(*index as usize) {
                        let sprite = sprite.clone();
                        self.spr(fb, &sprite, x - self.cam_x, y - self.cam_y, *flipx, *flipy);
                    }
                    if let Some(name) = label {
                        crate::draw_text(fb, name, x - self.cam_x + 2, y - self.cam_y + 2, 1);
                    }
                }
                Command::Text { s, x, y, color } => {
                    crate::draw_text(fb, s, x - self.cam_x, y - self.cam_y, *color);
                }
            }
        }
    }

    fn rect(&self, fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, color: u8) {
        for py in y..y.saturating_add(h) {
            for px in x..x.saturating_add(w) {
                fb.set(px, py, color);
            }
        }
    }

    fn rectb(&self, fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, color: u8) {
        for px in x..x.saturating_add(w) {
            fb.set(px, y, color);
            fb.set(px, y + h - 1, color);
        }
        for py in y..y.saturating_add(h) {
            fb.set(x, py, color);
            fb.set(x + w - 1, py, color);
        }
    }

    fn spr(
        &self,
        fb: &mut Framebuffer,
        s: &IndexedSprite,
        x: i32,
        y: i32,
        flipx: bool,
        flipy: bool,
    ) {
        for py in 0..s.height as i32 {
            for px in 0..s.width as i32 {
                let sx = if flipx { s.width as i32 - 1 - px } else { px };
                let sy = if flipy { s.height as i32 - 1 - py } else { py };
                let color = s.pixels[(sy * s.width as i32 + sx) as usize];
                fb.set(x + px, y + py, color);
            }
        }
    }
}

impl IndexedSprite {
    /// Placeholder 8×8 padrão para sprite ausente.
    pub fn placeholder8(name: &str) -> Self {
        Self::placeholder(name, 8, 8)
    }
}
