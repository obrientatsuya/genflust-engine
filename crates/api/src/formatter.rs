//! Formatter: contrato de saída estruturada (ENGINE_VISION2 §4.12).
//!
//! O core e o CLI montam um `Value` e o `Formatter` (impl em `fmt-toon` ou
//! `fmt-json`) serializa. Nunca se monta TOON à mão (R12).

/// Árvore de valor para saída estruturada.
///
/// Tabela = chaves declaradas uma vez + linhas de valores (formato TOON).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    /// `{ fields, rows }`: cabeçalho `nome[N]{campos}:` + 1 linha por item.
    Table {
        fields: Vec<String>,
        rows: Vec<Vec<Value>>,
    },
    /// Mapa chave→valor (saída de objeto simples).
    Map(Vec<(String, Value)>),
}

impl Value {
    pub fn table(fields: Vec<String>, rows: Vec<Vec<Value>>) -> Self {
        Self::Table { fields, rows }
    }

    pub fn map(items: Vec<(String, Value)>) -> Self {
        Self::Map(items)
    }

    pub fn s(v: impl Into<String>) -> Self {
        Self::Str(v.into())
    }

    pub fn i(v: i64) -> Self {
        Self::Int(v)
    }
}

/// Trait de fronteira para formatos de saída (`toon` padrão, `json`, `text`).
pub trait Formatter {
    fn name(&self) -> &'static str;

    /// Serializa um valor raiz. `title` nomeia a saída (ex.: `check`, `error`).
    fn format(&self, title: &str, value: &Value) -> String;
}
