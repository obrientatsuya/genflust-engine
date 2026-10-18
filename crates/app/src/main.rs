//! `engine` — composition root (ENGINE_VISION2 §3.3): o único lugar que
//! conhece impls concretas. Parseia flags e entrega ao CLI.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use core::DEFAULT_SEED;

fn usage() {
    eprintln!(
        "engine <run|check|screenshot> [--frames N] [--seed S] [--out ARQUIVO] [--format toon|json]"
    );
}

/// Uso inválido nunca vira valor silencioso (R18): erro + exit 2.
fn usage_error(msg: &str) -> ExitCode {
    eprintln!("{msg}");
    usage();
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let mut command: Option<String> = None;
    let mut format = String::from("toon");
    let mut seed = DEFAULT_SEED;
    let mut frames = 1u64;
    let mut out = PathBuf::from("out/screenshot.png");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "run" | "check" | "screenshot" => command = Some(arg),
            "--format" => match args.next().as_deref() {
                Some("toon") => format = "toon".into(),
                Some("json") => format = "json".into(),
                Some(other) => return usage_error(&format!("--format inválido: {other}")),
                None => return usage_error("--format exige um valor"),
            },
            "--seed" => match args.next() {
                Some(v) => match v.parse() {
                    Ok(s) => seed = s,
                    Err(_) => return usage_error(&format!("--seed inválido: {v}")),
                },
                None => return usage_error("--seed exige um valor"),
            },
            "--frames" => match args.next() {
                Some(v) => match v.parse::<u64>() {
                    Ok(n) if n >= 1 => frames = n,
                    _ => return usage_error(&format!("--frames inválido: {v}")),
                },
                None => return usage_error("--frames exige um valor"),
            },
            "--out" => match args.next() {
                Some(v) => out = PathBuf::from(v),
                None => return usage_error("--out exige um valor"),
            },
            other => return usage_error(&format!("flag desconhecida: {other}")),
        }
    }

    let Some(cmd) = command else {
        return usage_error("falta o comando");
    };

    let root = match env::current_dir() {
        Ok(p) => p,
        Err(e) => return usage_error(&format!("não consegui obter o diretório atual: {e}")),
    };
    let command = match cmd.as_str() {
        "run" => cli::Command::Run,
        "check" => cli::Command::Check,
        _ => cli::Command::Screenshot,
    };
    let cli = cli::Cli {
        format,
        seed,
        frames,
        out,
        command,
    };
    ExitCode::from(cli::execute(cli, root) as u8)
}
