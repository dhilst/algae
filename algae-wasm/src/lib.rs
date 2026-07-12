//! WebAssembly wrapper around `algae-kernel`.
//!
//! The kernel is environment-free (parse → elaborate → check, no I/O), which
//! makes it portable to `wasm32-unknown-unknown`. This crate adds the small
//! amount of glue a browser needs:
//!
//! * a [`BundledResolver`] that satisfies `import`s from the standard library,
//!   whose sources are embedded at compile time (there is no filesystem in the
//!   browser);
//! * serializable diagnostic / result DTOs, since the kernel's `Diagnostic` is
//!   deliberately not `Serialize`;
//! * `#[wasm_bindgen]` entry points ([`check`] and [`format`]) that hand plain
//!   JS values back to the CodeMirror editor. Syntax highlighting is done
//!   client-side by a CodeMirror `StreamLanguage`, so no tokenizer is exported.

use algae_kernel::diagnostics::{line_col, Diagnostic, Fix, Severity};
use algae_kernel::elaborate::proof::{elaborate_unit, SourceResolver};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// The Algae v2 standard library (`algae/stdlib/v1`), embedded so that `import`
/// statements resolve without a filesystem. Keyed by module name (the file stem).
const STDLIB: &[(&str, &str)] = &[
    ("adt", include_str!("../../algae/stdlib/v1/adt.alg")),
    ("core", include_str!("../../algae/stdlib/v1/core.alg")),
    ("group", include_str!("../../algae/stdlib/v1/group.alg")),
    ("list", include_str!("../../algae/stdlib/v1/list.alg")),
    ("monad", include_str!("../../algae/stdlib/v1/monad.alg")),
    ("nat", include_str!("../../algae/stdlib/v1/nat.alg")),
    ("option", include_str!("../../algae/stdlib/v1/option.alg")),
    ("result", include_str!("../../algae/stdlib/v1/result.alg")),
];

/// Resolves `import`s against the embedded standard library, plus any extra
/// modules the caller supplies (e.g. sibling snippets on a docs page).
struct BundledResolver {
    extra: Vec<(String, String)>,
}

impl BundledResolver {
    fn new(extra: Vec<(String, String)>) -> BundledResolver {
        BundledResolver { extra }
    }
}

impl SourceResolver for BundledResolver {
    fn resolve(&self, module: &str) -> Result<String, String> {
        if let Some((_, src)) = self.extra.iter().find(|(name, _)| name == module) {
            return Ok(src.clone());
        }
        if let Some((_, src)) = STDLIB.iter().find(|(name, _)| *name == module) {
            return Ok((*src).to_string());
        }
        Err(format!("unknown module `{module}`"))
    }
}

/// A machine-applicable fix flattened for JavaScript: replace the text in the
/// span with `replacement`. Spans are given both as 0-based byte offsets and as
/// 1-based line/col (the editor addresses positions by line/col for Unicode
/// safety, matching `JsDiag`).
#[derive(Serialize)]
struct JsFix {
    title: String,
    replacement: String,
    start: usize,
    end: usize,
    line: usize,
    col: usize,
    end_line: usize,
    end_col: usize,
}

impl JsFix {
    fn from(source: &str, f: &Fix) -> JsFix {
        let (line, col) = line_col(source, f.span.start);
        let (end_line, end_col) = line_col(source, f.span.end);
        JsFix {
            title: f.title.clone(),
            replacement: f.replacement.clone(),
            start: f.span.start,
            end: f.span.end,
            line,
            col,
            end_line,
            end_col,
        }
    }
}

/// A diagnostic flattened for JavaScript. Spans are 0-based byte offsets into
/// the source (CodeMirror's native addressing); line/column are 1-based for
/// display.
#[derive(Serialize)]
struct JsDiag {
    severity: String,
    message: String,
    /// Byte offset of the span start, or 0 when the diagnostic has no span.
    start: usize,
    /// Byte offset of the span end, or 0 when the diagnostic has no span.
    end: usize,
    line: usize,
    col: usize,
    end_line: usize,
    end_col: usize,
    /// Whether the diagnostic carried a source span at all.
    has_span: bool,
    /// Machine-applicable fix suggestions, surfaced by the editor as
    /// Ctrl-Space autocomplete completions. Empty for most diagnostics.
    fixes: Vec<JsFix>,
}

