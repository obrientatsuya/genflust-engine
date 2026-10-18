//! `cargo xtask search <termo...>`: busca determinística no repositório.
//!
//! Retrieval para agentes: devolve só os trechos certos, com arquivo:linha,
//! em saída TOON, com orçamento. Zero dependência, mesma entrada = mesma saída
//! (§4.6 L0/L1; o `engine search` de projeto vem na fase 5).

use std::fs;
use std::path::{Path, PathBuf};

const EXTENSIONS: &[&str] = &["md", "toml", "rs", "luau", "toon", "txt"];
const MAX_RESULTS: usize = 8;
const SNIPPET_CHARS: usize = 140;
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;

struct Hit {
    file: String,
    line: usize,
    score: usize,
    text: String,
}

/// Executa a busca; devolve exit code (0 ok, 2 uso).
pub fn run(terms: &[String], root: &Path) -> std::process::ExitCode {
    if terms.is_empty() {
        eprintln!("uso: cargo xtask search <termo> [<termo>...]");
        return std::process::ExitCode::from(2);
    }
    let needles: Vec<String> = terms.iter().map(|t| t.to_lowercase()).collect();
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files(root, &mut files);
    files.sort();

    let mut hits: Vec<Hit> = Vec::new();
    for file in &files {
        scan_file(file, root, &needles, &mut hits);
    }
    // Ordem estável: score desc, arquivo asc, linha asc.
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.line.cmp(&b.line))
    });
    hits.truncate(MAX_RESULTS);

    let rows: Vec<String> = hits
        .iter()
        .map(|h| {
            format!(
                "  {},{},{},\"{}\"",
                h.file,
                h.line,
                h.score,
                h.text.replace('"', "\\\"")
            )
        })
        .collect();
    println!("search[{}]{{file,line,score,text}}:", rows.len());
    for r in &rows {
        println!("{r}");
    }
    if rows.is_empty() {
        println!("  (nada encontrado; tente outros termos ou leia llms.txt e docs/)");
    }
    std::process::ExitCode::SUCCESS
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    const SKIP: &[&str] = &[".git", "target", "node_modules", ".agents", "out"];
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if path.is_dir() {
            if !SKIP.contains(&name) {
                collect_files(&path, out);
            }
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if !EXTENSIONS.contains(&ext) {
            continue;
        }
        if fs::metadata(&path)
            .map(|m| m.len() <= MAX_FILE_BYTES)
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

fn scan_file(file: &Path, root: &Path, needles: &[String], hits: &mut Vec<Hit>) {
    let text = match fs::read_to_string(file) {
        Ok(t) => t,
        Err(_) => return,
    };
    let rel = file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string()
        .replace('\\', "/");
    // Uma passada por linha: linha que casa vira hit próprio.
    for (i, line) in text.lines().enumerate() {
        let lower = line.to_lowercase();
        let score: usize = needles.iter().map(|n| count_matches(&lower, n)).sum();
        if score == 0 {
            continue;
        }
        let mut snippet: String = line.trim().to_string();
        if snippet.chars().count() > SNIPPET_CHARS {
            snippet = snippet.chars().take(SNIPPET_CHARS).collect::<String>() + "…";
        }
        hits.push(Hit {
            file: rel.clone(),
            line: i + 1,
            score,
            text: snippet,
        });
    }
}

/// Conta ocorrências; termos curtos (<4 chars) exigem limite de palavra
/// (evita "RAG" casar dentro de "amostragem").
fn count_matches(lower: &str, needle: &str) -> usize {
    let short = needle.chars().count() < 4;
    let mut count = 0;
    let mut start = 0;
    while let Some(pos) = lower[start..].find(needle) {
        let abs = start + pos;
        let end = abs + needle.len();
        let before_ok = lower[..abs]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        let after_ok = lower[end..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric());
        if !short || (before_ok && after_ok) {
            count += 1;
        }
        start = end.max(start + 1);
    }
    count
}
