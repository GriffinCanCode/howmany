use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use crate::core::types::FileStats;
use crate::utils::errors::{HowManyError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub stats: FileStats,
    pub last_modified: u64,
    pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileCache {
    entries: HashMap<PathBuf, CacheEntry>,
    cache_version: u32,
}

impl FileCache {
    const CACHE_VERSION: u32 = 1;
    
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            cache_version: Self::CACHE_VERSION,
        }
    }
    
    pub fn load() -> Result<Self> {
        let cache_path = Self::cache_path()?;
        
        if cache_path.exists() {
            let content = fs::read_to_string(&cache_path)?;
            let cache: FileCache = serde_json::from_str(&content)
                .map_err(|e| HowManyError::invalid_config(format!("Failed to parse cache: {}", e)))?;
            
            // Check cache version compatibility
            if cache.cache_version == Self::CACHE_VERSION {
                Ok(cache)
            } else {
                // Cache version mismatch, start fresh
                Ok(Self::new())
            }
        } else {
            Ok(Self::new())
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let cache_path = Self::cache_path()?;
        
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&cache_path, content)?;
        Ok(())
    }
    
    pub fn get(&self, path: &Path) -> Option<&FileStats> {
        if let Ok(metadata) = fs::metadata(path) {
            if let Some(entry) = self.entries.get(path) {
                let current_modified = metadata.modified()
                    .ok()?
                    .duration_since(UNIX_EPOCH)
                    .ok()?
                    .as_secs();
                
                let current_size = metadata.len();
                
                // Check if file hasn't changed
                if entry.last_modified == current_modified && entry.file_size == current_size {
                    return Some(&entry.stats);
                }
            }
        }
        None
    }
    
    pub fn insert(&mut self, path: PathBuf, stats: FileStats) -> Result<()> {
        if let Ok(metadata) = fs::metadata(&path) {
            let last_modified = metadata.modified()?
                .duration_since(UNIX_EPOCH)
                .map_err(|e| HowManyError::file_processing(format!("Time error: {}", e)))?
                .as_secs();
            
            let file_size = metadata.len();
            
            let entry = CacheEntry {
                stats,
                last_modified,
                file_size,
            };
            
            self.entries.insert(path, entry);
        }
        Ok(())
    }
    
    pub fn remove(&mut self, path: &Path) {
        self.entries.remove(path);
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    pub fn cleanup_missing_files(&mut self) {
        let missing_paths: Vec<_> = self.entries
            .keys()
            .filter(|path| !path.exists())
            .cloned()
            .collect();
        
        for path in missing_paths {
            self.entries.remove(&path);
        }
    }
    
    pub fn size(&self) -> usize {
        self.entries.len()
    }
    
    fn cache_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| HowManyError::invalid_config("Could not find cache directory"))?;
        
        Ok(cache_dir.join("howmany").join("file_cache.json"))
    }
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;
    
    #[test]
    fn test_cache_creation() {
        let cache = FileCache::new();
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.cache_version, FileCache::CACHE_VERSION);
    }
    
    #[test]
    fn test_cache_insert_and_get() {
        let project = TestProject::new("test_project").unwrap();
        let file_path = project.create_file("test.rs", "fn main() {}").unwrap();
        
        let mut cache = FileCache::new();
        let stats = FileStats {
            total_lines: 1,
            code_lines: 1,
            comment_lines: 0,
            blank_lines: 0,
            file_size: 12,
            doc_lines: 0,
        };
        
        cache.insert(file_path.clone(), stats.clone()).unwrap();
        
        let cached_stats = cache.get(&file_path);
        assert!(cached_stats.is_some());
        assert_eq!(cached_stats.unwrap().total_lines, 1);
    }
    
    #[test]
    fn test_cache_miss_on_modified_file() {
        let project = TestProject::new("test_project").unwrap();
        let file_path = project.create_file("test.rs", "fn main() {}").unwrap();
        
        let mut cache = FileCache::new();
        let stats = FileStats {
            total_lines: 1,
            code_lines: 1,
            comment_lines: 0,
            blank_lines: 0,
            file_size: 12,
            doc_lines: 0,
        };
        
        cache.insert(file_path.clone(), stats).unwrap();
        
        // Modify the file
        std::thread::sleep(std::time::Duration::from_millis(10));
        project.create_file("test.rs", "fn main() {}\nfn test() {}").unwrap();
        
        // Cache should miss now
        let cached_stats = cache.get(&file_path);
        assert!(cached_stats.is_none());
    }
    
    #[test]
    fn test_cache_cleanup() {
        let project = TestProject::new("test_project").unwrap();
        let file_path = project.create_file("test.rs", "fn main() {}").unwrap();
        
        let mut cache = FileCache::new();
        let stats = FileStats {
            total_lines: 1,
            code_lines: 1,
            comment_lines: 0,
            blank_lines: 0,
            file_size: 12,
            doc_lines: 0,
        };
        
        cache.insert(file_path.clone(), stats).unwrap();
        assert_eq!(cache.size(), 1);
        
        // Remove the file
        fs::remove_file(&file_path).unwrap();
        
        // Cleanup should remove the entry
        cache.cleanup_missing_files();
        assert_eq!(cache.size(), 0);
    }
} 