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

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let mut command = None;
    let mut format = String::from("toon");
    let mut seed = DEFAULT_SEED;
    let mut frames = 1u64;
    let mut out = PathBuf::from("out/screenshot.png");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "run" | "check" | "screenshot" => command = Some(arg),
            "--format" => {
                if let Some(v) = args.next() {
                    format = v;
                }
            }
            "--seed" => {
                if let Some(v) = args.next() {
                    seed = v.parse().unwrap_or(DEFAULT_SEED);
                }
            }
            "--frames" => {
                if let Some(v) = args.next() {
                    frames = v.parse().unwrap_or(1).max(1);
                }
            }
            "--out" => {
                if let Some(v) = args.next() {
                    out = PathBuf::from(v);
                }
            }
            other => {
                eprintln!("flag desconhecida: {other}");
                usage();
                return ExitCode::from(2);
            }
        }
    }

    let Some(cmd) = command else {
        usage();
        return ExitCode::from(2);
    };

    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
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
