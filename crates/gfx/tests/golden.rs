//! Golden tests do raster (ENT-01-13/14/15): fixtures versionadas em
//! `fixtures/fase1.txt` (nome=hash). Regenerar: `GEN_FIXTURE=1 cargo test -p gfx`.

use std::collections::HashMap;
use std::path::PathBuf;

use api::Command;
use gfx::{Framebuffer, IndexedSprite, Palette, Raster, frame_hash, write_png};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fase1.txt")
}

fn gen_mode() -> bool {
    std::env::var("GEN_FIXTURE").is_ok()
}

fn raster(commands: &[Command], sprites: &[IndexedSprite]) -> Framebuffer {
    let mut fb = Framebuffer::new();
    let mut r = Raster::new();
    r.exec(commands, sprites, &mut fb);
    fb
}

/// Cenas nomeadas: mesma definição usada pra gerar e conferir a fixture.
fn scenes() -> Vec<(&'static str, Framebuffer)> {
    let mut out = Vec::new();

    // clear exato
    out.push(("clear_7", raster(&[Command::Clear { color: 7 }], &[])));

    // retângulos nas bordas (negativo, estouro, contorno parcial)
    out.push((
        "rect_borders",
        raster(
            &[
                Command::Clear { color: 0 },
                Command::Rect {
                    x: -4,
                    y: -4,
                    w: 10,
                    h: 10,
                    color: 3,
                },
                Command::Rect {
                    x: 315,
                    y: 175,
                    w: 10,
                    h: 10,
                    color: 5,
                },
                Command::RectB {
                    x: -2,
                    y: 170,
                    w: 8,
                    h: 8,
                    color: 8,
                },
            ],
            &[],
        ),
    ));

    // sprite: blit, espelhamento, fora da tela, sobreposição
    let small = IndexedSprite {
        width: 3,
        height: 2,
        pixels: vec![1, 2, 3, 4, 5, 6],
    };
    out.push((
        "spr_flip_overlap",
        raster(
            &[
                Command::Clear { color: 0 },
                Command::Spr {
                    index: 0,
                    x: -1,
                    y: -1,
                    flipx: false,
                    flipy: false,
                    label: None,
                },
                Command::Spr {
                    index: 0,
                    x: 4,
                    y: 2,
                    flipx: true,
                    flipy: false,
                    label: None,
                },
                Command::Spr {
                    index: 0,
                    x: 6,
                    y: 3,
                    flipx: false,
                    flipy: true,
                    label: None,
                },
                Command::Spr {
                    index: 0,
                    x: 5,
                    y: 3,
                    flipx: false,
                    flipy: false,
                    label: None,
                },
            ],
            &[small],
        ),
    ));

    // placeholder estável para um nome conhecido
    let ph = IndexedSprite::placeholder8("player_idle");
    out.push((
        "placeholder_player_idle",
        raster(
            &[Command::Spr {
                index: 0,
                x: 2,
                y: 2,
                flipx: false,
                flipy: false,
                label: Some("player_idle".into()),
            }],
            &[ph],
        ),
    ));

    out
}

