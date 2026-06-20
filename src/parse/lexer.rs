//! Hand-written lexer producing tokens with byte spans.
//!
//! Both ASCII and Unicode spellings of an operator map to the same
//! [`TokenKind`], so downstream code never distinguishes the two. `fmt` uses
//! the token spans to swap glyphs while copying all other bytes verbatim.

use crate::diagnostics::{Diagnostic, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // Names and literals
    Ident(String),
    Number(String),

    // Keywords
    KwImport,
    KwSort,
    KwOp,
    KwAxiom,
    KwRule,
    KwLemma,
    KwTheorem,
    KwProof,
    KwQed,
    KwCase,
    KwTheory,
    KwLaw,
    KwModel,
    KwSatisfies,
    KwIff,
    KwInclude,
    KwForall,
    KwExists,
    KwSt,
    KwAs,
    KwEnd,
    KwSortU, // the kind `Sort`
    KwProp,  // `Prop`
    KwFalse, // `False`
    KwLambda, // `lambda` / `λ`

    // Sequent / logic / type operators (ASCII or Unicode)
    Turnstile, // |-  ⊢
    Arrow,     // ->
    Implies,   // =>  ⇒
    Iff,       // <=> ⇔
    And,       // /\  ∧
    Or,        // \/  ∨
    Not,       // ~   ¬
    Star,      // *   ×
    Bar,       // |   (sum)
    Eq,        // =
    Separator, // ----...  ────...

    // Punctuation
    ColonEq, // :=
    Colon,   // :
    Comma,
    Semi,
    Dot,
    LParen,
    RParen,
    LBrace,
    RBrace,

    // Term infix operators
    Plus,
    Minus,
    Slash,
    EqEq,
    Lt,
    Gt,
    Le,
    Ge,

    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn is(&self, k: &TokenKind) -> bool {
        &self.kind == k
    }
}

