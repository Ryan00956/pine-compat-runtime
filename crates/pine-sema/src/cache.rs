use std::collections::HashMap;

use pine_syntax::SourceFile;

use crate::analysis::{Analysis, analyze_source};

#[derive(Debug, Default, Clone)]
pub struct CompileCache {
    entries: HashMap<CompileCacheKey, Analysis>,
    hits: usize,
    misses: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheStats {
    pub entries: usize,
    pub hits: usize,
    pub misses: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CompileCacheKey {
    name: String,
    text: String,
}
impl CompileCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze(&mut self, source: &SourceFile) -> Analysis {
        let key = CompileCacheKey::from_source(source);
        if let Some(analysis) = self.entries.get(&key) {
            self.hits += 1;
            return analysis.clone();
        }

        self.misses += 1;
        let analysis = analyze_source(source);
        self.entries.insert(key, analysis.clone());
        analysis
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    #[must_use]
    pub fn stats(&self) -> CompileCacheStats {
        CompileCacheStats {
            entries: self.entries.len(),
            hits: self.hits,
            misses: self.misses,
        }
    }
}
impl CompileCacheKey {
    fn from_source(source: &SourceFile) -> Self {
        Self {
            name: source.name().to_owned(),
            text: source.text().to_owned(),
        }
    }
}
