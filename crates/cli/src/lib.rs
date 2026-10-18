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

/// Tabela `errors[N]{code,file,line,msg,fix}` — formato único de erro (R12/R13).
pub(crate) fn errors_value(rows: Vec<Vec<Value>>) -> Value {
    let fields = ["code", "file", "line", "msg", "fix"]
        .map(str::to_string)
        .to_vec();
    Value::table(fields, rows)
}

/// Erro → saída TOON/JSON e exit code (1 = erro, 0 = ok, 2 = uso).
pub fn report_error(err: &EngineError, fmt: &dyn Formatter) -> i32 {
    let row = vec![
        Value::s(err.code),
        Value::s(&err.file),
        Value::i(err.line as i64),
        Value::s(&err.msg),
        Value::s(&err.fix),
    ];
    eprint!("{}", fmt.format("errors", &errors_value(vec![row])));
    1
}
