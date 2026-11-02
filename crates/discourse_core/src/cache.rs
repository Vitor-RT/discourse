use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

const MAX_CACHE_AGE: Duration = Duration::from_secs(24 * 60 * 60); // 1 day

pub struct Cache {
    namespace: String,
    store: RwLock<HashMap<String, (String, Instant)>>,
}

impl Cache {
    pub fn new(namespace: Option<&str>) -> Self {
        Self {
            namespace: namespace.unwrap_or("_CACHE").to_string(),
            store: RwLock::new(HashMap::new()),
        }
    }

    pub fn normalize_key(&self, key: &str) -> String {
        format!("{}:{}", self.namespace, key)
    }

    pub fn keys(&self, pattern: &str) -> Vec<String> {
        let prefix = format!("{}:", self.namespace);
        let store = self.store.read().unwrap();
        store
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .filter(|k| {
                if pattern == "*" {
                    true
                } else {
                    let suffix = &k[prefix.len()..];
                    if pattern.ends_with('*') {
                        suffix.starts_with(&pattern[..pattern.len() - 1])
                    } else {
                        suffix == pattern
                    }
                }
            })
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        let keys = self.keys("*");
        let mut store = self.store.write().unwrap();
        for key in keys {
            store.remove(&key);
        }
    }

    pub fn exist(&self, name: &str) -> bool {
        let key = self.normalize_key(name);
        let store = self.store.read().unwrap();
        if let Some((_, expires_at)) = store.get(&key) {
            if Instant::now() <= *expires_at {
                return true;
            }
        }
        false
    }

    pub fn read(&self, name: &str) -> Option<String> {
        let key = self.normalize_key(name);
        self.read_entry(&key)
    }

    pub fn write(&self, name: &str, value: &str, expires_in: Option<Duration>) {
        let key = self.normalize_key(name);
        self.write_entry(&key, value, expires_in);
    }

    pub fn delete(&self, name: &str) {
        let key = self.normalize_key(name);
        let mut store = self.store.write().unwrap();
        store.remove(&key);
    }

    pub fn fetch<F>(&self, name: &str, expires_in: Option<Duration>, force: bool, blk: F) -> String
    where
        F: FnOnce() -> String,
    {
        if !force {
            if let Some(val) = self.read(name) {
                return val;
            }
        }

        let val = blk();
        self.write(name, &val, expires_in);
        val
    }

    fn read_entry(&self, key: &str) -> Option<String> {
        let store = self.store.read().unwrap();
        if let Some((value, expires_at)) = store.get(key) {
            if Instant::now() <= *expires_at {
                return Some(value.clone());
            }
        }
        None
    }

    fn write_entry(&self, key: &str, value: &str, expires_in: Option<Duration>) {
        let expiry = expires_in.unwrap_or(MAX_CACHE_AGE);
        let mut store = self.store.write().unwrap();
        store.insert(
            key.to_string(),
            (value.to_string(), Instant::now() + expiry),
        );
    }
}

pub struct DistributedCache {
    pub key: String,
    pub hash: RwLock<HashMap<String, String>>,
}

