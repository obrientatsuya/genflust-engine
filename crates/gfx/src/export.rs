//! Saída do framebuffer: PNG (RGBA via LUT de paleta) e hash determinístico.
//!
//! O hash é o contrato de reprodução (R7): mesma seed → mesmos bytes de
//! framebuffer → mesmo hash, em qualquer SO.

use std::path::Path;

use api::EngineError;

use crate::{Framebuffer, Palette};

/// FNV-1a 64 sobre os pixels do framebuffer (ordem fixa de varredura).
pub fn frame_hash(fb: &Framebuffer) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in fb.pixels() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Escreve o framebuffer como PNG 8-bit RGB (conversão de paleta só aqui,
/// na saída — ENGINE_VISION2 §3.6-D).
pub fn write_png(fb: &Framebuffer, palette: &Palette, path: &Path) -> Result<(), EngineError> {
    let mut rgba = Vec::with_capacity(fb.pixels().len() * 3);
    for &p in fb.pixels() {
        let c = palette.rgba(p);
        rgba.extend_from_slice(&[c.r, c.g, c.b]);
    }
    let file = std::fs::File::create(path).map_err(|e| {
        EngineError::new(
            "E103",
            format!("não consegui criar {}: {e}", path.display()),
            "verifique permissões de escrita do diretório de saída",
        )
    })?;
    let mut enc = png::Encoder::new(file, fb.width as u32, fb.height as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| {
        EngineError::new(
            "E103",
            format!("erro de PNG: {e}"),
            "erro interno de encoder",
        )
    })?;
    writer.write_image_data(&rgba).map_err(|e| {
        EngineError::new(
            "E103",
            format!("erro de PNG: {e}"),
            "erro interno de encoder",
        )
    })?;
    Ok(())
}