fn run() -> (Vec<(&'static str, u64)>, HashMap<String, u64>) {
    let computed: Vec<(&'static str, u64)> = scenes()
        .into_iter()
        .map(|(n, fb)| (n, frame_hash(&fb)))
        .collect();
    let fixture = load_fixture();
    (computed, fixture)
}

/// Lê `nome=hash` do arquivo de fixture (hex).
fn load_fixture() -> HashMap<String, u64> {
    let mut map = HashMap::new();
    let Ok(text) = std::fs::read_to_string(fixture_path()) else {
        return map;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=')
            && let Ok(h) = u64::from_str_radix(v.trim(), 16)
        {
            map.insert(k.trim().to_string(), h);
        }
    }
    map
}

fn write_fixture(computed: &[(&'static str, u64)]) {
    let mut text =
        String::from("# Fixtures golden do raster (fase 1). hash FNV-1a 64 do framebuffer, hex.\n");
    for (name, hash) in computed {
        text.push_str(&format!("{name}={hash:016x}\n"));
    }
    std::fs::create_dir_all(fixture_path().parent().unwrap()).unwrap();
    std::fs::write(fixture_path(), text).unwrap();
}

#[test]
fn golden_matches_versioned_fixture() {
    let (computed, fixture) = run();
    if gen_mode() {
        write_fixture(&computed);
        eprintln!("fixture regerada em {}", fixture_path().display());
        return;
    }
    assert!(
        !fixture.is_empty(),
        "fixture ausente: rode GEN_FIXTURE=1 cargo test -p gfx"
    );
    for (name, hash) in &computed {
        let expected = fixture
            .get(*name)
            .unwrap_or_else(|| panic!("fixture sem entrada para {name}"));
        assert_eq!(
            *expected, *hash,
            "golden divergiu em {name}: esperado {expected:016x}, obtido {hash:016x}"
        );
    }
}

#[test]
fn clear_is_exact_on_every_pixel() {
    let fb = raster(&[Command::Clear { color: 9 }], &[]);
    assert!(fb.pixels().iter().all(|&p| p == 9));
}

#[test]
fn rect_never_writes_out_of_bounds() {
    let fb = raster(
        &[
            Command::Rect {
                x: -10,
                y: -10,
                w: 400,
                h: 300,
                color: 4,
            },
            Command::RectB {
                x: -5,
                y: -5,
                w: 20,
                h: 20,
                color: 6,
            },
        ],
        &[],
    );
    // Só o interior protegido: leitura externa devolve 0 por contrato.
    assert_eq!(fb.get(-1, -1), 0);
    assert_eq!(fb.get(320, 0), 0);
    assert_eq!(fb.get(0, 180), 0);
}

#[test]
fn spr_flip_is_pixel_exact() {
    let s = IndexedSprite {
        width: 2,
        height: 1,
        pixels: vec![1, 2],
    };
    let normal = raster(
        &[Command::Spr {
            index: 0,
            x: 0,
            y: 0,
            flipx: false,
            flipy: false,
            label: None,
        }],
        std::slice::from_ref(&s),
    );
    let flipped = raster(
        &[Command::Spr {
            index: 0,
            x: 0,
            y: 0,
            flipx: true,
            flipy: false,
            label: None,
        }],
        &[s],
    );
    assert_eq!(normal.get(0, 0), 1);
    assert_eq!(normal.get(1, 0), 2);
    assert_eq!(flipped.get(0, 0), 2);
    assert_eq!(flipped.get(1, 0), 1);
}

#[test]
fn placeholder_is_deterministic_per_name() {
    assert_eq!(
        IndexedSprite::placeholder8("player_idle"),
        IndexedSprite::placeholder8("player_idle")
    );
    assert_ne!(
        IndexedSprite::placeholder8("player_idle"),
        IndexedSprite::placeholder8("coin")
    );
}

#[test]
fn png_roundtrip_matches_palette_lut() {
    let palette = Palette::default();
    let fb = raster(
        &[
            Command::Clear { color: 0 },
            Command::Rect {
                x: 1,
                y: 1,
                w: 2,
                h: 2,
                color: 8,
            },
        ],
        &[],
    );
    let out = std::env::temp_dir().join(format!("genflust-golden-{}.png", std::process::id()));
    write_png(&fb, &palette, &out).unwrap();

    let file = std::io::BufReader::new(std::fs::File::open(&out).unwrap());
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size().unwrap_or(1 << 20)];
    let info = reader.next_frame(&mut buf).unwrap();
    assert_eq!((info.width, info.height), (320, 180));
    // Fundo = paleta[0]; retângulo = paleta[8] — mesmo LUT do framebuffer.
    let bg = palette.rgba(0);
    let rect = palette.rgba(8);
    assert_eq!(&buf[0..3], &[bg.r, bg.g, bg.b]);
    let idx = ((info.width + 1) * 3) as usize;
    assert_eq!(&buf[idx..idx + 3], &[rect.r, rect.g, rect.b]);
    std::fs::remove_file(&out).ok();
}
