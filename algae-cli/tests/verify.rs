//! Integration tests: the standard library must verify; the reject corpus must
//! fail. Paths are anchored at the repo root via `CARGO_MANIFEST_DIR` so they
//! resolve regardless of the test's working directory.

use algae_cli::project::DirResolver;
use std::path::{Path, PathBuf};

/// The repository root (one level up from this crate's manifest directory).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn stdlib_dir() -> PathBuf {
    repo_root().join("algae/stdlib/v1")
}

fn module_name(path: &Path) -> String {
    path.file_stem().unwrap().to_str().unwrap().to_string()
}

/// Verify a single `.alg` file, resolving imports against its directory and the
/// standard library. Returns the list of errors (empty = success).
fn verify(path: &Path) -> Vec<String> {
    let src = std::fs::read_to_string(path).expect("read source");
    let roots = vec![path.parent().unwrap().to_path_buf(), stdlib_dir()];
    let resolver = DirResolver::new(roots);
    algae_kernel::verify_source(&src, &module_name(path), &resolver)
        .iter()
        .map(|d| d.render(None))
        .collect()
}

/// Extract the substring from an optional `#expect: <text>` comment line in a
/// fixture. Reject fixtures use it to assert they fail for the *intended*
/// reason, not merely that some error occurred.
fn expected_substring(path: &Path) -> Option<String> {
    let src = std::fs::read_to_string(path).expect("read source");
    for line in src.lines() {
        let line = line.trim_start();
        if let Some(rest) = line.strip_prefix("#expect:") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn alg_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "alg").unwrap_or(false))
        .collect();
    v.sort();
    v
}

#[test]
fn stdlib_verifies() {
    for f in alg_files(&stdlib_dir()) {
        let errors = verify(&f);
        assert!(
            errors.is_empty(),
            "stdlib module {} failed to verify: {:?}",
            f.display(),
            errors
        );
    }
}

#[test]
fn accept_corpus_verifies() {
    let dir = repo_root().join("tests/accept");
    for f in alg_files(&dir) {
        let errors = verify(&f);
        assert!(
            errors.is_empty(),
            "accept fixture {} failed to verify: {:?}",
            f.display(),
            errors
        );
    }
}

#[test]
fn wip_admits_and_is_flagged() {
    // A `by wip` proof: the admitted goal is skipped (no check error), but the
    // obligation is flagged `wip` so the CLI fails the run.
    let src = "\
import core(reflexivity);
sort Nat : Sort;
op z : -> Nat;
lemma t
  |- z = z;
proof
  by wip;
wip;
";
    let resolver = DirResolver::new(vec![stdlib_dir()]);
    let unit = algae_kernel::elaborate::proof::elaborate_unit(src, "t", &resolver, true)
        .expect("wip proof should elaborate");
    assert!(
        unit.obligations.iter().any(|o| o.wip),
        "obligation should be flagged wip"
    );
    // The sound part still checks clean (the admit is skipped).
    for o in &unit.obligations {
        let errs = algae_kernel::core::check::check(&o.root, &o.label, &unit.rewrite);
        assert!(errs.is_empty(), "admitted proof should not produce check errors: {errs:?}");
    }
}

#[test]
fn reject_corpus_fails() {
    let dir = repo_root().join("tests/reject");
    let files = alg_files(&dir);
    assert!(!files.is_empty(), "no reject fixtures found");
    for f in files {
        let errors = verify(&f);
        assert!(
            !errors.is_empty(),
            "reject fixture {} was accepted but should have failed",
            f.display()
        );
        // If the fixture pins down its expected failure reason, the error list
        // must contain it — so the fixture fails for the *intended* reason.
        if let Some(sub) = expected_substring(&f) {
            assert!(
                errors.iter().any(|e| e.contains(&sub)),
                "reject fixture {} failed, but no error matched #expect {:?}; errors: {:?}",
                f.display(),
                sub,
                errors
            );
        }
    }
}
