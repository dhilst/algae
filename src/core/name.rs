//! Interned symbols. Interning is per compilation unit and uses first-use
//! order, which keeps the symbol table deterministic for serialization.

use std::collections::HashMap;

/// An interned symbol (index into an [`Interner`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sym(pub u32);

#[derive(Clone, Debug, Default)]
pub struct Interner {
    map: HashMap<String, u32>,
    names: Vec<String>,
}

impl Interner {
    pub fn new() -> Interner {
        Interner::default()
    }

    pub fn intern(&mut self, s: &str) -> Sym {
        if let Some(&id) = self.map.get(s) {
            return Sym(id);
        }
        let id = self.names.len() as u32;
        self.names.push(s.to_string());
        self.map.insert(s.to_string(), id);
        Sym(id)
    }

    pub fn resolve(&self, s: Sym) -> &str {
        &self.names[s.0 as usize]
    }

    /// Look up an already-interned string without inserting.
    pub fn get(&self, s: &str) -> Option<Sym> {
        self.map.get(s).map(|&id| Sym(id))
    }

    /// All interned strings in first-use order (for serialization).
    pub fn strings(&self) -> &[String] {
        &self.names
    }

    /// Generate a globally-fresh symbol with a readable base name. Used for
    /// eigenvariable renaming during proof elaboration.
    pub fn fresh(&mut self, base: &str) -> Sym {
        let mut n = 0u32;
        loop {
            let candidate = format!("{base}#{n}");
            if !self.map.contains_key(&candidate) {
                return self.intern(&candidate);
            }
            n += 1;
        }
    }
}
