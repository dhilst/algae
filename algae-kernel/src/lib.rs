//! Algae v2 — a parser-oriented proof and algebraic specification language.
//!
//! This is the kernel: an environment-free core (no threads, filesystem, or
//! terminal I/O). The toolchain is organized as a pipeline:
//! Parse → Elaborate → IR → Check.

pub mod core;
pub mod diagnostics;
pub mod elaborate;
pub mod fmt;
pub mod parse;

/// Elaborate and fully proof-check a source unit, returning all diagnostics
/// (empty on success). A convenience entry point for tests and embedding.
pub fn verify_source(
    source: &str,
    module_name: &str,
    resolver: &dyn elaborate::proof::SourceResolver,
) -> Vec<diagnostics::Diagnostic> {
    let unit = match elaborate::proof::elaborate_unit(source, module_name, resolver, true) {
        Ok(u) => u,
        Err(diags) => return diags,
    };
    let mut errors = Vec::new();
    for ob in &unit.obligations {
        errors.extend(core::check::check(&ob.root, &ob.label, &unit.rewrite));
    }
    errors
}
