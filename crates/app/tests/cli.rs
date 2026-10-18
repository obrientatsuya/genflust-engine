//! Testes de CLI de ponta a ponta (ENT-01-16-S01/S04, ENT-01-17-S03):
//! stdout/stderr/exit code/artefatos do binário `engine`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn engine() -> &'static str {
    env!("CARGO_BIN_EXE_engine")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "genflust-cli-{}-{name}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn hello() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("app em crates/app")
        .join("examples")
        .join("hello")
}

#[test]
fn check_ok_on_example() {
    let out = Command::new(engine())
        .args(["check"])
        .current_dir(hello())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("check: ok"), "veio: {stdout}");
    assert!(stdout.contains("missing_assets"), "aviso de arte visível");
}

#[test]
fn missing_project_is_structured_error() {
    let dir = scratch("semprojeto");
    let out = Command::new(engine())
        .args(["check"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("E020"), "erro estruturado com código: {err}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn unknown_flag_is_usage_error() {
    let out = Command::new(engine())
        .args(["run", "--bogus"])
        .current_dir(hello())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn corrupt_asset_is_effective_error_not_silence() {
    let dir = scratch("corrupt");
    std::fs::write(dir.join("engine.toml"), "name = \"corrupt\"\n").unwrap();
    std::fs::write(dir.join("main.luau"), "function update(dt) end\n").unwrap();
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("assets/bad.png"), b"nao e png").unwrap();

    let out = Command::new(engine())
        .args(["check"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "PNG corrompido deve falhar o check"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("E100"), "com código e local: {err}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn check_aggregates_all_corrupt_assets() {
    let dir = scratch("corrupt2");
    std::fs::write(dir.join("engine.toml"), "name = \"corrupt2\"\n").unwrap();
    std::fs::write(dir.join("main.luau"), "function update(dt) end\n").unwrap();
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(dir.join("assets/bad1.png"), b"nao e png").unwrap();
    std::fs::write(dir.join("assets/bad2.png"), b"tambem nao").unwrap();

    let out = Command::new(engine())
        .args(["check"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    // O diagnóstico agrega TODOS os arquivos, não só o primeiro (T1).
    assert!(err.contains("errors[2]"), "esperado errors[2]: {err}");
    assert!(
        err.contains("bad1.png") && err.contains("bad2.png"),
        "os dois no diagnóstico: {err}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn script_syntax_error_is_effective_error() {
    let dir = scratch("scripterr");
    std::fs::write(dir.join("engine.toml"), "name = \"scripterr\"\n").unwrap();
    // erro de sintaxe real (end a mais): pega no check
    std::fs::write(dir.join("main.luau"), "function update(dt) end end\n").unwrap();

    let out = Command::new(engine())
        .args(["check"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("E050"), "erro de script apontado: {err}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn script_runtime_error_fails_run() {
    let dir = scratch("scriptrun");
    std::fs::write(dir.join("engine.toml"), "name = \"scriptrun\"\n").unwrap();
    // chamar função inexistente só explode quando o update roda
    std::fs::write(
        dir.join("main.luau"),
        "function update(dt) funcao_que_nao_existe() end\n",
    )
    .unwrap();

    let out = Command::new(engine())
        .args(["run", "--frames", "1"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "run deve falhar, não silenciar");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("E050"), "erro de runtime apontado: {err}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn screenshot_writes_artifacts_where_told() {
    let out_path = std::env::temp_dir()
        .join(format!("genflust-art-{}", std::process::id()))
        .join("shot.png");
    let out = Command::new(engine())
        .args([
            "screenshot",
            "--frames",
            "10",
            "--seed",
            "42",
            "--out",
            out_path.to_str().unwrap(),
        ])
        .current_dir(hello())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out_path.exists(), "artefato no caminho pedido");
    // silêncio no sucesso (R13)
    assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
    std::fs::remove_dir_all(out_path.parent().unwrap()).ok();
}
