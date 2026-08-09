use std::collections::HashMap;

pub struct ContextCache {
    cache: HashMap<String, (String, usize)>, // key -> (content, token_count)
    max_entries: usize,
}

impl ContextCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries,
        }
    }

    pub fn get(&self, key: &str) -> Option<&(String, usize)> {
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: String, content: String, token_count: usize) {
        // Simple eviction: clear cache if full (YAGNI: upgrade to LRU later if needed)
        if self.cache.len() >= self.max_entries {
            self.cache.clear();
            tracing::debug!("Context cache evicted due to size limit");
        }
        self.cache.insert(key, (content, token_count));
    }
}
