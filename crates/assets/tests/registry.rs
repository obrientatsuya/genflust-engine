//! Assets: PNG real, corrompido, placeholders (ENT-01-13).

use std::path::PathBuf;

use assets::{Registry, load_png};
use gfx::{IndexedSprite, Palette};

fn temp_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("genflust-assets-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// Gera um PNG 2×1 (branco, vermelho) em disco e devolve o caminho.
fn write_white_red_png() -> PathBuf {
    let path = temp_path("white_red.png");
    let file = std::fs::File::create(&path).unwrap();
    let mut enc = png::Encoder::new(file, 2, 1);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().unwrap();
    // branco (fff1e8) e vermelho (ff004d) — cores exatas da paleta PICO-8
    w.write_image_data(&[0xff, 0xf1, 0xe8, 0xff, 0x00, 0x4d])
        .unwrap();
    path
}

#[test]
fn loads_real_png_and_maps_colors_to_palette() {
    let path = write_white_red_png();
    let sprite = load_png(&path, &Palette::default()).unwrap();
    assert_eq!((sprite.width, sprite.height), (2, 1));
    assert_eq!(sprite.pixels, vec![7, 8]);
    std::fs::remove_file(&path).ok();
}

#[test]
fn corrupt_png_is_error_not_silence() {
    let path = temp_path("corrupt.png");
    std::fs::write(&path, b"isto nao e um png valido").unwrap();
    let err = load_png(&path, &Palette::default()).unwrap_err();
    assert_eq!(err.code, "E100", "corrompido deve virar erro estruturado");
    assert!(err.file.contains("corrupt.png"));
    std::fs::remove_file(&path).ok();
}

#[test]
fn name_registry_keeps_swapping_pixels_transparent() {
    // Trocar pixels mantendo o nome não altera a resolução de gameplay.
    let mut reg = Registry::new();
    let (idx_a, label_a) = reg.resolve_or_placeholder("player_idle");
    assert_eq!(label_a.as_deref(), Some("player_idle"), "ainda sem arte");
    let (idx_b, _) = reg.resolve_or_placeholder("player_idle");
    assert_eq!(idx_a, idx_b);
    assert_eq!(
        reg.sprite(idx_a).unwrap(),
        &IndexedSprite::placeholder8("player_idle")
    );
}

#[test]
fn placeholder_not_recreated_per_call() {
    let mut reg = Registry::new();
    for _ in 0..1000 {
        reg.resolve_or_placeholder("bullet");
    }
    assert_eq!(
        reg.sprites().len(),
        1,
        "placeholder deve ser criado uma vez e reusado (sem conversão por frame)"
    );
}
