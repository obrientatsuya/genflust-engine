//! Paleta fixa de 32 cores (ENGINE_VISION2 §2.2). Padrão: PICO-8; um arquivo
//! de texto pode sobrescrever (`i hex` por linha, índice 0..31).

use std::path::Path;

use api::EngineError;

/// Cor RGBA final (a conversão índice → RGBA acontece só na saída, §3.6-D).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

/// Paleta padrão PICO-8 (32 cores).
pub const PALETTE_PICO8: [Rgba; 32] = [
    Rgba::rgb(0x00, 0x00, 0x00),
    Rgba::rgb(0x1D, 0x2B, 0x53),
    Rgba::rgb(0x7E, 0x25, 0x53),
    Rgba::rgb(0x00, 0x87, 0x51),
    Rgba::rgb(0xAB, 0x52, 0x36),
    Rgba::rgb(0x5F, 0x57, 0x4F),
    Rgba::rgb(0xC2, 0xC3, 0xC7),
    Rgba::rgb(0xFF, 0xF1, 0xE8),
    Rgba::rgb(0xFF, 0x00, 0x4D),
    Rgba::rgb(0xFF, 0xA3, 0x00),
    Rgba::rgb(0xFF, 0xEC, 0x27),
    Rgba::rgb(0x00, 0xE4, 0x36),
    Rgba::rgb(0x29, 0xAD, 0xFF),
    Rgba::rgb(0x83, 0x76, 0x9C),
    Rgba::rgb(0xFF, 0x77, 0xA8),
    Rgba::rgb(0xFF, 0xCC, 0xAA),
    Rgba::rgb(0x29, 0x18, 0x14),
    Rgba::rgb(0x11, 0x1D, 0x35),
    Rgba::rgb(0x42, 0x21, 0x36),
    Rgba::rgb(0x12, 0x53, 0x59),
    Rgba::rgb(0x74, 0x2F, 0x29),
    Rgba::rgb(0x49, 0x33, 0x3B),
    Rgba::rgb(0xA2, 0x88, 0x79),
    Rgba::rgb(0xF3, 0xEF, 0x7D),
    Rgba::rgb(0xBE, 0x12, 0x50),
    Rgba::rgb(0xFF, 0x6C, 0x24),
    Rgba::rgb(0xA8, 0xE7, 0x2E),
    Rgba::rgb(0x00, 0xB5, 0x43),
    Rgba::rgb(0x06, 0x5A, 0xB5),
    Rgba::rgb(0x75, 0x46, 0x65),
    Rgba::rgb(0xFF, 0x6E, 0x59),
    Rgba::rgb(0xFF, 0x9D, 0x81),
];

/// Paleta do projeto: 32 cores fixas, sobrescrevíveis por arquivo de texto.
pub struct Palette {
    pub colors: [Rgba; 32],
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: PALETTE_PICO8,
        }
    }
}

impl Palette {
    /// Carrega `i hex` (i = 0..31, hex = `rrggbb`) de um arquivo de texto.
    /// Linhas em branco e comentários `#` são ignorados; cores faltantes
    /// mantêm a paleta padrão.
    pub fn load(path: &Path) -> Result<Self, EngineError> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            EngineError::new(
                "E101",
                format!("não consegui ler a paleta {}: {e}", path.display()),
                "verifique o caminho em engine.toml (palette = ...)",
            )
        })?;
        let mut colors = PALETTE_PICO8;
        for (n, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (idx, hex) = line.split_once(char::is_whitespace).ok_or_else(|| {
                EngineError::new(
                    "E102",
                    format!("paleta linha {}: esperado `i hex`", n + 1),
                    "ex.: `7 fff1e8` (índice 0..31, cor rrggbb)",
                )
            })?;
            let idx: usize = idx.trim().parse().map_err(|_| {
                EngineError::new(
                    "E102",
                    format!("paleta linha {}: índice inválido `{idx}`", n + 1),
                    "índice deve ser 0..31",
                )
            })?;
            if idx >= 32 {
                return Err(EngineError::new(
                    "E102",
                    format!("paleta linha {}: índice {idx} fora de 0..31", n + 1),
                    "índice deve ser 0..31",
                ));
            }
            let hex = hex.trim();
            if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(EngineError::new(
                    "E102",
                    format!("paleta linha {}: cor `{hex}` inválida", n + 1),
                    "formato rrggbb, ex.: `ff004d`",
                ));
            }
            let v = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            colors[idx] = Rgba::rgb(v(0), v(2), v(4));
        }
        Ok(Self { colors })
    }

    pub fn rgba(&self, index: u8) -> Rgba {
        self.colors[index.min(31) as usize]
    }

    /// Índice de paleta mais próximo de uma cor RGBA (para carregar PNGs).
    pub fn nearest(&self, c: Rgba) -> u8 {
        let mut best = 0u8;
        let mut best_d = i64::MAX;
        for (i, p) in self.colors.iter().enumerate() {
            let dr = p.r as i64 - c.r as i64;
            let dg = p.g as i64 - c.g as i64;
            let db = p.b as i64 - c.b as i64;
            let d = dr * dr + dg * dg + db * db;
            if d < best_d {
                best_d = d;
                best = i as u8;
            }
        }
        best
    }
}
