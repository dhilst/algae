//! Source spans and diagnostics with deterministic rendering.

use std::fmt;
use std::path::{Path, PathBuf};

/// A byte range into a source file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    /// A span covering both inputs.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => f.write_str("error"),
            Severity::Warning => f.write_str("warning"),
        }
    }
}

/// A machine-applicable suggestion attached to a diagnostic: replace the text
/// in `span` with `replacement`. `title` labels it in an editor's UI (e.g. a
/// CodeMirror autocomplete completion). Carried as structured data — the free
/// text describing the fix still lives in the diagnostic's `message`.
#[derive(Clone, Debug)]
pub struct Fix {
    pub title: String,
    pub replacement: String,
    pub span: Span,
}

/// A single diagnostic message, optionally anchored to a span in a file.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub file: Option<PathBuf>,
    pub span: Option<Span>,
    /// Machine-applicable suggestions. Empty for most diagnostics.
    pub fixes: Vec<Fix>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            message: message.into(),
            file: None,
            span: None,
            fixes: Vec::new(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Warning,
            message: message.into(),
            file: None,
            span: None,
            fixes: Vec::new(),
        }
    }

    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Diagnostic {
        self.file = Some(file.into());
        self
    }

    pub fn with_span(mut self, span: Span) -> Diagnostic {
        self.span = Some(span);
        self
    }

    pub fn with_fix(mut self, fix: Fix) -> Diagnostic {
        self.fixes.push(fix);
        self
    }

    pub fn with_fixes(mut self, fixes: Vec<Fix>) -> Diagnostic {
        self.fixes = fixes;
        self
    }

    /// Render the diagnostic. If `source` is provided and the diagnostic has a
    /// span, a line:col location and the offending line are shown.
    pub fn render(&self, source: Option<&str>) -> String {
        let mut out = String::new();
        let loc = match (self.file.as_deref(), self.span, source) {
            (file, Some(span), Some(src)) => {
                let (line, col) = line_col(src, span.start);
                let path = file.map(display_path).unwrap_or_default();
                format!("{path}:{line}:{col}: ")
            }
            (Some(file), _, _) => format!("{}: ", display_path(file)),
            _ => String::new(),
        };
        out.push_str(&format!("{loc}{}: {}", self.severity, self.message));
        if let (Some(span), Some(src)) = (self.span, source) {
            if let Some(snippet) = line_snippet(src, span) {
                out.push('\n');
                out.push_str(&snippet);
            }
        }
        out
    }
}

fn display_path(p: &Path) -> String {
    p.display().to_string()
}

/// 1-based line and column for a byte offset.
pub fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(src.len());
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in src.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn line_snippet(src: &str, span: Span) -> Option<String> {
    let start = span.start.min(src.len());
    let line_start = src[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = src[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(src.len());
    let line = &src[line_start..line_end];
    let caret_col = src[line_start..start].chars().count();
    let caret_len = src[start..span.end.min(line_end).max(start)]
        .chars()
        .count()
        .max(1);
    let mut s = String::new();
    s.push_str("  ");
    s.push_str(line);
    s.push('\n');
    s.push_str("  ");
    s.push_str(&" ".repeat(caret_col));
    s.push_str(&"^".repeat(caret_len));
    Some(s)
}

/// The standard result type for fallible phases that produce a batch of errors.
pub type DResult<T> = Result<T, Vec<Diagnostic>>;
