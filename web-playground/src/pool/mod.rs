//! Element Pool - collections of HTML+CSS design snippets by category
//!
//! The pool is the source of truth for what elements exist.
//! Users contribute new snippets, the system samples from the pool
//! and applies random transforms for training diversity.

pub mod builtins;
pub mod kind;
pub mod snippet;

pub use kind::ElementKind;
pub use snippet::DesignSnippet;

/// The pool of all available design snippets, indexed by kind
#[derive(Clone)]
pub struct ElementPool {
    snippets: std::collections::HashMap<ElementKind, Vec<DesignSnippet>>,
}

impl ElementPool {
    pub fn new() -> Self {
        Self {
            snippets: std::collections::HashMap::new(),
        }
    }

    /// Create a pool seeded with built-in snippets
    pub fn with_builtins() -> Self {
        let mut pool = Self::new();
        for snippet in builtins::builtin_snippets() {
            pool.add(snippet);
        }
        pool
    }

    pub fn add(&mut self, snippet: DesignSnippet) {
        self.snippets
            .entry(snippet.kind)
            .or_default()
            .push(snippet);
    }

    /// Get all snippets of a given kind
    pub fn get(&self, kind: ElementKind) -> &[DesignSnippet] {
        self.snippets.get(&kind).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all snippets across all kinds
    pub fn all(&self) -> Vec<&DesignSnippet> {
        self.snippets.values().flat_map(|v| v.iter()).collect()
    }

    pub fn total(&self) -> usize {
        self.snippets.values().map(|v| v.len()).sum()
    }
}
