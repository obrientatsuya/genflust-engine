//! Roundtrip: mesma estrutura decodifica equivalente nos dois formatos
//! (ENT-01-16-S03). O TOON de teste é lido por um parser mínimo do próprio
//! formato que o encoder emite.

use api::Value;

/// Parser mínimo do TOON emitido pelo encoder (mapas, tabelas, escalares).
fn parse_toon(text: &str) -> serde_json::Value {
    let mut lines = text.lines().peekable();
    parse_block(&mut lines, 0)
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn parse_block<'a, I>(lines: &mut std::iter::Peekable<I>, indent: usize) -> serde_json::Value
where
    I: Iterator<Item = &'a str>,
{
    // Decide entre mapa e tabela pela primeira linha do bloco.
    let mut map = serde_json::Map::new();
    while let Some(&line) = lines.peek() {
        if line.trim().is_empty() {
            lines.next();
            continue;
        }
        let cur = indent_of(line);
        if cur < indent {
            break;
        }
        if cur > indent {
            // conteúdo mais fundo é consumido pelo dono; segurança
            lines.next();
            continue;
        }
        let line = lines.next().unwrap().trim().to_string();
        if let Some(header) = line.strip_suffix(':') {
            if let Some(open) = header.find('[')
                && let Some(close) = header.find(']')
                && let Ok(n) = header[open + 1..close].parse::<usize>()
            {
                // tabela: nome[N]{campos}:
                let name = &header[..open];
                let fields: Vec<String> = header[close + 1..]
                    .trim()
                    .trim_start_matches('{')
                    .trim_end_matches('}')
                    .split(',')
                    .filter(|f| !f.is_empty())
                    .map(str::to_string)
                    .collect();
                let mut rows = Vec::new();
                for _ in 0..n {
                    if let Some(row) = lines.next() {
                        let cells = parse_row(row.trim());
                        let obj: serde_json::Map<String, serde_json::Value> =
                            fields.iter().cloned().zip(cells).collect();
                        rows.push(serde_json::Value::Object(obj));
                    }
                }
                map.insert(name.to_string(), serde_json::Value::Array(rows));
                continue;
            }
            // mapa aninhado: "chave:" + bloco indentado
            let child = parse_block(lines, indent + 2);
            map.insert(header.to_string(), child);
            continue;
        }
        if let Some((k, v)) = line.split_once(": ") {
            map.insert(k.to_string(), parse_scalar(v));
        }
    }
    serde_json::Value::Object(map)
}

fn parse_row(line: &str) -> Vec<serde_json::Value> {
    line.split(',').map(parse_scalar).collect()
}

fn parse_scalar(s: &str) -> serde_json::Value {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix('"').and_then(|x| x.strip_suffix('"')) {
        return serde_json::Value::String(inner.replace("\\\"", "\"").replace("\\\\", "\\"));
    }
    if let Ok(i) = s.parse::<i64>() {
        return serde_json::Value::Number(i.into());
    }
    match s {
        "true" => serde_json::Value::Bool(true),
        "false" => serde_json::Value::Bool(false),
        other => serde_json::Value::String(other.to_string()),
    }
}

#[test]
fn same_value_decodes_equivalent_in_both_formats() {
    let value = Value::map(vec![
        ("frame".into(), Value::i(120)),
        ("ok".into(), Value::Bool(true)),
        ("nota".into(), Value::s("com espaço e vírgula, ok")),
        (
            "entities".into(),
            Value::table(
                vec!["id".into(), "tag".into()],
                vec![
                    vec![Value::i(1), Value::s("player")],
                    vec![Value::i(2), Value::s("enemy")],
                ],
            ),
        ),
    ]);

    let toon = cli::formatter("toon").format("state", &value);
    let json = cli::formatter("json").format("state", &value);

    // O título ("state") é cabeçalho nos dois: compare o conteúdo por trás dele.
    let from_toon = parse_toon(&toon)
        .get("state")
        .cloned()
        .expect("bloco state no TOON");
    let from_json: serde_json::Value = serde_json::from_str(&json).unwrap();
    let json_inner = from_json.get("state").unwrap().clone();

    assert_eq!(from_toon, json_inner, "TOON:\n{toon}\nJSON:\n{json}");
}