impl JsDiag {
    fn from(source: &str, d: &Diagnostic) -> JsDiag {
        let severity = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
        .to_string();
        let fixes: Vec<JsFix> = d.fixes.iter().map(|f| JsFix::from(source, f)).collect();
        match d.span {
            Some(span) => {
                let (line, col) = line_col(source, span.start);
                let (end_line, end_col) = line_col(source, span.end);
                JsDiag {
                    severity,
                    message: d.message.clone(),
                    start: span.start,
                    end: span.end,
                    line,
                    col,
                    end_line,
                    end_col,
                    has_span: true,
                    fixes,
                }
            }
            None => JsDiag {
                severity,
                message: d.message.clone(),
                start: 0,
                end: 0,
                line: 1,
                col: 1,
                end_line: 1,
                end_col: 1,
                has_span: false,
                fixes,
            },
        }
    }
}

/// The outcome of a full proof check.
#[derive(Serialize)]
struct CheckResult {
    /// True when there are no error-severity diagnostics.
    ok: bool,
    diagnostics: Vec<JsDiag>,
    /// Number of proof obligations discovered by elaboration.
    obligations: usize,
    /// Number of obligations left admitted (`wip`).
    wip: usize,
}

/// Run elaboration and proof checking, returning a structured result.
///
/// Mirrors `algae_kernel::verify_source` but also reports the obligation and
/// `wip` counts so the editor can print "✓ checked N obligation(s)". `extra` is
/// an optional list of `[name, source]` pairs made available to `import`.
fn run_check(source: &str, module_name: &str, extra: Vec<(String, String)>) -> CheckResult {
    let resolver = BundledResolver::new(extra);
    let unit = match elaborate_unit(source, module_name, &resolver, true) {
        Ok(u) => u,
        Err(diags) => {
            let diagnostics: Vec<JsDiag> = diags.iter().map(|d| JsDiag::from(source, d)).collect();
            return CheckResult {
                ok: false,
                diagnostics,
                obligations: 0,
                wip: 0,
            };
        }
    };

    let mut errors: Vec<Diagnostic> = Vec::new();
    for ob in &unit.obligations {
        errors.extend(algae_kernel::core::check::check(
            &ob.root,
            &ob.label,
            &unit.rewrite,
        ));
    }
    let wip = unit.obligations.iter().filter(|o| o.wip).count();
    // Warnings are reported but do not affect `ok`.
    let ok = errors.is_empty();
    let diagnostics: Vec<JsDiag> = unit
        .warnings
        .iter()
        .chain(errors.iter())
        .map(|d| JsDiag::from(source, d))
        .collect();
    CheckResult {
        ok,
        diagnostics,
        obligations: unit.obligations.len(),
        wip,
    }
}

/// Convert an `extra` JS array of `[name, source]` string pairs into owned Rust
/// pairs. A missing/undefined value yields no extras.
fn parse_extra(extra: JsValue) -> Vec<(String, String)> {
    if extra.is_undefined() || extra.is_null() {
        return Vec::new();
    }
    serde_wasm_bindgen::from_value::<Vec<(String, String)>>(extra).unwrap_or_default()
}

