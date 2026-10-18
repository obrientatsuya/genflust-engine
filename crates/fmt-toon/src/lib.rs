//! Formatter TOON (ENGINE_VISION2 §4.12): `chave: valor`; listas uniformes
//! como `nome[N]{campos}:` + 1 linha por item; aninhamento por indentação.
//! Sempre via encoder (R12).

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
                    for (k, v) in items {
                        render_nested(&mut out, 1, k, v);
                    }
                } else {
                    for (k, v) in items {
                        render_nested(&mut out, 0, k, v);
                    }
                }
            }
            Value::Table { .. } => {
                if title.is_empty() {
                    render_table(&mut out, 0, "", value);
                } else {
                    render_table(&mut out, 0, title, value);
                }
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

/// Renderiza um par chave→valor em `indent` níveis (2 espaços cada).
fn render_nested(out: &mut String, indent: usize, key: &str, value: &Value) {
    match value {
        Value::Table { .. } => render_table(out, indent, key, value),
        Value::Map(items) => {
            pad(out, indent);
            out.push_str(key);
            out.push_str(":\n");
            for (k, v) in items {
                render_nested(out, indent + 1, k, v);
            }
        }
        scalar_value => {
            pad(out, indent);
            out.push_str(key);
            out.push_str(": ");
            out.push_str(&scalar(scalar_value));
            out.push('\n');
        }
    }
}

fn render_table(out: &mut String, indent: usize, key: &str, value: &Value) {
    let Value::Table { fields, rows } = value else {
        return;
    };
    pad(out, indent);
    if !key.is_empty() {
        out.push_str(key);
    }
    out.push_str(&format!("[{}]{{{}}}:", rows.len(), fields.join(",")));
    for row in rows {
        out.push('\n');
        pad(out, indent + 1);
        out.push_str(&row.iter().map(scalar).collect::<Vec<_>>().join(","));
    }
    out.push('\n');
}

fn pad(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

/// Escalar com quoting mínimo (aspas quando há espaço, vírgula, aspas ou `:`).
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
    fn nested_table_inside_map() {
        let v = Value::map(vec![
            ("frame".into(), Value::i(120)),
            (
                "entities".into(),
                Value::table(vec!["id".into()], vec![vec![Value::i(1)]]),
            ),
        ]);
        let out = Toon.format("state", &v);
        assert_eq!(out, "state:\n  frame: 120\n  entities[1]{id}:\n    1\n");
    }

    #[test]
    fn quotes_when_needed() {
        let v = Value::s("com espaço");
        assert_eq!(scalar(&v), "\"com espaço\"");
    }
}
