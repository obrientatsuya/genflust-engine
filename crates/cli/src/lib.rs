//! CLI (ENGINE_VISION2 §2.4/§4.5): comandos, saída via Formatter, exit
//! codes significativos e erros estruturados.

mod commands;
mod config;
mod project;

pub use commands::{Cli, Command, execute};
pub use config::EngineConfig;
pub use project::Project;

use api::{EngineError, Formatter, Value};

/// Monta o Formatter pedido por `--format` (padrão `toon`).
pub fn formatter(format: &str) -> Box<dyn Formatter> {
    match format {
        "json" => Box::new(fmt_json::Json),
        _ => Box::new(fmt_toon::Toon),
    }
}

/// Erro → saída TOON/JSON e exit code (1 = erro, 0 = ok, 2 = uso).
pub fn report_error(err: &EngineError, fmt: &dyn Formatter) -> i32 {
    let value = Value::table(
        vec![
            "code".into(),
            "file".into(),
            "line".into(),
            "msg".into(),
            "fix".into(),
        ],
        vec![vec![
            Value::s(err.code),
            Value::s(&err.file),
            Value::i(err.line as i64),
            Value::s(&err.msg),
            Value::s(&err.fix),
        ]],
    );
    eprint!("{}", fmt.format("errors", &value));
    1
}