/// Tokenize `src`. Whitespace and `#` comments are skipped (their bytes are
/// recoverable from the gaps between token spans, which `fmt` relies on).
pub fn lex(src: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let chars: Vec<(usize, char)> = src.char_indices().collect();
    let n = chars.len();
    let byte_at = |idx: usize| -> usize {
        if idx < n {
            chars[idx].0
        } else {
            src.len()
        }
    };
    let peek = |idx: usize| -> Option<char> { chars.get(idx).map(|x| x.1) };

    let mut tokens: Vec<Token> = Vec::new();
    let mut errors: Vec<Diagnostic> = Vec::new();
    let mut i = 0usize;

    while i < n {
        let (off, c) = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '#' {
            while i < n && chars[i].1 != '\n' {
                i += 1;
            }
            continue;
        }

        // Multi-character / contextual ASCII operators.
        let mut push = |kind: TokenKind, start_idx: usize, len: usize| {
            tokens.push(Token {
                kind,
                span: Span::new(byte_at(start_idx), byte_at(start_idx + len)),
            });
        };

        match c {
            // Single-char Unicode operators.
            '⊢' => { push(TokenKind::Turnstile, i, 1); i += 1; }
            '→' => { push(TokenKind::Arrow, i, 1); i += 1; }
            '×' => { push(TokenKind::Star, i, 1); i += 1; }
            '∧' => { push(TokenKind::And, i, 1); i += 1; }
            '∨' => { push(TokenKind::Or, i, 1); i += 1; }
            '⇒' => { push(TokenKind::Implies, i, 1); i += 1; }
            '⇔' => { push(TokenKind::Iff, i, 1); i += 1; }
            '¬' => { push(TokenKind::Not, i, 1); i += 1; }
            'λ' => { push(TokenKind::KwLambda, i, 1); i += 1; }
            '∀' => { push(TokenKind::KwForall, i, 1); i += 1; }
            '∃' => { push(TokenKind::KwExists, i, 1); i += 1; }
            '─' => {
                let start = i;
                while peek(i) == Some('─') {
                    i += 1;
                }
                push(TokenKind::Separator, start, i - start);
            }

            '(' => { push(TokenKind::LParen, i, 1); i += 1; }
            ')' => { push(TokenKind::RParen, i, 1); i += 1; }
            '{' => { push(TokenKind::LBrace, i, 1); i += 1; }
            '}' => { push(TokenKind::RBrace, i, 1); i += 1; }
            ',' => { push(TokenKind::Comma, i, 1); i += 1; }
            ';' => { push(TokenKind::Semi, i, 1); i += 1; }
            '.' => { push(TokenKind::Dot, i, 1); i += 1; }
            '+' => { push(TokenKind::Plus, i, 1); i += 1; }
            '~' => { push(TokenKind::Not, i, 1); i += 1; }
            '*' => { push(TokenKind::Star, i, 1); i += 1; }

            '/' => {
                if peek(i + 1) == Some('\\') {
                    push(TokenKind::And, i, 2);
                    i += 2;
                } else {
                    push(TokenKind::Slash, i, 1);
                    i += 1;
                }
            }
            '\\' => {
                if peek(i + 1) == Some('/') {
                    push(TokenKind::Or, i, 2);
                    i += 2;
                } else {
                    errors.push(
                        Diagnostic::error("unexpected character `\\`")
                            .with_span(Span::new(off, byte_at(i + 1))),
                    );
                    i += 1;
                }
            }
            '|' => {
                if peek(i + 1) == Some('-') {
                    push(TokenKind::Turnstile, i, 2);
                    i += 2;
                } else {
                    push(TokenKind::Bar, i, 1);
                    i += 1;
                }
            }
            '-' => {
                if peek(i + 1) == Some('>') {
                    push(TokenKind::Arrow, i, 2);
                    i += 2;
                } else {
                    // Count a run of '-' for a separator line.
                    let start = i;
                    let mut run = 0;
                    while peek(start + run) == Some('-') {
                        run += 1;
                    }
                    if run >= 3 {
                        push(TokenKind::Separator, start, run);
                        i += run;
                    } else {
                        push(TokenKind::Minus, i, 1);
                        i += 1;
                    }
                }
            }
            '=' => {
                if peek(i + 1) == Some('>') {
                    push(TokenKind::Implies, i, 2);
                    i += 2;
                } else if peek(i + 1) == Some('=') {
                    push(TokenKind::EqEq, i, 2);
                    i += 2;
                } else {
                    push(TokenKind::Eq, i, 1);
                    i += 1;
                }
            }
            '<' => {
                if peek(i + 1) == Some('=') && peek(i + 2) == Some('>') {
                    push(TokenKind::Iff, i, 3);
                    i += 3;
                } else if peek(i + 1) == Some('=') {
                    push(TokenKind::Le, i, 2);
                    i += 2;
                } else {
                    push(TokenKind::Lt, i, 1);
                    i += 1;
                }
            }
            '>' => {
                if peek(i + 1) == Some('=') {
                    push(TokenKind::Ge, i, 2);
                    i += 2;
                } else {
                    push(TokenKind::Gt, i, 1);
                    i += 1;
                }
            }
            ':' => {
                if peek(i + 1) == Some('=') {
                    push(TokenKind::ColonEq, i, 2);
                    i += 2;
                } else {
                    push(TokenKind::Colon, i, 1);
                    i += 1;
                }
            }

            _ if c == '_' || c.is_ascii_alphabetic() => {
                let start = i;
                i += 1;
                while let Some(ch) = peek(i) {
                    if ch == '_' || ch.is_ascii_alphanumeric() {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let text: String = chars[start..i].iter().map(|x| x.1).collect();
                let kind = keyword(&text).unwrap_or(TokenKind::Ident(text));
                push(kind, start, i - start);
            }
            _ if c.is_ascii_digit() => {
                let start = i;
                i += 1;
                while let Some(ch) = peek(i) {
                    if ch.is_ascii_digit() {
                        i += 1;
                    } else {
                        break;
                    }
                }
                let text: String = chars[start..i].iter().map(|x| x.1).collect();
                push(TokenKind::Number(text), start, i - start);
            }
            _ => {
                errors.push(
                    Diagnostic::error(format!("unexpected character `{c}`"))
                        .with_span(Span::new(off, byte_at(i + 1))),
                );
                i += 1;
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(src.len(), src.len()),
    });

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

fn keyword(s: &str) -> Option<TokenKind> {
    Some(match s {
        "import" => TokenKind::KwImport,
        "sort" => TokenKind::KwSort,
        "op" => TokenKind::KwOp,
        "axiom" => TokenKind::KwAxiom,
        "rule" => TokenKind::KwRule,
        "lemma" => TokenKind::KwLemma,
        "theorem" => TokenKind::KwTheorem,
        "proof" => TokenKind::KwProof,
        "qed" => TokenKind::KwQed,
        "case" => TokenKind::KwCase,
        "theory" => TokenKind::KwTheory,
        "law" => TokenKind::KwLaw,
        "model" => TokenKind::KwModel,
        "satisfies" => TokenKind::KwSatisfies,
        "iff" => TokenKind::KwIff,
        "include" => TokenKind::KwInclude,
        "forall" => TokenKind::KwForall,
        "exists" => TokenKind::KwExists,
        "st" => TokenKind::KwSt,
        "as" => TokenKind::KwAs,
        "end" => TokenKind::KwEnd,
        "Sort" => TokenKind::KwSortU,
        "Prop" => TokenKind::KwProp,
        "False" => TokenKind::KwFalse,
        "lambda" => TokenKind::KwLambda,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        lex(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn basic_tokens() {
        use TokenKind::*;
        assert_eq!(
            kinds("sort Nat : Sort;"),
            vec![KwSort, Ident("Nat".into()), Colon, KwSortU, Semi, Eof]
        );
    }

    #[test]
    fn ascii_and_unicode_operators_match() {
        assert_eq!(kinds("|- x"), kinds("⊢ x"));
        assert_eq!(kinds("a /\\ b"), kinds("a ∧ b"));
        assert_eq!(kinds("a \\/ b"), kinds("a ∨ b"));
        assert_eq!(kinds("a => b"), kinds("a ⇒ b"));
        assert_eq!(kinds("a <=> b"), kinds("a ⇔ b"));
        assert_eq!(kinds("~a"), kinds("¬a"));
        assert_eq!(kinds("A * B"), kinds("A × B"));
        assert_eq!(kinds("lambda x"), kinds("λ x"));
        assert_eq!(kinds("A -> B"), kinds("A → B"));
    }

    #[test]
    fn op_is_a_keyword() {
        use TokenKind::*;
        assert_eq!(kinds("op"), vec![KwOp, Eof]);
    }

    #[test]
    fn operator_disambiguation() {
        use TokenKind::*;
        assert_eq!(kinds("a -> b"), vec![Ident("a".into()), Arrow, Ident("b".into()), Eof]);
        assert_eq!(kinds("a - b"), vec![Ident("a".into()), Minus, Ident("b".into()), Eof]);
        assert_eq!(kinds("------------------------"), vec![Separator, Eof]);
        assert_eq!(kinds("x := y"), vec![Ident("x".into()), ColonEq, Ident("y".into()), Eof]);
        assert_eq!(kinds("x <= y"), vec![Ident("x".into()), Le, Ident("y".into()), Eof]);
        assert_eq!(kinds("x == y"), vec![Ident("x".into()), EqEq, Ident("y".into()), Eof]);
    }

    #[test]
    fn comments_and_qualified() {
        use TokenKind::*;
        assert_eq!(
            kinds("core.reflexivity # comment\n"),
            vec![Ident("core".into()), Dot, Ident("reflexivity".into()), Eof]
        );
    }
}
