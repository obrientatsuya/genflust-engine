//! Formatter TOON (ENGINE_VISION2 §4.12): `chave: valor`; listas uniformes
//! como `nome[N]{campos}:` + 1 linha por item. Sempre via encoder (R12).

use api::{Formatter, Value};

pub struct Toon;

impl Formatter for Toon {
    fn name(&self) -> &'static str {
        "toon"
    }

    fn format(&self, title: &str, value: &Value) -> String {
        let mut out = String::new();
        match value {
            Value::Map(items) => {
                if !title.is_empty() {
                    out.push_str(title);
                    out.push_str(":\n");
                }
                for (k, v) in items {
                    out.push_str(&format!("  {k}: {}\n", scalar(v)));
                }
            }
            Value::Table { fields, rows } => {
                if !title.is_empty() {
                    out.push_str(title);
                }
                out.push_str(&format!("[{}]{{{}}}:", rows.len(), fields.join(",")));
                for row in rows {
                    out.push('\n');
                    out.push_str("  ");
                    out.push_str(&row.iter().map(scalar).collect::<Vec<_>>().join(","));
                }
                out.push('\n');
            }
            other => {
                if !title.is_empty() {
                    out.push_str(title);
                    out.push_str(": ");
                }
                out.push_str(&scalar(other));
                out.push('\n');
            }
        }
        out
    }
}

/// Escalar com quoting mínimo (aspas quando há espaço, vírgula ou aspas).
fn scalar(v: &Value) -> String {
    match v {
        Value::Str(s) => {
            let needs_quote = s.is_empty()
                || s.contains(' ')
                || s.contains(',')
                || s.contains('"')
                || s.contains(':');
            if needs_quote {
                format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                s.clone()
            }
        }
        Value::Int(i) => i.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Map(_) | Value::Table { .. } => String::from("(tabela)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_format() {
        let v = Value::table(
            vec!["id".into(), "tag".into()],
            vec![
                vec![Value::i(1), Value::s("player")],
                vec![Value::i(2), Value::s("enemy")],
            ],
        );
        let out = Toon.format("entities", &v);
        assert_eq!(out, "entities[2]{id,tag}:\n  1,player\n  2,enemy\n");
    }

    #[test]
    fn quotes_when_needed() {
        let v = Value::s("com espaço");
        assert_eq!(scalar(&v), "\"com espaço\"");
    }
}
