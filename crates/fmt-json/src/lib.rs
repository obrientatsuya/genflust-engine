//! Formatter JSON: interop para CI e ferramentas externas (ENGINE_VISION2
//! §4.12 — `--format json`). TOON continua sendo o padrão.

use api::{Formatter, Value};

pub struct Json;

impl Formatter for Json {
    fn name(&self) -> &'static str {
        "json"
    }

    fn format(&self, title: &str, value: &Value) -> String {
        let mut root = serde_json::Map::new();
        root.insert(title.to_string(), to_json(value));
        serde_json::to_string(&root).unwrap_or_else(|_| "{}".into())
    }
}

fn to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Str(s) => serde_json::Value::String(s.clone()),
        Value::Int(i) => serde_json::Value::Number((*i).into()),
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Map(items) => {
            serde_json::Value::Object(items.iter().map(|(k, v)| (k.clone(), to_json(v))).collect())
        }
        Value::Table { fields, rows } => serde_json::Value::Array(
            rows.iter()
                .map(|row| {
                    serde_json::Value::Object(
                        fields
                            .iter()
                            .zip(row.iter())
                            .map(|(k, v)| (k.clone(), to_json(v)))
                            .collect(),
                    )
                })
                .collect(),
        ),
    }
}
