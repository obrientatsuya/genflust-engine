//! Contrato de determinismo (R7 / fase 1): mesma seed → mesmo hash de frame;
//! entidades da cena montadas antes do primeiro tick; handle obsoleto nunca
//! aceito (R10).

use cli::Project;

fn hello() -> std::path::PathBuf {
    // crates/cli → raiz do workspace → examples/hello
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("cli deve viver em crates/cli")
        .join("examples")
        .join("hello")
}

#[test]
fn same_seed_same_frame_hash() {
    let mut a = Project::build(&hello(), 42).expect("projeto hello deve montar");
    let mut b = Project::build(&hello(), 42).expect("projeto hello deve montar");
    a.run_frames(60).expect("run a");
    b.run_frames(60).expect("run b");
    assert_eq!(a.frame_hash, b.frame_hash);
    assert_ne!(a.frame_hash, 0, "frame vazio não pode passar como hash");
}

#[test]
fn different_seed_differs() {
    let mut a = Project::build(&hello(), 42).expect("projeto hello deve montar");
    let mut b = Project::build(&hello(), 7).expect("projeto hello deve montar");
    a.run_frames(60).expect("run a");
    b.run_frames(60).expect("run b");
    assert_ne!(a.frame_hash, b.frame_hash);
}

#[test]
fn scene_entities_ready_before_first_tick() {
    let p = Project::build(&hello(), 42).expect("projeto hello deve montar");
    let world = p.world.lock().unwrap();
    assert!(world.resolve("player").is_some());
    assert!(world.resolve("coin").is_some());
    assert_eq!(world.by_tag("player").len(), 1);
}

#[test]
fn screenshot_png_matches_hash_run() {
    let mut a = Project::build(&hello(), 42).expect("projeto hello deve montar");
    a.run_frames(30).expect("run a");
    let out = std::env::temp_dir().join(format!("genflust-test-{}.png", std::process::id()));
    a.save_png(&out).expect("png");
    assert!(out.exists() && std::fs::metadata(&out).unwrap().len() > 0);
    std::fs::remove_file(&out).ok();
}
