use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct Cache<K, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
    max_entries: usize,
}

struct CacheEntry<V> {
    value: V,
    created: Instant,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_entries: 1000,
        }
    }

    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let data = self.data.read().await;
        let entry = data.get(key)?;

        if entry.created.elapsed() > self.ttl {
            return None;
        }

        Some(entry.value.clone())
    }

    pub async fn set(&self, key: K, value: V) {
        let mut data = self.data.write().await;

        if data.len() >= self.max_entries {
            let oldest = data
                .iter()
                .min_by_key(|(_, entry)| entry.created)
                .map(|(k, _)| k.clone());

            if let Some(oldest_key) = oldest {
                data.remove(&oldest_key);
            }
        }

        data.insert(
            key,
            CacheEntry {
                value,
                created: Instant::now(),
            },
        );
    }

    pub async fn invalidate(&self, key: &K) {
        let mut data = self.data.write().await;
        data.remove(key);
    }

    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }

    pub async fn size(&self) -> usize {
        let data = self.data.read().await;
        data.len()
    }
}

pub type StringCache = Cache<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache() {
        let cache = Cache::<String, String>::new(Duration::from_secs(10));

        cache.set("key1".to_string(), "value1".to_string()).await;

        let value = cache.get(&"key1".to_string()).await;
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "value1");
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = Cache::<String, String>::new(Duration::from_secs(10));

        let value = cache.get(&"nonexistent".to_string()).await;
        assert!(value.is_none());
    }
}
