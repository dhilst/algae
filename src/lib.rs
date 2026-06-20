//! Algae v2 — a parser-oriented proof and algebraic specification language.
//!
//! The toolchain is organized as a pipeline:
//! Parse → Elaborate → IR → Bytecode (`.algo`) → Check.

pub mod bytecode;
pub mod cli;
pub mod core;
pub mod diagnostics;
pub mod elaborate;
pub mod fmt;
pub mod parse;
pub mod project;

/// Elaborate and fully proof-check a source unit, returning all errors (empty
/// on success). A convenience entry point for tests and embedding.
pub fn verify_source(
    source: &str,
    module_name: &str,
    resolver: &dyn elaborate::proof::SourceResolver,
    jobs: usize,
) -> Vec<String> {
    let unit = match elaborate::proof::elaborate_unit(source, module_name, resolver, true) {
        Ok(u) => u,
        Err(diags) => return diags.iter().map(|d| d.render(None)).collect(),
    };
    let mut errors = Vec::new();
    for ob in &unit.obligations {
        errors.extend(core::check::check(&ob.root, &ob.label, jobs, &unit.rewrite));
    }
    errors
}
