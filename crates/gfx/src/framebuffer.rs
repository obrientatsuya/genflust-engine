//! Framebuffer indexado: 320×180 × u8 (ENGINE_VISION2 §3.6-D).

use api::{HEIGHT, WIDTH};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u8>,
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Framebuffer {
    pub fn new() -> Self {
        Self {
            width: WIDTH,
            height: HEIGHT,
            pixels: vec![0; WIDTH * HEIGHT],
        }
    }

    pub fn clear(&mut self, color: u8) {
        self.pixels.fill(color);
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, color: u8) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = color;
        }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize]
        } else {
            0
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}
