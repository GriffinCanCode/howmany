//! Persistent per-file statistics cache.
//!
//! Three properties matter more than raw speed here, because a cache that is
//! wrong or that corrupts itself is worse than no cache at all:
//!
//! * **Never serve stale data.** An entry is only reused when the file's size
//!   *and* modification time both match what was recorded.
//! * **Never fail the run.** A missing, truncated, or foreign-version cache file
//!   is discarded and rebuilt rather than surfaced as an error.
//! * **Never leave a half-written file.** Saving writes to a temporary file in
//!   the same directory and renames it into place, so a process killed mid-save
//!   leaves the previous cache intact.

use crate::core::types::FileStats;
use crate::utils::errors::{HowManyError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Bump when the meaning of a cache entry changes; older files are discarded.
// Bumped when the meaning of a stored field changes, so that a cache written by
// an older build is discarded rather than misread. Version 3 records
// modification times in nanoseconds rather than seconds.
const CACHE_VERSION: u32 = 3;

/// Upper bound on retained entries *per project*.
///
/// Without a bound the cache grows for the lifetime of the machine: every file
/// ever analyzed, in every repository, re-serialized on each run. Eviction keeps
/// load and save time bounded no matter how long the tool has been in use.
const MAX_ENTRIES: usize = 100_000;

/// Upper bound on retained per-project cache files.
///
/// Each scanned project gets its own file, so the directory would otherwise grow
/// once per project forever. Pruning the least recently used files keeps the
/// directory small without ever affecting an answer.
const MAX_SCOPES: usize = 64;

/// Identity of a file version: its size and modification time.
///
/// Callers that already have `std::fs::Metadata` -- traversal always does --
/// should build the key from it. That removes the `stat` the cache would
/// otherwise perform for every single file on every single run.
///
/// The timestamp is kept in **nanoseconds**. At one-second resolution, two edits
/// inside the same second that happened to leave the file the same length were
/// indistinguishable, and the second one was served from the cache -- which is
/// exactly what a code generator, a `git checkout`, or a fast editor does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheKey {
    pub size: u64,
    pub modified: u64,
}

impl CacheKey {
    /// Build a key from metadata already in hand.
    pub fn from_metadata(metadata: &fs::Metadata) -> Option<Self> {
        Some(Self {
            size: metadata.len(),
            modified: to_unix_nanos(metadata.modified().ok()?)?,
        })
    }

    /// Build a key by inspecting `path`, if it can be read.
    pub fn for_path(path: &Path) -> Option<Self> {
        Self::from_metadata(&fs::metadata(path).ok()?)
    }
}

fn to_unix_nanos(time: SystemTime) -> Option<u64> {
    let since_epoch = time.duration_since(UNIX_EPOCH).ok()?;
    u64::try_from(since_epoch.as_nanos()).ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub stats: FileStats,
    pub last_modified: u64,
    pub file_size: u64,
    /// Monotonic counter used to evict the least recently touched entries.
    #[serde(default)]
    pub touched: u64,
}