/// Proof-check `source` as module `module_name`.
///
/// Returns a `CheckResult` object: `{ ok, diagnostics: [{severity, message,
/// start, end, line, col, end_line, end_col, has_span, fixes}], obligations,
/// wip }`, where each `fixes` entry is `{ title, replacement, start, end, line,
/// col, end_line, end_col }` — a machine-applicable suggestion the editor
/// surfaces as an autocomplete completion. An empty `diagnostics` array with
/// `ok: true` means success.
#[wasm_bindgen]
pub fn check(source: &str, module_name: &str, extra: JsValue) -> Result<JsValue, JsValue> {
    let result = run_check(source, module_name, parse_extra(extra));
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// The result of formatting: either the rewritten `text`, or `errors`.
#[derive(Serialize)]
struct FormatResult {
    ok: bool,
    text: Option<String>,
    diagnostics: Vec<JsDiag>,
}

/// Normalize operator glyphs. With `ascii = true`, Unicode operators become
/// their ASCII spellings; otherwise ASCII becomes Unicode. Returns
/// `{ ok, text, diagnostics }` (each diagnostic shaped as in [`check`], incl.
/// its `fixes` array).
#[wasm_bindgen]
pub fn format(source: &str, ascii: bool) -> Result<JsValue, JsValue> {
    let result = match algae_kernel::fmt::format_source(source, ascii) {
        Ok(text) => FormatResult {
            ok: true,
            text: Some(text),
            diagnostics: Vec::new(),
        },
        Err(diags) => FormatResult {
            ok: false,
            text: None,
            diagnostics: diags.iter().map(|d| JsDiag::from(source, d)).collect(),
        },
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A self-contained good proof: `refl` is in the embedded `core` stdlib.
    const GOOD: &str = "import core(refl);\n\nsort T : Sort;\nop a : -> T;\n\nlemma a_refl\n  |- a = a;\nproof\n  by refl(T, a);\nqed;\n";

    #[test]
    fn good_proof_checks() {
        let r = run_check(GOOD, "playground", Vec::new());
        assert!(r.ok, "expected success, got {:?}", r.diagnostics_messages());
        assert_eq!(r.obligations, 1);
        assert_eq!(r.wip, 0);
        assert!(r.diagnostics.is_empty());
    }

    #[test]
    fn broken_proof_reports_error() {
        // Wrong argument to refl: the sequent no longer matches the proof term.
        let bad = GOOD.replace("by refl(T, a);", "by refl(T, T);");
        let r = run_check(&bad, "playground", Vec::new());
        assert!(!r.ok, "expected failure");
        assert!(!r.diagnostics.is_empty());
        // At least one error carries a source span usable by the editor.
        assert!(r.diagnostics.iter().any(|d| d.severity == "error"));
    }

    #[test]
    fn hole_reports_goal_and_candidates() {
        // `by wip(?name)` surfaces a structured hole report to the editor.
        let src = "import core(refl);\n\nsort T : Sort;\nop a : -> T;\n\nlemma h\n  |- a = a;\nproof\n  by wip(?goal);\nwip;\n";
        let r = run_check(src, "playground", Vec::new());
        assert!(!r.ok, "a hole leaves the proof incomplete");
        assert!(
            r.diagnostics.iter().any(|d| d.message.contains("found hole ?goal")),
            "hole report reaches the web UI"
        );
        assert!(r.diagnostics.iter().any(|d| d.message.contains("Candidates:")));
    }

    #[test]
    fn tactic_hole_reports_arguments() {
        // `by ref(?a, ?b)?` solves argument holes against the goal and surfaces
        // them (with the resulting subgoal) to the editor.
        let src = "import core(symmetry);\n\nsort T : Sort;\nop a : -> T;\nop b : -> T;\naxiom ab |- a = b;\n\nlemma flip\n  |- b = a;\nproof\n  by symmetry(T, ?x, ?y) then ?g;\nwip;\n";
        let r = run_check(src, "playground", Vec::new());
        assert!(!r.ok, "a tactic hole leaves the proof incomplete");
        assert!(
            r.diagnostics.iter().any(|d| d.message.contains("found tactic hole")),
            "tactic-hole report reaches the web UI"
        );
        assert!(r.diagnostics.iter().any(|d| d.message.contains("Holes:")));
    }

    #[test]
    fn unknown_import_reports_error() {
        let r = run_check("import nope(x);\n", "playground", Vec::new());
        assert!(!r.ok);
        assert!(r
            .diagnostics
            .iter()
            .any(|d| d.message.contains("nope") || d.severity == "error"));
    }

    #[test]
    fn diagnostics_carry_fixes() {
        // A complete proof closed with `wip` → terminator fix swapping in `qed`.
        let src = "import core(refl);\n\nsort T : Sort;\nop a : -> T;\n\nlemma l\n  |- a = a;\nproof\n  by refl(T, a);\nwip;\n";
        let r = run_check(src, "playground", Vec::new());
        assert!(!r.ok);
        let fix = r
            .diagnostics
            .iter()
            .flat_map(|d| &d.fixes)
            .find(|f| f.replacement == "qed")
            .expect("expected a serialized `qed` fix");
        // The fix carries both byte offsets and line/col for the editor.
        assert_eq!(&src[fix.start..fix.end], "wip");
        assert!(fix.line >= 1 && fix.col >= 1);
        assert!(!fix.title.is_empty());
    }

    #[test]
    fn hole_diagnostic_carries_candidate_fixes() {
        let src = "import core(refl);\n\nsort T : Sort;\nop a : -> T;\n\nlemma h\n  |- a = a;\nproof\n  by wip(?goal);\nwip;\n";
        let r = run_check(src, "playground", Vec::new());
        let fixes: Vec<&JsFix> = r.diagnostics.iter().flat_map(|d| &d.fixes).collect();
        assert!(!fixes.is_empty(), "hole should surface candidate fixes to JS");
        assert!(fixes.iter().all(|f| f.replacement.starts_with("by ")));
    }

    #[test]
    fn extra_module_resolves() {
        let src = "import mymod(Foo);\n";
        let extra = vec![("mymod".to_string(), "sort Foo : Sort;\n".to_string())];
        let r = run_check(src, "playground", extra);
        assert!(r.ok, "expected success, got {:?}", r.diagnostics_messages());
    }

    impl CheckResult {
        fn diagnostics_messages(&self) -> Vec<String> {
            self.diagnostics.iter().map(|d| d.message.clone()).collect()
        }
    }
}
