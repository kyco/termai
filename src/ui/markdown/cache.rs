// Cache implementation for markdown rendering
use ratatui::text::Text;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use lru::LruCache;
use std::num::NonZeroUsize;

/// Cache for rendered markdown content
pub struct MarkdownCache {
    cache: LruCache<u64, Text<'static>>,
    hit_count: u64,
    miss_count: u64,
}

impl MarkdownCache {
    /// Create a new cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).unwrap());
        Self {
            cache: LruCache::new(capacity),
            hit_count: 0,
            miss_count: 0,
        }
    }
    
    /// Get cached rendered text
    pub fn get(&mut self, markdown: &str) -> Option<Text<'static>> {
        let key = self.hash_content(markdown);
        if let Some(text) = self.cache.get(&key) {
            self.hit_count += 1;
            Some(text.clone())
        } else {
            self.miss_count += 1;
            None
        }
    }
    
    /// Insert rendered text into cache
    pub fn insert(&mut self, markdown: String, text: Text<'static>) {
        let key = self.hash_content(&markdown);
        self.cache.put(key, text);
    }
    
    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            capacity: self.cache.cap().get(),
            size: self.cache.len(),
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            hit_rate: if self.hit_count + self.miss_count > 0 {
                self.hit_count as f64 / (self.hit_count + self.miss_count) as f64
            } else {
                0.0
            },
        }
    }
    
    /// Remove old entries to make room (called automatically by LRU)
    pub fn trim_to_size(&mut self, max_size: usize) {
        while self.cache.len() > max_size {
            self.cache.pop_lru();
        }
    }
    
    /// Hash markdown content for cache key
    fn hash_content(&self, content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub capacity: usize,
    pub size: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache: {}/{} entries, {} hits, {} misses, {:.1}% hit rate",
            self.size,
            self.capacity,
            self.hit_count,
            self.miss_count,
            self.hit_rate * 100.0
        )
    }
}

/// Advanced cache with content-aware eviction
pub struct SmartMarkdownCache {
    cache: HashMap<u64, CacheEntry>,
    access_order: Vec<u64>,
    max_size: usize,
    max_memory_bytes: usize,
    current_memory_bytes: usize,
}

#[derive(Clone)]
struct CacheEntry {
    text: Text<'static>,
    last_access: std::time::Instant,
    access_count: u32,
    estimated_size: usize,
}

impl SmartMarkdownCache {
    pub fn new(max_size: usize, max_memory_mb: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: Vec::new(),
            max_size,
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_memory_bytes: 0,
        }
    }
    
    pub fn get(&mut self, markdown: &str) -> Option<Text<'static>> {
        let key = self.hash_content(markdown);
        
        if let Some(entry) = self.cache.get_mut(&key) {
            entry.last_access = std::time::Instant::now();
            entry.access_count += 1;
            
            // Move to end of access order
            if let Some(pos) = self.access_order.iter().position(|&x| x == key) {
                self.access_order.remove(pos);
            }
            self.access_order.push(key);
            
            Some(entry.text.clone())
        } else {
            None
        }
    }
    
    pub fn insert(&mut self, markdown: String, text: Text<'static>) {
        let key = self.hash_content(&markdown);
        let estimated_size = self.estimate_text_size(&text);
        
        // Remove existing entry if present
        if let Some(old_entry) = self.cache.remove(&key) {
            self.current_memory_bytes -= old_entry.estimated_size;
        }
        
        // Ensure we have space
        self.make_space_for(estimated_size);
        
        let entry = CacheEntry {
            text,
            last_access: std::time::Instant::now(),
            access_count: 1,
            estimated_size,
        };
        
        self.cache.insert(key, entry);
        self.access_order.push(key);
        self.current_memory_bytes += estimated_size;
    }
    
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.current_memory_bytes = 0;
    }
    
    fn make_space_for(&mut self, size_needed: usize) {
        // Remove entries until we have enough space
        while (self.cache.len() >= self.max_size || 
               self.current_memory_bytes + size_needed > self.max_memory_bytes) &&
              !self.cache.is_empty() {
            
            // Find least recently used entry
            if let Some(&lru_key) = self.access_order.first() {
                if let Some(entry) = self.cache.remove(&lru_key) {
                    self.current_memory_bytes -= entry.estimated_size;
                }
                self.access_order.remove(0);
            } else {
                break;
            }
        }
    }
    
    fn estimate_text_size(&self, text: &Text) -> usize {
        // Rough estimation of memory usage
        text.lines.iter()
            .map(|line| {
                line.spans.iter()
                    .map(|span| span.content.len() + 32) // Content + style overhead
                    .sum::<usize>()
            })
            .sum::<usize>() + 64 // Text overhead
    }
    
    fn hash_content(&self, content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::{Line, Span};
    
    #[test]
    fn test_cache_basic_operations() {
        let mut cache = MarkdownCache::new(10);
        let text = Text::from("test");
        
        // Miss on empty cache
        assert!(cache.get("test").is_none());
        
        // Insert and retrieve
        cache.insert("test".to_string(), text.clone());
        assert!(cache.get("test").is_some());
        
        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
    }
    
    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = MarkdownCache::new(2);
        
        cache.insert("first".to_string(), Text::from("1"));
        cache.insert("second".to_string(), Text::from("2"));
        cache.insert("third".to_string(), Text::from("3")); // Should evict "first"
        
        assert!(cache.get("first").is_none());
        assert!(cache.get("second").is_some());
        assert!(cache.get("third").is_some());
    }
    
    #[test]
    fn test_smart_cache() {
        let mut cache = SmartMarkdownCache::new(3, 1); // 1MB limit
        let text = Text::from(Line::from(vec![Span::from("test")]));
        
        cache.insert("test".to_string(), text.clone());
        assert!(cache.get("test").is_some());
        
        cache.clear();
        assert!(cache.get("test").is_none());
    }
}