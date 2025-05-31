//! Compression algorithm abstraction for making the benchmarking system agnostic to compression methods.

use crate::error::TransformError;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use zstd::ZStandardCompression;

pub mod helpers;
pub mod zstd;

/// Supported compression algorithms for benchmarking and estimation.
/// Some algorithms are both estimate and compress, some are estimate only.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    bincode::Encode,
    bincode::Decode,
    Default,
)]
pub enum CompressionAlgorithm {
    /// ZStandard compression
    #[default]
    ZStandard,
    /// Estimate using lossless-transform-utils. Compression not supported.
    LosslessTransformUtils,
}

impl CompressionAlgorithm {
    /// Returns the file extension typically used for this compression algorithm.
    pub fn file_extension(&self) -> &'static str {
        match self {
            CompressionAlgorithm::ZStandard => "zst",
            CompressionAlgorithm::LosslessTransformUtils => "ltu", // not a compression algorithm, but an estimation method
        }
    }

    /// Returns a human-readable name for this compression algorithm.
    pub fn name(&self) -> &'static str {
        match self {
            CompressionAlgorithm::ZStandard => "ZStandard",
            CompressionAlgorithm::LosslessTransformUtils => "lossless-transform-utils",
        }
    }

    /// Returns the default compression level for this algorithm.
    pub fn default_compression_level(&self) -> i32 {
        match self {
            CompressionAlgorithm::ZStandard => 16,
            CompressionAlgorithm::LosslessTransformUtils => 0, // Not applicable, as this is an estimation method
        }
    }

    /// Returns the default estimation compression level for this algorithm.
    pub fn default_estimate_compression_level(&self) -> i32 {
        match self {
            CompressionAlgorithm::ZStandard => 3,
            CompressionAlgorithm::LosslessTransformUtils => 0, // Not applicable, as this is an estimation method
        }
    }

    /// Returns all available compression algorithms.
    pub fn all() -> &'static [CompressionAlgorithm] {
        &[
            CompressionAlgorithm::ZStandard,
            CompressionAlgorithm::LosslessTransformUtils,
        ]
    }

    /// Checks if this compression algorithm supports actual compression.
    pub fn supports_compress(&self) -> bool {
        match self {
            CompressionAlgorithm::ZStandard => true,
            CompressionAlgorithm::LosslessTransformUtils => false,
        }
    }
}

impl fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for CompressionAlgorithm {
    type Err = TransformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zstd" | "zstandard" => Ok(CompressionAlgorithm::ZStandard),
            "ltu" | "lossless-transform-utils" => Ok(CompressionAlgorithm::LosslessTransformUtils),
            _ => Err(TransformError::Debug(format!(
                "Unknown compression algorithm: {s}. Available: zstd, ltu",
            ))),
        }
    }
}

/// Trait for compression operations with different algorithms.
pub trait CompressionOperations {
    /// Compresses data and returns the compressed data and size.
    ///
    /// # Parameters
    /// * `data_ptr` - Pointer to the data to compress
    /// * `len_bytes` - Length of the data in bytes
    /// * `compression_level` - Compression level (algorithm-specific)
    ///
    /// # Returns
    /// A tuple of (compressed_data, compressed_size)
    fn compress_data(
        &self,
        data_ptr: *const u8,
        len_bytes: usize,
        compression_level: i32,
    ) -> Result<(Box<[u8]>, usize), TransformError>;

    /// Decompresses data into a pre-allocated buffer.
    ///
    /// # Parameters
    /// * `compressed_data` - The compressed data
    /// * `output_buffer` - Buffer to decompress into
    ///
    /// # Returns
    /// The number of bytes decompressed
    fn decompress_data(
        &self,
        compressed_data: &[u8],
        output_buffer: &mut [u8],
    ) -> Result<usize, TransformError>;
}

/// Factory for creating compression operation instances.
pub fn create_compression_operations(
    algorithm: CompressionAlgorithm,
) -> Box<dyn CompressionOperations> {
    match algorithm {
        CompressionAlgorithm::ZStandard => Box::new(ZStandardCompression),
        CompressionAlgorithm::LosslessTransformUtils => panic!(
            "LosslessTransformUtils is not a compression algorithm, only an estimation method"
        ),
    }
}
