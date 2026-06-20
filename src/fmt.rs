//! `fmt`: whitespace-preserving operator-glyph normalization.
//!
//! The formatter is purely token-level: it copies the source verbatim and only
//! rewrites operator tokens that have an ASCII/Unicode pair, so all whitespace,
//! comments and layout are preserved exactly. By default ASCII glyphs become
//! Unicode; with `ascii = true` the reverse.

use crate::diagnostics::Diagnostic;
use crate::parse::lexer::{lex, TokenKind};

/// Return the target spelling for a swappable token, or `None` to keep the
/// original bytes. `original` is the source slice for the token (needed to size
/// separator lines).
fn glyph(kind: &TokenKind, ascii: bool, original: &str) -> Option<String> {
    let pair = match kind {
        TokenKind::Turnstile => ("|-", "⊢"),
        TokenKind::Star => ("*", "×"),
        TokenKind::Implies => ("=>", "⇒"),
        TokenKind::Iff => ("<=>", "⇔"),
        TokenKind::And => ("/\\", "∧"),
        TokenKind::Or => ("\\/", "∨"),
        TokenKind::Not => ("~", "¬"),
        TokenKind::KwLambda => ("lambda", "λ"),
        TokenKind::Separator => {
            let n = original.chars().count();
            let ch = if ascii { '-' } else { '─' };
            return Some(std::iter::repeat(ch).take(n).collect());
        }
        _ => return None,
    };
    Some(if ascii { pair.0.to_string() } else { pair.1.to_string() })
}

/// Format `source`, returning the rewritten text.
pub fn format_source(source: &str, ascii: bool) -> Result<String, Vec<Diagnostic>> {
    let tokens = lex(source)?;
    let mut out = String::with_capacity(source.len());
    let mut pos = 0usize;
    for t in &tokens {
        if t.kind == TokenKind::Eof {
            break;
        }
        // Copy the gap (whitespace/comments) before this token verbatim.
        out.push_str(&source[pos..t.span.start]);
        let original = &source[t.span.start..t.span.end];
        match glyph(&t.kind, ascii, original) {
            Some(s) => out.push_str(&s),
            None => out.push_str(original),
        }
        pos = t.span.end;
    }
    out.push_str(&source[pos..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_to_unicode_preserves_layout() {
        let src = "axiom r(T : Sort, x : T)\n  |- x = x;\n";
        let got = format_source(src, false).unwrap();
        assert_eq!(got, "axiom r(T : Sort, x : T)\n  ⊢ x = x;\n");
    }

    #[test]
    fn unicode_to_ascii_roundtrips() {
        let uni = "lemma l |- P ∧ Q ⇒ R;\n";
        let asc = format_source(uni, true).unwrap();
        assert_eq!(asc, "lemma l |- P /\\ Q => R;\n");
        // ASCII back to Unicode returns the original glyphs.
        assert_eq!(format_source(&asc, false).unwrap(), "lemma l ⊢ P ∧ Q ⇒ R;\n");
    }

    #[test]
    fn separator_length_preserved() {
        let src = "----------\n";
        let got = format_source(src, false).unwrap();
        assert_eq!(got, "──────────\n");
        assert_eq!(got.chars().filter(|&c| c == '─').count(), 10);
    }

    #[test]
    fn comments_and_whitespace_untouched() {
        let src = "# a comment with => inside\nsort  Nat   :   Sort ;\n";
        let got = format_source(src, false).unwrap();
        assert_eq!(got, src);
    }
}
