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

/// Compresses data using the specified algorithm.
///
/// This function dispatches the compression task to the appropriate
/// implementation based on the `CompressionAlgorithm`.
///
/// # Parameters
/// * `data_ptr` - Pointer to the data to compress.
/// * `len_bytes` - Length of the data in bytes.
/// * `algorithm` - The compression algorithm to use.
/// * `compression_level` - Compression level (algorithm-specific).
///
/// # Returns
/// A [`Result`] containing a tuple of (compressed_data, compressed_size),
/// or a [`TransformError`] if compression is not supported or fails.
pub fn compress_with_algorithm(
    data_ptr: *const u8,
    len_bytes: usize,
    algorithm: CompressionAlgorithm,
    compression_level: i32,
) -> Result<(Box<[u8]>, usize), TransformError> {
    match algorithm {
        CompressionAlgorithm::ZStandard => {
            ZStandardCompression.compress_data(data_ptr, len_bytes, compression_level)
        }
        CompressionAlgorithm::LosslessTransformUtils => todo!(),
    }
}

/// Decompresses data using the specified algorithm.
///
/// This function dispatches the decompression task to the appropriate
/// implementation based on the `CompressionAlgorithm`.
///
/// # Parameters
/// * `compressed_data` - The compressed data.
/// * `output_buffer` - Buffer to decompress into.
/// * `algorithm` - The compression algorithm to use.
///
/// # Returns
/// A [`Result`] containing the number of bytes decompressed,
/// or a [`TransformError`] if decompression is not supported or fails.
pub fn decompress_with_algorithm(
    compressed_data: &[u8],
    output_buffer: &mut [u8],
    algorithm: CompressionAlgorithm,
) -> Result<usize, TransformError> {
    match algorithm {
        CompressionAlgorithm::ZStandard => {
            ZStandardCompression.decompress_data(compressed_data, output_buffer)
        }
        CompressionAlgorithm::LosslessTransformUtils => todo!(),
    }
}
