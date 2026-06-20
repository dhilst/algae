//! Project model: module resolution and (later) the `algae.json` manifest and
//! `.algo` cache. For now, a directory-based source resolver.

use crate::elaborate::proof::SourceResolver;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// The `algae.json` project manifest. It lets a project pull `.alg` modules from
/// other locations in the tree (source roots and named dependencies).
#[derive(Debug, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: u32,
    /// Directories searched for modules, relative to the manifest.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Named dependencies; each `path` is a directory of modules.
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    /// Override the standard-library directory.
    #[serde(default)]
    pub stdlib: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub path: String,
}

impl Manifest {
    /// Search upward from `start` for an `algae.json`, returning its directory
    /// and parsed contents.
    pub fn find(start: &Path) -> Option<(PathBuf, Manifest)> {
        let mut dir = if start.is_dir() {
            start.to_path_buf()
        } else {
            start.parent()?.to_path_buf()
        };
        loop {
            let candidate = dir.join("algae.json");
            if candidate.is_file() {
                let text = std::fs::read_to_string(&candidate).ok()?;
                let manifest: Manifest = serde_json::from_str(&text).ok()?;
                return Some((dir, manifest));
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// The module search roots this manifest defines (relative paths resolved
    /// against `dir`), with the standard library appended.
    pub fn roots(&self, dir: &Path, stdlib_override: Option<PathBuf>) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        for s in &self.sources {
            roots.push(dir.join(s));
        }
        for d in &self.dependencies {
            roots.push(dir.join(&d.path));
        }
        let stdlib = stdlib_override
            .or_else(|| self.stdlib.as_ref().map(|s| dir.join(s)))
            .unwrap_or_else(default_stdlib);
        roots.push(stdlib);
        roots
    }
}

/// Resolves module names to `.alg` sources by searching a list of roots in
/// order (project source roots, then the standard library).
pub struct DirResolver {
    pub roots: Vec<PathBuf>,
}

impl DirResolver {
    pub fn new(roots: Vec<PathBuf>) -> DirResolver {
        DirResolver { roots }
    }
}

impl SourceResolver for DirResolver {
    fn resolve(&self, module: &str) -> Result<String, String> {
        for r in &self.roots {
            let p = r.join(format!("{module}.alg"));
            if p.is_file() {
                return std::fs::read_to_string(&p).map_err(|e| e.to_string());
            }
        }
        Err(format!("module `{module}` not found in any source root"))
    }
}

/// The default standard-library directory.
pub fn default_stdlib() -> PathBuf {
    PathBuf::from("algae/stdlib/v1")
}
