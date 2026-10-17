//! xtask — ferramentas de desenvolvimento e CI (AGENTS.md §9, §13).
//! Uso: `cargo xtask <arch-check|token-budget|perf-budget>`.
//!
//! Bootstrap honesto: cobrem o que existe hoje e ganham dentes conforme os
//! crates da engine aparecem (grafo, contratos, métodos por trait).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MAX_LINES: usize = 300; // R1 (violação = falha)
const TARGET_LINES: usize = 200; // R1 (alvo = aviso)
const L0_TOKEN_BUDGET: usize = 3000; // AGENTS.md §6
const SKIP_DIRS: &[&str] = &[".git", ".agents", ".github", "target", "node_modules"];

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("arch-check") => arch_check(),
        Some("token-budget") => token_budget(),
        Some("perf-budget") => perf_budget(args.collect()),
        Some(other) => {
            eprintln!("comando desconhecido: {other}\n");
            usage();
            ExitCode::from(2)
        }
        None => {
            usage();
            ExitCode::from(2)
        }
    }
}

fn usage() {
    eprintln!("uso: cargo xtask <arch-check|token-budget|perf-budget>");
}

/// Raiz do workspace: o diretório que contém o Cargo.toml raiz (pai do xtask).
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask fora do workspace")
        .to_path_buf()
}

/// R1: nenhum arquivo Rust de crate com mais de 300 linhas (alvo 200).
fn arch_check() -> ExitCode {
    let mut violations = Vec::new();
    let mut warnings = Vec::new();
    let mut checked = 0usize;

    let root = root();
    walk_crates(&root, &root, &mut violations, &mut warnings, &mut checked);

    for (file, lines) in &warnings {
        println!("aviso: {file}: {lines} linhas (alvo {TARGET_LINES})");
    }
    if violations.is_empty() {
        println!("arch-check: ok ({checked} arquivos Rust verificados)");
        ExitCode::SUCCESS
    } else {
        for (file, lines) in &violations {
            eprintln!("R1 violada: {file}: {lines} linhas (máx {MAX_LINES})");
        }
        eprintln!("arch-check: falhou ({checked} arquivos verificados)");
        ExitCode::from(1)
    }
}

/// Varre a árvore procurando crates (dir com Cargo.toml) e checa src/**/*.rs.
fn walk_crates(
    dir: &Path,
    root: &Path,
    violations: &mut Vec<(String, usize)>,
    warnings: &mut Vec<(String, usize)>,
    checked: &mut usize,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if SKIP_DIRS.contains(&name) {
            continue;
        }
        if path.join("Cargo.toml").is_file() {
            let src = path.join("src");
            if src.is_dir() {
                walk_rs(&src, root, violations, warnings, checked);
            }
        }
        walk_crates(&path, root, violations, warnings, checked);
    }
}

fn walk_rs(
    dir: &Path,
    root: &Path,
    violations: &mut Vec<(String, usize)>,
    warnings: &mut Vec<(String, usize)>,
    checked: &mut usize,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rs(&path, root, violations, warnings, checked);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let lines = fs::read_to_string(&path)
                .map(|s| s.lines().count())
                .unwrap_or(0);
            *checked += 1;
            let display = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string()
                .replace('\\', "/");
            if lines > MAX_LINES {
                violations.push((display, lines));
            } else if lines > TARGET_LINES {
                warnings.push((display, lines));
            }
        }
    }
}

/// L0 (`llms.txt`): estimativa de tokens = bytes / 4 (sem tokenizer, como
/// ferramentas de estimativa usam). Budget: 3k tokens.
fn token_budget() -> ExitCode {
    let path = root().join("llms.txt");
    if !path.is_file() {
        println!("token-budget: ok — llms.txt ainda não existe (0 tokens)");
        return ExitCode::SUCCESS;
    }
    let bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let tokens = (bytes as f64 * 0.25).ceil() as usize;
    println!("token-budget: llms.txt ≈{tokens} tokens (orçamento {L0_TOKEN_BUDGET})");
    if tokens > L0_TOKEN_BUDGET {
        eprintln!("token-budget: falhou — llms.txt acima do orçamento");
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// Perf-budget: roda em máquina declarada (baseline em `perf/baselines.toml`),
/// nunca em runner compartilhado — variância alta demais para a meta de 5%.
fn perf_budget(args: Vec<String>) -> ExitCode {
    let machine = args
        .iter()
        .position(|a| a == "--machine")
        .and_then(|i| args.get(i + 1))
        .cloned();

    let Some(machine) = machine else {
        eprintln!("perf-budget: declare a máquina: cargo xtask perf-budget --machine <nome>");
        eprintln!("baseline esperada em perf/baselines.toml: [machines.<nome>] headless_fps = N");
        return ExitCode::from(2);
    };

    let baseline = baseline_fps(&machine);
    match baseline {
        Some(fps) => println!("perf-budget: máquina '{machine}', baseline {fps} fps headless"),
        None => println!("perf-budget: máquina '{machine}' sem baseline em perf/baselines.toml"),
    }
    // Bootstrap: sem `engine bench` no workspace, não há o que medir.
    println!("perf-budget: ok — nenhum benchmark registrado (engine bench chega na fase 2)");
    ExitCode::SUCCESS
}

/// Lê `perf/baselines.toml` e devolve o headless_fps da máquina, se declarada.
fn baseline_fps(machine: &str) -> Option<u64> {
    let text = fs::read_to_string(root().join("perf").join("baselines.toml")).ok()?;
    let mut current = String::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("[machines.") {
            current = name.strip_suffix(']').unwrap_or(name).to_string();
        } else if current == machine
            && let Some((key, value)) = line.split_once('=')
            && key.trim() == "headless_fps"
        {
            return value.trim().parse().ok();
        }
    }
    None
}