impl CacheEntry {
    fn matches(&self, key: &CacheKey) -> bool {
        self.last_modified == key.modified && self.file_size == key.size
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileCache {
    entries: HashMap<PathBuf, CacheEntry>,
    cache_version: u32,
    #[serde(default)]
    clock: u64,
    /// Where this cache persists, or `None` for an in-memory cache.
    ///
    /// Not serialized: it names the file holding the cache, so storing it inside
    /// that file would only be able to disagree with reality.
    #[serde(skip)]
    location: Option<PathBuf>,
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FileCache {
    pub const CACHE_VERSION: u32 = CACHE_VERSION;

    /// An in-memory cache that is never written to disk.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            cache_version: CACHE_VERSION,
            clock: 0,
            location: None,
        }
    }

    /// The cache for one project root, loaded from disk if it is there.
    ///
    /// Scoping by root is what keeps the cache worth having. A single shared file
    /// costs every run the time to parse and rewrite *every file the machine has
    /// ever analyzed*: after a few large repositories that overhead exceeded the
    /// work it saved, so the cache made runs slower. It also meant a large
    /// project could evict a small one's entries, so alternating between two
    /// repositories left neither of them cached.
    pub fn scoped(root: &Path) -> Self {
        match Self::cache_path_for(root) {
            Ok(path) => {
                let mut cache = Self::read_from(&path);
                cache.location = Some(path);
                cache
            }
            Err(_) => Self::new(),
        }
    }

    /// Read a cache file, treating anything unusable as an empty cache.
    ///
    /// A cache written by a different version, or one that no longer parses, is
    /// treated as absent rather than as an error: the correct response to an
    /// unusable cache is to rebuild it.
    fn read_from(path: &Path) -> Self {
        let Ok(content) = fs::read_to_string(path) else {
            return Self::new();
        };
        match serde_json::from_str::<FileCache>(&content) {
            Ok(cache) if cache.cache_version == CACHE_VERSION => cache,
            _ => Self::new(),
        }
    }

    pub fn version(&self) -> u32 {
        self.cache_version
    }

    /// Write the cache atomically.
    ///
    /// The payload is written to a sibling temporary file and renamed over the
    /// destination, so an interrupted save can never leave a truncated cache
    /// that the next run would have to discard.
    ///
    /// An in-memory cache has nowhere to go and saving it is a no-op, not an
    /// error: the caller asked for a cache that is not persisted.
    pub fn save(&self) -> Result<()> {
        let Some(cache_path) = self.location.as_ref() else {
            return Ok(());
        };

        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let payload = serde_json::to_vec(self)?;
        let temp_path = cache_path.with_extension(format!("tmp.{}", std::process::id()));

        fs::write(&temp_path, &payload)?;
        if let Err(err) = fs::rename(temp_path.as_path(), cache_path) {
            let _ = fs::remove_file(&temp_path);
            return Err(err.into());
        }

        Self::prune_scopes();
        Ok(())
    }

    /// Statistics for `path` if the recorded version is still current.
    pub fn get(&self, path: &Path) -> Option<&FileStats> {
        let key = CacheKey::for_path(path)?;
        self.get_with_key(path, &key)
    }

    /// Statistics for `path` given a key the caller already computed.
    pub fn get_with_key(&self, path: &Path, key: &CacheKey) -> Option<&FileStats> {
        self.entries
            .get(path)
            .filter(|entry| entry.matches(key))
            .map(|entry| &entry.stats)
    }

    /// Record statistics for `path`, inspecting it to build the key.
    pub fn insert(&mut self, path: PathBuf, stats: FileStats) -> Result<()> {
        match CacheKey::for_path(&path) {
            Some(key) => {
                self.insert_with_key(path, stats, key);
                Ok(())
            }
            None => Err(HowManyError::file_processing(format!(
                "could not read metadata for {}",
                path.display()
            ))),
        }
    }

    /// Record statistics for `path` under a caller-supplied key.
    pub fn insert_with_key(&mut self, path: PathBuf, stats: FileStats, key: CacheKey) {
        self.clock = self.clock.wrapping_add(1);
        self.entries.insert(
            path,
            CacheEntry {
                stats,
                last_modified: key.modified,
                file_size: key.size,
                touched: self.clock,
            },
        );

        if self.entries.len() > MAX_ENTRIES {
            self.evict_oldest();
        }
    }

    /// Merge entries produced elsewhere, e.g. by parallel workers.
    pub fn extend_from(
        &mut self,
        updates: impl IntoIterator<Item = (PathBuf, FileStats, CacheKey)>,
    ) {
        for (path, stats, key) in updates {
            self.insert_with_key(path, stats, key);
        }
    }

    /// Drop the least recently touched quarter of the cache.
    ///
    /// Evicting in bulk keeps this off the hot path: it runs once per overflow
    /// rather than on every insertion past the limit.
    fn evict_oldest(&mut self) {
        let target = MAX_ENTRIES * 3 / 4;
        let mut ages: Vec<(u64, PathBuf)> = self
            .entries
            .iter()
            .map(|(path, entry)| (entry.touched, path.clone()))
            .collect();
        ages.sort_unstable_by_key(|(touched, _)| *touched);

        for (_, path) in ages
            .into_iter()
            .take(self.entries.len().saturating_sub(target))
        {
            self.entries.remove(&path);
        }
    }

    pub fn remove(&mut self, path: &Path) {
        self.entries.remove(path);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.clock = 0;
    }

    /// Forget entries whose files no longer exist.
    pub fn cleanup_missing_files(&mut self) {
        self.entries.retain(|path, _| path.exists());
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Directory holding the per-project cache files.
    ///
    /// `HOWMANY_CACHE_DIR` overrides the platform cache directory, which lets
    /// tests and sandboxed environments keep the cache out of the user's real
    /// one instead of racing against it.
    pub fn cache_dir() -> Result<PathBuf> {
        let dir = match std::env::var_os("HOWMANY_CACHE_DIR") {
            Some(dir) => PathBuf::from(dir),
            None => dirs::cache_dir()
                .ok_or_else(|| HowManyError::invalid_config("Could not find cache directory"))?
                .join("howmany"),
        };
        Ok(dir.join("projects"))
    }

    /// Location of the cache file for one project root.
    ///
    /// The root is canonicalized first so that `.`, a relative path and an
    /// absolute path to the same directory share one cache instead of building
    /// three. The readable stem is only there to make the directory diagnosable
    /// by eye; the hash is what distinguishes the scopes.
    pub fn cache_path_for(root: &Path) -> Result<PathBuf> {
        let resolved = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
        let stem: String = resolved
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "root".to_string())
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .take(32)
            .collect();
        let stem = if stem.is_empty() {
            "root".to_string()
        } else {
            stem
        };
        Ok(Self::cache_dir()?.join(format!("{stem}-{:016x}.json", path_hash(&resolved))))
    }

    /// Drop the least recently modified cache files past `MAX_SCOPES`.
    ///
    /// Errors are ignored throughout: failing to tidy the cache directory is not
    /// a reason to fail an analysis that has already produced its answer.
    fn prune_scopes() {
        let Ok(dir) = Self::cache_dir() else { return };
        let Ok(entries) = fs::read_dir(&dir) else {
            return;
        };

        let mut files: Vec<(SystemTime, PathBuf)> = entries
            .flatten()
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .filter_map(|entry| {
                let modified = entry.metadata().and_then(|meta| meta.modified()).ok()?;
                Some((modified, entry.path()))
            })
            .collect();

        if files.len() <= MAX_SCOPES {
            return;
        }

        files.sort_unstable_by_key(|(modified, _)| *modified);
        for (_, path) in files.iter().take(files.len() - MAX_SCOPES) {
            let _ = fs::remove_file(path);
        }
    }
}

/// FNV-1a over the path bytes.
///
/// Written out rather than taken from `DefaultHasher` because the value names a
/// file on disk: a hasher whose output changed between toolchains would silently
/// orphan every existing cache file.
fn path_hash(path: &Path) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_utils::TestProject;

