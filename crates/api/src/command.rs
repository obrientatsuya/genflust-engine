//! Comando de desenho: o que `draw()` produz e o rasterizador consome.
//!
//! O Luau nunca toca o framebuffer diretamente; ele só emite comandos
//! (ENGINE_VISION2 §3.2: draw → command buffer → raster).

/// Comando visual gerado por `draw()` (Luau) e executado pelo raster (Rust).
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Limpa o framebuffer com a cor de paleta `color`.
    Clear { color: u8 },
    /// Retângulo preenchido.
    Rect {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: u8,
    },
    /// Contorno de retângulo.
    RectB {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: u8,
    },
    /// Sprite resolvido (índice no registro de assets).
    Spr {
        index: u32,
        x: i32,
        y: i32,
        flipx: bool,
        flipy: bool,
        /// Nome pedido quando o asset não existe (placeholder + rótulo, §4.8).
        label: Option<String>,
    },
    /// Texto com fonte bitmap embutida.
    Text {
        s: String,
        x: i32,
        y: i32,
        color: u8,
    },
    /// Deslocamento de câmera aplicado aos comandos seguintes do frame.
    Cam { x: i32, y: i32 },
}
