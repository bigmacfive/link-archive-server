use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;
use uuid::Uuid;
use serde::{Serialize, de::DeserializeOwned};

static CACHE: Lazy<Mutex<HashMap<String, CacheEntry>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
struct CacheEntry {
    data: Vec<u8>,
    expiry: u64,
}

pub struct Cache {
    ttl: u64,
}

impl Cache {
    pub fn new(ttl_seconds: u64) -> Self {
        Self { ttl: ttl_seconds }
    }

    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), anyhow::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let serialized = bincode::serialize(value)?;
        let entry = CacheEntry {
            data: serialized,
            expiry: now + self.ttl,
        };

        let mut cache = CACHE.lock().unwrap();
        cache.insert(key.to_string(), entry);
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut cache = CACHE.lock().unwrap();
        
        if let Some(entry) = cache.get(key) {
            if entry.expiry > now {
                return bincode::deserialize(&entry.data).ok();
            } else {
                cache.remove(key);
            }
        }
        None
    }

    pub fn delete(&self, key: &str) {
        let mut cache = CACHE.lock().unwrap();
        cache.remove(key);
    }

    pub fn clear_expired(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut cache = CACHE.lock().unwrap();
        cache.retain(|_, entry| entry.expiry > now);
    }

    pub fn get_or_insert<T, F>(&self, key: &str, f: F) -> Result<T, anyhow::Error>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Result<T, anyhow::Error>,
    {
        if let Some(value) = self.get(key) {
            return Ok(value);
        }

        let value = f()?;
        self.set(key, &value)?;
        Ok(value)
    }

    pub fn invalidate_by_prefix(&self, prefix: &str) {
        let mut cache = CACHE.lock().unwrap();
        cache.retain(|key, _| !key.starts_with(prefix));
    }

    pub fn get_keys_by_prefix(&self, prefix: &str) -> Vec<String> {
        let cache = CACHE.lock().unwrap();
        cache
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect()
    }
}

pub fn create_cache_key(parts: &[&str]) -> String {
    parts.join(":")
}

pub fn create_user_cache_key(user_id: Uuid, suffix: &str) -> String {
    create_cache_key(&["user", &user_id.to_string(), suffix])
}

pub fn create_link_cache_key(link_id: Uuid) -> String {
    create_cache_key(&["link", &link_id.to_string()])
}