    fn stats(total: usize) -> FileStats {
        FileStats {
            total_lines: total,
            code_lines: total,
            comment_lines: 0,
            blank_lines: 0,
            file_size: total as u64,
            doc_lines: 0,
        }
    }

    #[test]
    fn test_cache_creation() {
        let cache = FileCache::new();
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.version(), FileCache::CACHE_VERSION);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let project = TestProject::new("test_project").unwrap();
        let file_path = project.create_file("test.rs", "fn main() {}").unwrap();

        let mut cache = FileCache::new();
        cache.insert(file_path.clone(), stats(1)).unwrap();

        let cached = cache.get(&file_path);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().total_lines, 1);
    }

    #[test]
    fn test_cache_miss_on_modified_file() {
        let project = TestProject::new("test_project").unwrap();
        let file_path = project.create_file("test.rs", "fn main() {}").unwrap();

        let mut cache = FileCache::new();
        cache.insert(file_path.clone(), stats(1)).unwrap();

        // A same-second edit still changes the size, so the entry is invalid.
        project
            .create_file("test.rs", "fn main() {}\nfn test() {}")
            .unwrap();

        assert!(
            cache.get(&file_path).is_none(),
            "cache served statistics for a file that changed"
        );
    }

    /// Size alone is not enough: an edit that preserves length must still
    /// invalidate, which is what the modification time is for.
    #[test]
    fn cache_misses_when_content_changes_without_size() {
        let project = TestProject::new("same_size").unwrap();
        let path = project.create_file("a.rs", "fn aaa() {}\n").unwrap();

        let mut cache = FileCache::new();
        let original = CacheKey::for_path(&path).unwrap();
        cache.insert_with_key(path.clone(), stats(1), original);

        // Rewrite with identical length but a later timestamp.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        project.create_file("a.rs", "fn bbb() {}\n").unwrap();
        let updated = CacheKey::for_path(&path).unwrap();

        assert_eq!(
            updated.size, original.size,
            "sizes should match for this test"
        );
        assert!(
            cache.get(&path).is_none(),
            "modification time did not invalidate a same-size edit"
        );
    }

    #[test]
    fn test_cache_cleanup() {
        let project = TestProject::new("test_project").unwrap();
        let file_path = project.create_file("test.rs", "fn main() {}").unwrap();

        let mut cache = FileCache::new();
        cache.insert(file_path.clone(), stats(1)).unwrap();
        assert_eq!(cache.size(), 1);

        fs::remove_file(&file_path).unwrap();
        cache.cleanup_missing_files();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn keys_from_metadata_and_path_agree() {
        let project = TestProject::new("keys").unwrap();
        let path = project.create_file("a.rs", "fn a() {}\n").unwrap();
        let metadata = fs::metadata(&path).unwrap();

        assert_eq!(
            CacheKey::for_path(&path).unwrap(),
            CacheKey::from_metadata(&metadata).unwrap()
        );
    }

    #[test]
    fn round_trips_through_json() {
        let mut cache = FileCache::new();
        cache.insert_with_key(
            PathBuf::from("/a/b.rs"),
            stats(7),
            CacheKey {
                size: 7,
                modified: 42,
            },
        );

        let encoded = serde_json::to_string(&cache).unwrap();
        let decoded: FileCache = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.size(), 1);
        assert_eq!(
            decoded
                .get_with_key(
                    Path::new("/a/b.rs"),
                    &CacheKey {
                        size: 7,
                        modified: 42
                    }
                )
                .unwrap()
                .total_lines,
            7
        );
    }

    /// A truncated or foreign cache must be discarded, never surfaced as an
    /// error that fails the run.
    #[test]
    fn corrupt_and_stale_caches_are_discarded() {
        for payload in [
            "",
            "{",
            "not json at all",
            r#"{"entries":{},"cache_version":1}"#,
            r#"{"entries":{},"cache_version":999999}"#,
        ] {
            let parsed = serde_json::from_str::<FileCache>(payload)
                .ok()
                .filter(|c| c.cache_version == FileCache::CACHE_VERSION);
            assert!(
                parsed.is_none(),
                "payload {payload:?} should not be accepted as a current cache"
            );
        }
    }

    #[test]
    fn save_and_load_round_trip_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        temp_env_cache_dir(dir.path(), || {
            let mut cache = FileCache::scoped(project.path());
            cache.insert_with_key(
                PathBuf::from("/x/y.rs"),
                stats(3),
                CacheKey {
                    size: 3,
                    modified: 9,
                },
            );
            cache.save().unwrap();

            let loaded = FileCache::scoped(project.path());
            assert_eq!(loaded.size(), 1);
        });
    }

    /// An in-memory cache has nowhere to save to, and must not invent a location.
    #[test]
    fn an_in_memory_cache_is_never_written() {
        let dir = tempfile::tempdir().unwrap();
        temp_env_cache_dir(dir.path(), || {
            FileCache::new().save().unwrap();
            let written = fs::read_dir(FileCache::cache_dir().unwrap())
                .map(|entries| entries.count())
                .unwrap_or(0);
            assert_eq!(written, 0, "an in-memory cache wrote files to disk");
        });
    }

    /// Saving must not leave temporary files behind.
    #[test]
    fn save_leaves_no_temporary_files() {
        let dir = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        temp_env_cache_dir(dir.path(), || {
            FileCache::scoped(project.path()).save().unwrap();

            // Matched on the `.tmp.<pid>` extension `save` uses, not on the
            // substring "tmp": the project directory in this test is itself a
            // temporary directory, so its name is inside the real cache file's.
            let leftovers: Vec<_> = fs::read_dir(FileCache::cache_dir().unwrap())
                .unwrap()
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.contains(".tmp."))
                .collect();
            assert!(
                leftovers.is_empty(),
                "temporary files left behind: {leftovers:?}"
            );
        });
    }

    /// A corrupt cache on disk must load as empty rather than propagate.
    #[test]
    fn load_recovers_from_a_corrupt_file_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        temp_env_cache_dir(dir.path(), || {
            let cache_path = FileCache::cache_path_for(project.path()).unwrap();
            fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
            fs::write(&cache_path, b"\x00\x01 not json").unwrap();

            assert!(FileCache::scoped(project.path()).is_empty());
        });
    }

    /// Two projects must not share a cache: that is what made the cache cost more
    /// than it saved, and what let a large project evict a small one's entries.
    #[test]
    fn each_project_root_gets_its_own_cache() {
        let dir = tempfile::tempdir().unwrap();
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        temp_env_cache_dir(dir.path(), || {
            let key = CacheKey {
                size: 1,
                modified: 1,
            };
            let mut first = FileCache::scoped(a.path());
            first.insert_with_key(PathBuf::from("/a/one.rs"), stats(1), key);
            first.save().unwrap();

            assert_eq!(FileCache::scoped(a.path()).size(), 1);
            assert_eq!(
                FileCache::scoped(b.path()).size(),
                0,
                "a second project must start with an empty cache"
            );
        });
    }

    /// `.` and an absolute path naming the same directory must share one cache,
    /// otherwise running from inside a project never hits what running from
    /// outside it stored.
    #[test]
    fn equivalent_roots_resolve_to_one_cache() {
        let project = tempfile::tempdir().unwrap();
        let nested = project.path().join("sub");
        fs::create_dir_all(&nested).unwrap();

        // Only the file name is compared: the directory comes from a process-wide
        // environment variable that other tests are entitled to change.
        let direct = FileCache::cache_path_for(&nested).unwrap();
        let indirect = FileCache::cache_path_for(&project.path().join("sub/../sub")).unwrap();
        assert_eq!(direct.file_name(), indirect.file_name());
    }

    /// The per-project files must not accumulate without bound.
    #[test]
    fn old_project_caches_are_pruned() {
        let dir = tempfile::tempdir().unwrap();
        temp_env_cache_dir(dir.path(), || {
            let scopes = FileCache::cache_dir().unwrap();
            fs::create_dir_all(&scopes).unwrap();
            for i in 0..(MAX_SCOPES + 10) {
                fs::write(scopes.join(format!("stale-{i:04}.json")), b"{}").unwrap();
            }

            let project = tempfile::tempdir().unwrap();
            FileCache::scoped(project.path()).save().unwrap();

            let remaining = fs::read_dir(&scopes).unwrap().count();
            assert!(
                remaining <= MAX_SCOPES + 1,
                "cache directory grew unbounded: {remaining} files"
            );
        });
    }

    #[test]
    fn eviction_bounds_the_cache() {
        let mut cache = FileCache::new();
        for i in 0..(MAX_ENTRIES + 500) {
            cache.insert_with_key(
                PathBuf::from(format!("/f/{i}.rs")),
                stats(1),
                CacheKey {
                    size: 1,
                    modified: i as u64,
                },
            );
        }
        assert!(
            cache.size() <= MAX_ENTRIES,
            "cache grew past its bound: {}",
            cache.size()
        );
        // The most recent insertion must survive eviction.
        assert!(cache
            .entries
            .contains_key(Path::new(&format!("/f/{}.rs", MAX_ENTRIES + 499))));
    }

    /// `HOWMANY_CACHE_DIR` must fully redirect the cache so that runs can be
    /// isolated from the user's real cache.
    #[test]
    fn cache_dir_can_be_redirected() {
        let dir = tempfile::tempdir().unwrap();
        temp_env_cache_dir(dir.path(), || {
            let path = FileCache::cache_path_for(Path::new(".")).unwrap();
            assert!(
                path.starts_with(dir.path()),
                "cache path {path:?} ignored HOWMANY_CACHE_DIR"
            );
        });
    }

    /// Serialised guard around the process-wide cache-dir override.
    fn temp_env_cache_dir(dir: &Path, body: impl FnOnce()) {
        use std::sync::Mutex;
        static LOCK: Mutex<()> = Mutex::new(());
        let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let previous = std::env::var_os("HOWMANY_CACHE_DIR");
        std::env::set_var("HOWMANY_CACHE_DIR", dir);
        body();
        match previous {
            Some(value) => std::env::set_var("HOWMANY_CACHE_DIR", value),
            None => std::env::remove_var("HOWMANY_CACHE_DIR"),
        }
    }
}