impl DistributedCache {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            hash: RwLock::new(HashMap::new()),
        }
    }

    pub fn defer_set(&self, k: &str, v: &str) {
        let mut hash = self.hash.write().unwrap();
        hash.insert(k.to_string(), v.to_string());
    }

    pub fn defer_get_set<F>(&self, k: &str, block: F) -> String
    where
        F: FnOnce() -> String,
    {
        {
            let hash = self.hash.read().unwrap();
            if let Some(val) = hash.get(k) {
                return val.clone();
            }
        }
        let value = block();
        self.defer_set(k, &value);
        value
    }

    pub fn defer_get_set_bulk<F, K>(
        &self,
        ks: &[String],
        key_blk: K,
        blk: F,
    ) -> HashMap<String, String>
    where
        K: Fn(&str) -> String,
        F: FnOnce(&[String]) -> Vec<String>,
    {
        let mut found_hash = HashMap::new();
        let mut missing_keys = Vec::new();

        {
            let hash = self.hash.read().unwrap();
            for k in ks {
                let mapped_key = key_blk(k);
                if let Some(val) = hash.get(&mapped_key) {
                    found_hash.insert(k.clone(), val.clone());
                } else {
                    missing_keys.push(k.clone());
                }
            }
        }

        if !missing_keys.is_empty() {
            let missing_values = blk(&missing_keys);
            let mut missing_hash = HashMap::new();
            for (k, v) in missing_keys.iter().zip(missing_values.into_iter()) {
                missing_hash.insert(k.clone(), v);
            }

            let mut hash = self.hash.write().unwrap();
            for (k, v) in missing_hash.iter() {
                hash.insert(key_blk(k), v.clone());
            }

            let mut result = HashMap::new();
            for k in ks {
                if let Some(v) = found_hash.get(k) {
                    result.insert(k.clone(), v.clone());
                } else if let Some(v) = missing_hash.get(k) {
                    result.insert(k.clone(), v.clone());
                }
            }
            result
        } else {
            found_hash
        }
    }

    pub fn clear(&self) {
        let mut hash = self.hash.write().unwrap();
        hash.clear();
    }

    pub fn clear_regex(&self, regex: &regex::Regex) {
        let mut hash = self.hash.write().unwrap();
        hash.retain(|k, _| !regex.is_match(k));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_write_and_read() {
        let cache = Cache::new(None);
        cache.write("key1", "value1", None);
        assert_eq!(cache.read("key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_cache_exist() {
        let cache = Cache::new(None);
        cache.write("key1", "value1", None);
        assert!(cache.exist("key1"));
        assert!(!cache.exist("key2"));
    }

    #[test]
    fn test_cache_delete() {
        let cache = Cache::new(None);
        cache.write("key1", "value1", None);
        cache.delete("key1");
        assert!(!cache.exist("key1"));
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(None);
        cache.write("key1", "value1", None);
        cache.write("key2", "value2", None);
        cache.clear();
        assert!(!cache.exist("key1"));
        assert!(!cache.exist("key2"));
    }

    #[test]
    fn test_cache_fetch() {
        let cache = Cache::new(None);
        let val1 = cache.fetch("key1", None, false, || "value1".to_string());
        assert_eq!(val1, "value1");

        let val2 = cache.fetch("key1", None, false, || "value2".to_string());
        assert_eq!(val2, "value1");

        let val3 = cache.fetch("key1", None, true, || "value3".to_string());
        assert_eq!(val3, "value3");
    }

    #[test]
    fn test_cache_expire() {
        let cache = Cache::new(None);
        cache.write("key1", "value1", Some(Duration::from_millis(10)));
        assert_eq!(cache.read("key1"), Some("value1".to_string()));
        thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.read("key1"), None);
    }

    #[test]
    fn test_distributed_cache_defer_set() {
        let cache = DistributedCache::new("test");
        cache.defer_set("key1", "value1");
        let hash = cache.hash.read().unwrap();
        assert_eq!(hash.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_distributed_cache_defer_get_set() {
        let cache = DistributedCache::new("test");
        let val = cache.defer_get_set("key1", || "value1".to_string());
        assert_eq!(val, "value1");

        let val2 = cache.defer_get_set("key1", || "value2".to_string());
        assert_eq!(val2, "value1");
    }

    #[test]
    fn test_distributed_cache_defer_get_set_bulk() {
        let cache = DistributedCache::new("test");
        cache.defer_set("key1", "value1");

        let ks = vec!["key1".to_string(), "key2".to_string()];
        let key_blk = |k: &str| k.to_string();
        let blk = |missing_keys: &[String]| -> Vec<String> {
            missing_keys.iter().map(|k| format!("{}val", k)).collect()
        };

        let result = cache.defer_get_set_bulk(&ks, key_blk, blk);

        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"key2val".to_string()));
    }

    #[test]
    fn test_distributed_cache_clear() {
        let cache = DistributedCache::new("test");
        cache.defer_set("key1", "value1");
        cache.clear();
        let hash = cache.hash.read().unwrap();
        assert!(hash.is_empty());
    }

    #[test]
    fn test_distributed_cache_clear_regex() {
        let cache = DistributedCache::new("test");
        cache.defer_set("key1", "value1");
        cache.defer_set("other", "value2");
        let regex = regex::Regex::new("^key").unwrap();
        cache.clear_regex(&regex);
        let hash = cache.hash.read().unwrap();
        assert_eq!(hash.get("key1"), None);
        assert_eq!(hash.get("other"), Some(&"value2".to_string()));
    }
}
