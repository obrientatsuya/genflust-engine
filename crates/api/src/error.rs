//! Erro estruturado (R13 / ENGINE_VISION2 §4.5): `code` estável, local, `msg`
//! curta, `fix` de correção e `docs`. Sem stack trace por padrão.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError {
    /// Código estável (ex.: `E012`), nunca muda para o mesmo problema.
    pub code: &'static str,
    /// Arquivo ou contexto onde o erro aconteceu (vazio quando global).
    pub file: String,
    /// Linha, quando conhecida (0 = desconhecida).
    pub line: usize,
    /// Mensagem curta.
    pub msg: String,
    /// Correção sugerida.
    pub fix: String,
    /// Comando de docs relacionado.
    pub docs: String,
}

impl EngineError {
    pub fn new(code: &'static str, msg: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            code,
            file: String::new(),
            line: 0,
            msg: msg.into(),
            fix: fix.into(),
            docs: String::from("engine docs"),
        }
    }

    /// Erro de contrato da engine (uso inválido de API).
    pub fn usage(code: &'static str, msg: impl Into<String>, fix: impl Into<String>) -> Self {
        let mut e = Self::new(code, msg, fix);
        e.file = String::from("cli");
        e
    }

    pub fn at(mut self, file: impl Into<String>, line: usize) -> Self {
        self.file = file.into();
        self.line = line;
        self
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.msg)
    }
}

impl std::error::Error for EngineError {}
