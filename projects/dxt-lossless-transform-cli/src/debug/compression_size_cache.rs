use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    path::PathBuf,
};

use crate::error::TransformError;

/// Simple compression cache that stores compressed sizes for specific inputs and compression levels.
///
/// This cache is format-agnostic and can be shared across all BC format analyses, as it is based on
/// input content hashes and compression levels rather than specific formats.
pub struct CompressionCache {
    /// Map from (content_hash, compression_level) -> compressed_size
    cache: HashMap<(u128, i32), usize>,
    /// Path to the cache file
    cache_file_path: PathBuf,
}

impl CompressionCache {
    /// Creates a new compression cache with default file path.
    pub fn new() -> Self {
        // Create cache directory in user's cache dir or fallback to current dir (Windows, etc.)
        let cache_dir = std::env::var("HOME")
            .map(|home| {
                PathBuf::from(home)
                    .join(".cache")
                    .join("dxt-lossless-transform-cli")
            })
            .unwrap_or_else(|_| PathBuf::from(".cache").join("dxt-lossless-transform-cli"));

        let cache_file_path = cache_dir.join("compression_size_cache.bin");

        Self {
            cache: HashMap::new(),
            cache_file_path,
        }
    }

    /// Loads the cache from disk if the cache file exists.
    pub fn load_from_disk(&mut self) -> Result<(), TransformError> {
        if !self.cache_file_path.exists() {
            return Ok(()); // No cache file yet
        }

        let mut file = File::open(&self.cache_file_path)
            .map_err(|e| TransformError::Debug(format!("Failed to open cache file: {e}")))?;

        self.cache = bincode::decode_from_std_read(&mut file, bincode::config::standard())
            .map_err(|e| TransformError::Debug(format!("Failed to deserialize cache: {e}")))?;

        Ok(())
    }

    /// Saves the cache to disk.
    pub fn save_to_disk(&self) -> Result<(), TransformError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.cache_file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TransformError::Debug(format!("Failed to create cache directory: {e}"))
            })?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cache_file_path)
            .map_err(|e| TransformError::Debug(format!("Failed to create cache file: {e}")))?;

        bincode::encode_into_std_write(&self.cache, &mut file, bincode::config::standard())
            .map_err(|e| TransformError::Debug(format!("Failed to serialize cache: {e}")))?;

        Ok(())
    }

    /// Gets a cached compression size for the given content hash and compression level.
    pub fn get(&self, content_hash: u128, compression_level: i32) -> Option<usize> {
        self.cache.get(&(content_hash, compression_level)).copied()
    }

    /// Inserts a compression size into the cache for the given content hash and compression level.
    pub fn insert(&mut self, content_hash: u128, compression_level: i32, compressed_size: usize) {
        self.cache
            .insert((content_hash, compression_level), compressed_size);
    }

    /// Returns the number of entries in the cache.
    pub fn len(&self) -> usize {
        self.cache.len()
    }
}
