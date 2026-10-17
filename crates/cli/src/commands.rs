//! Comandos da fase 1 (ENGINE_VISION2 §5): `run`, `screenshot`, `check`.
//! Silêncio no sucesso (R13); erro = tabela TOON com `code/file/line/msg/fix`.

use std::path::{Path, PathBuf};

use api::{EngineError, Formatter, Value};
use core::DEFAULT_SEED;

use crate::{Project, ProjectError, project::validate_world};

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
    match run(cli, &root) {
        Ok(code) => code,
        Err(e) => crate::report_error(&e, fmt.as_ref()),
    }
}

fn run(cli: Cli, root: &Path) -> Result<i32, EngineError> {
    let fmt = crate::formatter(&cli.format);
    match cli.command {
        Command::Check => check(root, fmt.as_ref()),
        Command::Run | Command::Screenshot => {
            let mut project = Project::build(root, cli.seed).map_err(ProjectError::into_engine)?;
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
    let mut missing_assets: Vec<String> = Vec::new();

    // Config e cena usam o mesmo carregador/validador do runtime (R15).
    let project = match Project::build(root, DEFAULT_SEED) {
        Ok(p) => Some(p),
        Err(crate::ProjectError::Missing(e)) => {
            errors.push(e);
            None
        }
    };

    if let Some(project) = &project {
        errors.extend(validate_world(&project.world.lock().expect("poisoned")));
        // Referências de sprite da cena sem arte = placeholder + aviso.
        let registry = project.registry.lock().expect("poisoned");
        let world = project.world.lock().expect("poisoned");
        for e in world.active() {
            if let Ok(sprite) = world.sprite(e)
                && registry.resolve(&sprite.asset).is_none()
            {
                missing_assets.push(sprite.asset.clone());
            }
        }
    }

    if !errors.is_empty() {
        let rows: Vec<Vec<Value>> = errors
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
        print!(
            "{}",
            fmt.format(
                "errors",
                &Value::table(
                    vec![
                        "code".into(),
                        "file".into(),
                        "line".into(),
                        "msg".into(),
                        "fix".into()
                    ],
                    rows,
                )
            )
        );
        return Ok(1);
    }

    if !missing_assets.is_empty() {
        missing_assets.sort();
        missing_assets.dedup();
        let rows: Vec<Vec<Value>> = missing_assets
            .into_iter()
            .map(|n| vec![Value::s(n)])
            .collect();
        print!(
            "{}",
            fmt.format("missing_assets", &Value::table(vec!["name".into()], rows),)
        );
    }
    println!("check: ok");
    Ok(0)
}
