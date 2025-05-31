use super::compression::CompressionAlgorithm;
use crate::error::TransformError;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

/// Cache for storing compressed data files on disk.
///
/// This cache stores actual compressed data files in a dedicated subdirectory, using content hashes,
/// compression levels, and compression algorithms as filenames. This allows for fast retrieval of
/// previously compressed data during benchmarks without needing to recompress the same data multiple times.
pub struct CompressedDataCache {
    /// Directory where compressed data files are stored
    cache_dir: PathBuf,
}

impl CompressedDataCache {
    /// Creates a new compressed data cache with default directory path.
    pub fn new() -> Self {
        // Create cache directory in user's cache dir or fallback to current dir (Windows, etc.)
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("dxt-lossless-transform-cli")
            .join("compressed_data");

        Self { cache_dir }
    }

    /// Ensures the cache directory exists.
    pub fn ensure_cache_dir(&self) -> Result<(), TransformError> {
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| TransformError::Debug(format!("Failed to create cache directory: {e}")))
    }

    /// Generates a cache key (filename) for the given content hash, compression level, and algorithm.
    fn cache_key(
        &self,
        content_hash: u128,
        compression_level: i32,
        algorithm: CompressionAlgorithm,
    ) -> PathBuf {
        let filename = format!(
            "{content_hash:32x}_{compression_level}.{}",
            algorithm.file_extension()
        );
        self.cache_dir.join(filename)
    }

    /// Loads compressed data from cache if it exists.
    ///
    /// Returns the compressed data and its size, or [`None`] if not cached.
    /// Takes a pre-calculated content hash to avoid recalculation.
    #[allow(clippy::type_complexity)]
    pub fn load_compressed_data(
        &self,
        content_hash: u128,
        compression_level: i32,
        algorithm: CompressionAlgorithm,
    ) -> Result<Option<(Box<[u8]>, usize)>, TransformError> {
        let cache_file = self.cache_key(content_hash, compression_level, algorithm);

        if !cache_file.exists() {
            return Ok(None);
        }

        let mut file = File::open(&cache_file)
            .map_err(|e| TransformError::Debug(format!("Failed to open cached file: {e}")))?;

        let mut compressed_data = Vec::new();
        file.read_to_end(&mut compressed_data)
            .map_err(|e| TransformError::Debug(format!("Failed to read cached file: {e}")))?;

        let compressed_size = compressed_data.len();
        let compressed_box = compressed_data.into_boxed_slice();

        Ok(Some((compressed_box, compressed_size)))
    }

    /// Saves compressed data to cache.
    /// Takes a pre-calculated content hash to avoid recalculation.
    pub fn save_compressed_data(
        &self,
        content_hash: u128,
        compression_level: i32,
        algorithm: CompressionAlgorithm,
        compressed_data: &[u8],
    ) -> Result<(), TransformError> {
        self.ensure_cache_dir()?;

        let cache_file = self.cache_key(content_hash, compression_level, algorithm);

        let mut file = File::create(&cache_file)
            .map_err(|e| TransformError::Debug(format!("Failed to create cache file: {e}")))?;

        file.write_all(compressed_data)
            .map_err(|e| TransformError::Debug(format!("Failed to write cache file: {e}")))?;

        Ok(())
    }

    /// Returns the number of cached files in the cache directory.
    pub fn cache_count(&self) -> usize {
        if !self.cache_dir.exists() {
            return 0;
        }

        let valid_extensions: Vec<&str> = CompressionAlgorithm::all_values()
            .iter()
            .map(|alg| alg.file_extension())
            .collect();

        fs::read_dir(&self.cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry.path().is_file()
                            && entry.path().extension().is_some_and(|ext| {
                                valid_extensions.contains(&ext.to_str().unwrap_or(""))
                            })
                    })
                    .count()
            })
            .unwrap_or(0)
    }
}
