//! Fonte bitmap 5×7 embutida (ASCII 32..=126). Sem dependência de sistema;
//! determinística em qualquer SO.

use crate::Framebuffer;

/// 5 colunas por glifo, 7 linhas, bit 0 = pixel superior-esquerdo por coluna.
const GLYPHS: &[u8; 95 * 5] = &include!("glyphs.in");

/// Fallback estável para glifos fora da tabela (box vazado), sem fontes do
/// sistema: mesma entrada = mesmos pixels em qualquer SO.
const BOX: [u8; 5] = [0x7f, 0x41, 0x41, 0x41, 0x7f];

pub fn draw_text(fb: &mut Framebuffer, s: &str, x: i32, y: i32, color: u8) {
    let mut cx = x;
    for ch in s.chars() {
        if ch == '\n' {
            continue;
        }
        let idx = (ch as u32).wrapping_sub(32);
        let cols = if idx < 95 {
            let base = idx as usize * 5;
            [
                GLYPHS[base],
                GLYPHS[base + 1],
                GLYPHS[base + 2],
                GLYPHS[base + 3],
                GLYPHS[base + 4],
            ]
        } else {
            BOX
        };
        for (col, bits) in cols.iter().enumerate() {
            for row in 0..7 {
                if bits & (1 << row) != 0 {
                    fb.set(cx + col as i32, y + row, color);
                }
            }
        }
        cx += 6;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draws_something_for_ascii() {
        let mut fb = Framebuffer::new();
        draw_text(&mut fb, "A", 0, 0, 7);
        let painted: usize = fb.pixels().iter().filter(|&&p| p == 7).count();
        assert!(painted > 3, "glifo 'A' deve pintar pixels");
    }

    #[test]
    fn unknown_glyph_has_stable_box_fallback() {
        // 'ç' está fora da tabela ASCII embutida (e aparece em rótulos reais).
        let mut a = Framebuffer::new();
        draw_text(&mut a, "ç", 0, 0, 7);
        let painted: usize = a.pixels().iter().filter(|&&p| p == 7).count();
        assert!(
            painted > 0,
            "glifo desconhecido desenha fallback, nunca some"
        );
        let mut b = Framebuffer::new();
        draw_text(&mut b, "ç", 0, 0, 7);
        assert_eq!(
            a.pixels(),
            b.pixels(),
            "fallback determinístico, sem fonte do sistema"
        );
    }
}
