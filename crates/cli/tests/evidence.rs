//! Evidências fase 1 (cli): orçamento adversarial, falha após mutação,
//! trace do tick, sandbox, caminho fora do projeto, screenshot golden.

use std::path::{Path, PathBuf};

use cli::Project;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "genflust-ev-{}-{name}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_project(dir: &Path, config: &str, main: &str) {
    std::fs::write(dir.join("engine.toml"), config).unwrap();
    std::fs::write(dir.join("main.luau"), main).unwrap();
}

fn hello() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("cli em crates/cli")
        .join("examples")
        .join("hello")
}

#[test]
fn infinite_loop_hits_budget_and_process_keeps_alive() {
    let dir = scratch("loop");
    write_project(
        &dir,
        "name = \"loop\"\n",
        "function update(dt) while true do end end\n",
    );
    let mut p = Project::build(&dir, 42).expect("monta");
    let err = p.run_frames(1).unwrap_err();
    assert!(
        format!("{err}").contains("orçamento") || err.msg.contains("orçamento"),
        "loop infinito deve morrer no orçamento, veio: {err}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn error_after_partial_mutation_stops_the_tick() {
    let dir = scratch("partial");
    write_project(
        &dir,
        "name = \"partial\"\n",
        "function update(dt)\n  ent.new(\"meio\")\n  error(\"boom\")\nend\n",
    );
    let mut p = Project::build(&dir, 42).expect("monta");
    let err = p.run_frames(1).unwrap_err();
    assert_eq!(err.code, "E050");
    // O tick que falhou não avança o tempo — nada de continuar silenciosamente.
    assert_eq!(p.clock.frame, 0);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn draw_cannot_mutate_world() {
    let dir = scratch("drawmut");
    write_project(
        &dir,
        "name = \"drawmut\"\n",
        "function update(dt) end\nfunction draw() ent.new(\"x\") end\n",
    );
    let mut p = Project::build(&dir, 42).expect("monta");
    let err = p.run_frames(1).unwrap_err();
    assert_eq!(err.code, "E016");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn tick_trace_shows_fixed_phase_order() {
    let mut p = Project::build(&hello(), 42).expect("hello monta");
    p.run_frames(2).expect("run");
    assert_eq!(
        p.trace,
        vec!["barrier", "update", "draw", "raster", "hash"],
        "ordem das fases é parte do contrato (§3.2)"
    );
}

fn build_err(dir: &Path) -> api::EngineError {
    match Project::build(dir, 42) {
        Ok(_) => panic!("projeto deveria falhar ao montar"),
        Err(e) => e.into_engine(),
    }
}

#[test]
fn require_outside_project_is_blocked() {
    let dir = scratch("require");
    write_project(
        &dir,
        "name = \"require\"\n",
        "local m = require(\"outro_modulo\")\n",
    );
    let err = build_err(&dir);
    assert_eq!(err.code, "E050", "sandbox não permite require: {err}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn path_escape_is_rejected() {
    let dir = scratch("escape");
    write_project(&dir, "name = \"escape\"\nscene = \"../fora.toml\"\n", "");
    let err = build_err(&dir);
    assert_eq!(err.code, "E023");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn pending_kill_cancels_spawn_from_script() {
    let dir = scratch("killpend");
    write_project(
        &dir,
        "name = \"killpend\"\n",
        "function load()\n  local e = ent.new(\"spark\")\n  ent.kill(e)\nend\n",
    );
    let mut p = Project::build(&dir, 42).expect("monta");
    p.run_frames(1).expect("run");
    let world = p.world.lock().unwrap();
    assert!(
        world.by_tag("spark").is_empty(),
        "spawn pendente foi cancelado"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn placeholder_does_not_multiply_across_frames() {
    let mut p = Project::build(&hello(), 42).expect("hello monta");
    p.run_frames(120).expect("run");
    // 2 assets ausentes na cena (player_idle, coin) → 2 placeholders, estáveis.
    let sprites = p.registry.lock().unwrap().sprites().len();
    assert_eq!(sprites, 2, "placeholders não podem ser recriados por frame");
}

#[test]
fn screenshot_png_dimensions_and_hash_fixture() {
    let mut p = Project::build(&hello(), 42).expect("hello monta");
    p.run_frames(60).expect("run");
    let out = std::env::temp_dir().join(format!("genflust-shot-{}.png", std::process::id()));
    p.save_png(&out).unwrap();

    // dimensões exatas
    let file = std::io::BufReader::new(std::fs::File::open(&out).unwrap());
    let info = png::Decoder::new(file).read_info().unwrap().info().clone();
    assert_eq!((info.width, info.height), (320, 180));

    // fixture versionada: hash do frame para seed 42 e 60 frames
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hello_f60_seed42.txt");
    if std::env::var("GEN_FIXTURE").is_ok() {
        std::fs::create_dir_all(fixture.parent().unwrap()).unwrap();
        std::fs::write(
            &fixture,
            format!("hello_f60_seed42={:016x}\n", p.frame_hash),
        )
        .unwrap();
        eprintln!("fixture regerada em {}", fixture.display());
        return;
    }
    let expected: u64 = std::fs::read_to_string(&fixture)
        .expect("fixture hello_f60_seed42.txt (regenere com GEN_FIXTURE=1)")
        .trim()
        .split_once('=')
        .and_then(|(_, v)| u64::from_str_radix(v.trim(), 16).ok())
        .expect("formato nome=hex");
    assert_eq!(expected, p.frame_hash, "golden do exemplo hello divergiu");
    std::fs::remove_file(&out).ok();
}
