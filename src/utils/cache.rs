use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub last_updated: u64,
    pub total_entries: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache<T> {
    pub metadata: CacheMetadata,
    pub entries: HashMap<String, CacheEntry<T>>,
}

#[allow(dead_code)]
impl<T> Cache<T> {
    pub fn new() -> Self {
        Self {
            metadata: CacheMetadata {
                last_updated: 0,
                total_entries: 0,
            },
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, data: T) {
        self.insert_with_metadata(key, data, HashMap::new());
    }

    pub fn insert_with_metadata(
        &mut self,
        key: String,
        data: T,
        metadata: HashMap<String, String>,
    ) {
        let entry = CacheEntry {
            data,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata,
        };

        self.entries.insert(key, entry);
        self.update_metadata();
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key).map(|entry| &entry.data)
    }

    pub fn get_entry(&self, key: &str) -> Option<&CacheEntry<T>> {
        self.entries.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<CacheEntry<T>> {
        let result = self.entries.remove(key);
        self.update_metadata();
        result
    }

    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.update_metadata();
    }

    pub fn is_stale(&self, max_age_seconds: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now.saturating_sub(self.metadata.last_updated) > max_age_seconds
    }

    pub fn is_entry_stale(&self, key: &str, max_age_seconds: u64) -> bool {
        if let Some(entry) = self.entries.get(key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            now.saturating_sub(entry.timestamp) > max_age_seconds
        } else {
            true // Non-existent entries are considered stale
        }
    }

    pub fn filter_by_metadata(&self, key: &str, value: &str) -> Vec<(&String, &T)> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.metadata.get(key).map_or(false, |v| v == value))
            .map(|(k, entry)| (k, &entry.data))
            .collect()
    }

    fn update_metadata(&mut self) {
        self.metadata.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.metadata.total_entries = self.entries.len();
    }
}

pub struct CacheManager {
    cache_dir: PathBuf,
}
#[allow(dead_code)]
impl CacheManager {
    /// Creates a new CacheManager instance.
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        Ok(Self { cache_dir })
    }

    fn get_cache_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().context("Unable to determine home directory")?;

        // Follow XDG Base Directory specification
        let cache_dir = home_dir.join(".local").join("share").join("gitforge");

        Ok(cache_dir)
    }

    pub fn ensure_cache_dir(&self) -> Result<()> {
        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir).with_context(|| {
                format!("Failed to create cache directory: {:?}", self.cache_dir)
            })?;
        }
        Ok(())
    }

    pub fn load_cache<T>(&self, cache_name: &str) -> Result<Cache<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let cache_file = self.cache_dir.join(format!("{}.json", cache_name));

        if !cache_file.exists() {
            return Ok(Cache::new());
        }

        let content = fs::read_to_string(&cache_file)
            .with_context(|| format!("Failed to read cache file: {:?}", cache_file))?;

        let cache: Cache<T> = serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse cache file: {:?}\n\nCaused by:\n    {}",
                cache_file,
                e
            )
        })?;

        Ok(cache)
    }

    pub fn save_cache<T>(&self, cache_name: &str, cache: &Cache<T>) -> Result<()>
    where
        T: Serialize,
    {
        self.ensure_cache_dir()?;

        let cache_file = self.cache_dir.join(format!("{}.json", cache_name));

        let content = serde_json::to_string_pretty(cache).context("Failed to serialize cache")?;

        fs::write(&cache_file, content)
            .with_context(|| format!("Failed to write cache file: {:?}", cache_file))?;

        Ok(())
    }

    pub fn cache_exists(&self, cache_name: &str) -> bool {
        self.cache_dir.join(format!("{}.json", cache_name)).exists()
    }

    pub fn clear_cache(&self, cache_name: &str) -> Result<()> {
        let cache_file = self.cache_dir.join(format!("{}.json", cache_name));

        if cache_file.exists() {
            fs::remove_file(&cache_file)
                .with_context(|| format!("Failed to remove cache file: {:?}", cache_file))?;
        }
        Ok(())
    }

    pub fn clear_all_caches(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).with_context(|| {
                format!("Failed to remove cache directory: {:?}", self.cache_dir)
            })?;
        }
        Ok(())
    }

    pub fn get_cache_path(&self, cache_name: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", cache_name))
    }

    pub fn get_cache_size(&self, cache_name: &str) -> Result<u64> {
        let cache_file = self.cache_dir.join(format!("{}.json", cache_name));

        if !cache_file.exists() {
            return Ok(0);
        }

        let metadata = fs::metadata(&cache_file)
            .with_context(|| format!("Failed to get cache file metadata: {:?}", cache_file))?;

        Ok(metadata.len())
    }

    pub fn list_caches(&self) -> Result<Vec<String>> {
        if !self.cache_dir.exists() {
            return Ok(vec![]);
        }

        let mut caches = Vec::new();

        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_stem() {
                    if let Some(name_str) = name.to_str() {
                        caches.push(name_str.to_string());
                    }
                }
            }
        }

        Ok(caches)
    }

    pub fn should_update_cache<T>(&self, cache_name: &str, max_age_seconds: u64) -> Result<bool>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !self.cache_exists(cache_name) {
            return Ok(true);
        }

        let cache: Cache<T> = self.load_cache(cache_name)?;
        Ok(cache.is_stale(max_age_seconds))
    }
}
