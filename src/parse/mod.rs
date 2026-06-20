//! Lexing and parsing of `.alg` source into a surface syntax tree.

pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::Module;

use crate::diagnostics::Diagnostic;

/// Lex and parse `source` into a [`Module`].
pub fn parse(source: &str) -> Result<Module, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    parser::parse_module(&tokens, source)
}
