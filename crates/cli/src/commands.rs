//! Comandos da fase 1 (ENGINE_VISION2 §5): `run`, `screenshot`, `check`.
//! Silêncio no sucesso (R13); erro = tabela TOON com `code/file/line/msg/fix`.

use std::path::{Path, PathBuf};

use api::{EngineError, Formatter, Value};
use core::DEFAULT_SEED;

use crate::{Project, project::validate_world};

pub struct Cli {
    pub format: String,
    pub seed: u64,
    pub frames: u64,
    pub out: PathBuf,
    pub command: Command,
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Run,
    Screenshot,
    Check,
}

/// Executa o comando parseado; devolve o exit code (0 ok, 1 erro, 2 uso).
pub fn execute(cli: Cli, root: PathBuf) -> i32 {
    let fmt = crate::formatter(&cli.format);
    match run(cli, &root, fmt.as_ref()) {
        Ok(code) => code,
        Err(e) => crate::report_error(&e, fmt.as_ref()),
    }
}

fn run(cli: Cli, root: &Path, fmt: &dyn Formatter) -> Result<i32, EngineError> {
    match cli.command {
        Command::Check => check(root, fmt),
        Command::Run | Command::Screenshot => {
            let mut project = Project::build(root, cli.seed)?;
            project.run_frames(cli.frames)?;
            if matches!(cli.command, Command::Screenshot) {
                project.save_png(&cli.out)?;
            }
            Ok(0)
        }
    }
}

/// `engine check`: config + cena + script + assets; falta de arte é aviso,
/// nunca erro (placeholders são primeira classe, §4.8).
fn check(root: &Path, fmt: &dyn Formatter) -> Result<i32, EngineError> {
    let mut errors: Vec<EngineError> = Vec::new();
    let mut missing: Vec<String> = Vec::new();

    // Config e cena usam o mesmo carregador/validador do runtime (R15).
    match Project::build(root, DEFAULT_SEED) {
        Ok(project) => {
            errors.extend(validate_world(&project.world.lock().expect("poisoned")));
            missing = missing_assets(&project);
        }
        Err(e) => errors.push(e),
    }

    if !errors.is_empty() {
        let rows = errors
            .iter()
            .map(|e| {
                vec![
                    Value::s(e.code),
                    Value::s(&e.file),
                    Value::i(e.line as i64),
                    Value::s(&e.msg),
                    Value::s(&e.fix),
                ]
            })
            .collect();
        // Erros vão para stderr (R13); avisos e "ok" ficam no stdout.
        eprint!("{}", fmt.format("errors", &errors_value(rows)));
        return Ok(1);
    }

    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        let rows = missing.into_iter().map(|n| vec![Value::s(n)]).collect();
        print!(
            "{}",
            fmt.format("missing_assets", &Value::table(vec!["name".into()], rows))
        );
    }
    println!("check: ok");
    Ok(0)
}

/// Nomes de sprite da cena sem arte (viram placeholder, §4.8).
fn missing_assets(project: &Project) -> Vec<String> {
    let registry = project.registry.lock().expect("poisoned");
    let world = project.world.lock().expect("poisoned");
    world
        .active()
        .into_iter()
        .filter_map(|e| world.sprite(e).ok())
        .filter(|s| registry.resolve(&s.asset).is_none())
        .map(|s| s.asset.clone())
        .collect()
}

fn errors_value(rows: Vec<Vec<Value>>) -> Value {
    let fields = ["code", "file", "line", "msg", "fix"]
        .map(str::to_string)
        .to_vec();
    Value::table(fields, rows)
}
