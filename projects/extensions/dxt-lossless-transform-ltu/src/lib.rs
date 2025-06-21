#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "c-exports")]
pub mod c_api;

use core::slice;
use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};
use dxt_lossless_transform_common::allocate::AllocateError;
use lossless_transform_utils::{entropy::*, histogram::*, match_estimator::*};
use thiserror::Error;

/// Errors that can occur during lossless-transform-utils size estimation.
#[derive(Debug, Error)]
pub enum LosslessTransformUtilsError {
    /// Error during allocation
    #[error(transparent)]
    AllocateError(#[from] AllocateError),

    /// Invalid input data
    #[error("Invalid input data")]
    InvalidInput,
}

/// Settings for size estimation multipliers.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EstimationSettings {
    /// Multiplier for LZ match effectiveness (default: 0.5809)
    pub lz_match_multiplier: f64,
    /// Multiplier for entropy coding efficiency (default: 1.1670)
    pub entropy_multiplier: f64,
}

/// Runtime analysis parameters for size estimation calculation.
#[derive(Debug, Clone)]
pub struct SizeEstimationParameters {
    /// Length of the original data in bytes
    pub data_len: usize,
    /// Number of LZ matches found in the data
    pub num_lz_matches: usize,
    /// Entropy value (bits per byte) from histogram analysis
    pub entropy: f64,
}

/// Lossless Transform Utils implementation of [`SizeEstimationOperations`].
///
/// This implementation uses the [`lossless_transform_utils`] library to provide
/// fast size estimation based on LZ match analysis and entropy calculation.
///
/// It's significantly faster than performing actual compression while still
/// providing reasonable accuracy for optimization purposes.
///
/// # Important: Texture-Specific Implementation
///
/// **This estimator is specifically tuned for DXT/BC texture data and may not work well with generic data.**
/// The default parameters have been carefully calibrated for texture data passed in via the various
/// `determine_optimal_transform` functions. (The [`DataType`] field is used to determine the settings.)
///
/// **Using [`LosslessTransformUtilsSizeEstimation::new_with_params`] or custom settings is discouraged**
/// unless you have conducted thorough testing with your specific data type and understand
/// the estimation model. The default settings via [`LosslessTransformUtilsSizeEstimation::new`]
/// should be used in most cases.
pub struct LosslessTransformUtilsSizeEstimation {
    /// Optional configuration settings for estimation multipliers
    /// If None, hardcoded defaults are used based on DataType
    settings: Option<EstimationSettings>,
}

impl LosslessTransformUtilsSizeEstimation {
    /// Creates a new [lossless_transform_utils] size estimator with default settings.
    pub fn new() -> Self {
        Self { settings: None }
    }

    /// Creates a new [lossless_transform_utils] size estimator with custom settings.
    ///
    /// # Warning
    /// **Using custom settings is discouraged unless you have conducted thorough testing**
    /// with your specific data type. The default estimator is tuned for DXT/BC texture data.
    /// Consider using [`Self::new`] instead.
    ///
    /// # Parameters
    /// * `settings` - Custom estimation settings
    pub fn new_with_settings(settings: EstimationSettings) -> Self {
        Self {
            settings: Some(settings),
        }
    }

    /// Creates a new lossless-transform-utils size estimator with custom parameters.
    ///
    /// # Warning
    /// **Using custom parameters is discouraged unless you have conducted thorough testing**
    /// with your specific data type. The default estimator is tuned for DXT/BC texture data,
    /// whose type is passed in via the [`DataType`] field.
    /// Consider using [`Self::new`] instead.
    ///
    /// # Parameters
    ///
    /// * `lz_match_multiplier` - Multiplier for LZ match effectiveness
    /// * `entropy_multiplier` - Multiplier for entropy coding efficiency
    pub fn new_with_params(lz_match_multiplier: f64, entropy_multiplier: f64) -> Self {
        Self {
            settings: Some(EstimationSettings {
                lz_match_multiplier,
                entropy_multiplier,
            }),
        }
    }

    /// Gets estimation settings for the given data type.
    /// Returns user-supplied settings if provided, otherwise returns hardcoded defaults for the data type.
    ///
    /// # Parameters
    /// * `data_type` - The type of data being estimated
    ///
    /// # Returns
    /// The estimation settings to use
    pub(crate) fn get_settings_for_data_type(&self, data_type: DataType) -> EstimationSettings {
        // If user provided settings, use those
        if let Some(user_settings) = self.settings {
            return user_settings;
        }

        // Otherwise, use hardcoded defaults based on data type
        // TODO: Adjust these slightly as needed.
        match data_type {
            DataType::Bc1Colours => EstimationSettings {
                lz_match_multiplier: 0.5809,
                entropy_multiplier: 1.1670,
            },
            DataType::Bc1DecorrelatedColours => EstimationSettings {
                lz_match_multiplier: 0.5809,
                entropy_multiplier: 1.1670,
            },
            DataType::Bc1SplitColours => EstimationSettings {
                lz_match_multiplier: 0.5809,
                entropy_multiplier: 1.1670,
            },
            DataType::Bc1SplitDecorrelatedColours => EstimationSettings {
                lz_match_multiplier: 0.5809,
                entropy_multiplier: 1.1670,
            },
            DataType::Unknown => EstimationSettings {
                lz_match_multiplier: 0.5809,
                entropy_multiplier: 1.1670,
            },
        }
    }

    /// Estimates the compressed size of data using the provided data type.
    ///
    /// # Arguments
    /// * `data` - The input data to analyze
    /// * `data_type` - The type of data being analyzed
    ///
    /// # Returns
    /// Estimated size in bytes after compression
    fn estimate_size(&self, data: &[u8], data_type: DataType) -> usize {
        if data.is_empty() {
            return 0;
        }

        // Get settings for this data type
        let settings = self.get_settings_for_data_type(data_type);

        // Estimate number of LZ matches
        let num_matches = estimate_num_lz_matches_fast(data);

        // Calculate entropy using histogram
        let mut histogram = Histogram32::default();
        histogram32_from_bytes(data, &mut histogram);
        let entropy = code_length_of_histogram32(&histogram, data.len() as u64);

        // Create runtime parameters for this estimation
        let params = SizeEstimationParameters {
            data_len: data.len(),
            num_lz_matches: num_matches,
            entropy,
        };

        size_estimate(params, settings)
    }
}

impl Default for LosslessTransformUtilsSizeEstimation {
    fn default() -> Self {
        Self::new()
    }
}

impl SizeEstimationOperations for LosslessTransformUtilsSizeEstimation {
    type Error = LosslessTransformUtilsError;

    fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
        // For estimation purposes, we don't need a compression buffer
        // since we're not actually compressing data
        Ok(0)
    }

    fn supports_data_type_differentiation(&self) -> bool {
        // LTU uses different estimation settings based on data type
        true
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        data_type: DataType,
        _output_ptr: *mut u8,
        _output_len: usize,
    ) -> Result<usize, Self::Error> {
        if input_ptr.is_null() {
            return Ok(0);
        }

        if len_bytes == 0 {
            return Ok(0);
        }

        // Safety: We've checked that input_ptr is not null and len_bytes > 0
        let data = slice::from_raw_parts(input_ptr, len_bytes);

        // Perform size estimation using the data type
        let estimated_size = self.estimate_size(data, data_type);

        Ok(estimated_size)
    }
}

/// Estimates the compressed size of data in bytes using the provided parameters.
///
/// # Arguments
/// * `params` - The runtime analysis parameters containing data length, LZ matches, and entropy
/// * `settings` - The estimation settings containing multipliers
///
/// # Returns
/// Estimated size in bytes after compression
pub fn size_estimate(params: SizeEstimationParameters, settings: EstimationSettings) -> usize {
    // Calculate expected bytes after LZ
    let bytes_after_lz = params
        .data_len
        .saturating_sub((params.num_lz_matches as f64 * settings.lz_match_multiplier) as usize);

    // Calculate expected bits and convert to bytes
    let estimated_bits = bytes_after_lz as f64 * params.entropy * settings.entropy_multiplier;
    let estimated_bytes = (estimated_bits / 8.0).ceil() as usize;

    // Ensure we return at least 1 byte for non-empty input
    if estimated_bytes == 0 && params.data_len > 0 {
        1
    } else {
        estimated_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_estimation_empty_data() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let result = unsafe {
            estimator.estimate_compressed_size(
                core::ptr::null(),
                0,
                DataType::Bc1Colours,
                core::ptr::null_mut(),
                0,
            )
        };
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_size_estimation_with_data() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let test_data = [0u8; 64]; // 64 bytes of zeros
        let result = unsafe {
            estimator.estimate_compressed_size(
                test_data.as_ptr(),
                test_data.len(),
                DataType::Bc1Colours,
                core::ptr::null_mut(),
                0,
            )
        };

        // Should return a reasonable estimate > 0
        assert!(result.is_ok());
        let size = result.unwrap();
        assert!(size > 0);
        assert!(size < test_data.len()); // Should be smaller than input for repetitive data
    }

    #[test]
    fn test_custom_settings() {
        let estimator = LosslessTransformUtilsSizeEstimation::new_with_params(0.6, 1.2);
        // Test that custom settings are used by checking the behavior
        assert!(estimator.settings.is_some());
        if let Some(settings) = estimator.settings {
            assert_eq!(settings.lz_match_multiplier, 0.6);
            assert_eq!(settings.entropy_multiplier, 1.2);
        }
    }

    #[test]
    fn test_default_settings_used_when_none_provided() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        assert!(estimator.settings.is_none());

        // Test that default settings are used for each data type
        let default_settings = estimator.get_settings_for_data_type(DataType::Bc1Colours);
        assert_eq!(default_settings.lz_match_multiplier, 0.5809);
        assert_eq!(default_settings.entropy_multiplier, 1.1670);

        // Test that all data types get the same default settings for now
        let data_types = [
            DataType::Bc1Colours,
            DataType::Bc1DecorrelatedColours,
            DataType::Bc1SplitColours,
            DataType::Bc1SplitDecorrelatedColours,
        ];

        for data_type in data_types {
            let settings = estimator.get_settings_for_data_type(data_type);
            assert_eq!(settings.lz_match_multiplier, 0.5809);
            assert_eq!(settings.entropy_multiplier, 1.1670);
        }
    }

    #[test]
    fn test_max_compressed_size() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let result = estimator.max_compressed_size(1024);
        assert_eq!(result.unwrap(), 0); // No buffer needed for estimation
    }

    #[test]
    fn test_size_estimate_function() {
        let params = SizeEstimationParameters {
            data_len: 100,
            num_lz_matches: 10,
            entropy: 4.0,
        };
        let settings = EstimationSettings {
            lz_match_multiplier: 0.5,
            entropy_multiplier: 1.0,
        };

        let estimate = size_estimate(params, settings);
        // Should be reasonable: (100 - 10*0.5) * 4.0 * 1.0 / 8 = 47.5 -> 48 bytes
        assert_eq!(estimate, 48);
    }

    #[test]
    fn test_size_estimate_non_zero_for_non_empty() {
        let params = SizeEstimationParameters {
            data_len: 1,
            num_lz_matches: 0,
            entropy: 0.0,
        };
        let settings = EstimationSettings {
            lz_match_multiplier: 0.5809,
            entropy_multiplier: 1.1670,
        };

        let estimate = size_estimate(params, settings);
        // Should return at least 1 byte for non-empty input
        assert_eq!(estimate, 1);
    }

    #[test]
    fn test_different_data_types() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let test_data = [0u8; 32];

        // Test with different data types (should all work with same settings for now)
        let data_types = [
            DataType::Bc1Colours,
            DataType::Bc1DecorrelatedColours,
            DataType::Bc1SplitColours,
            DataType::Bc1SplitDecorrelatedColours,
        ];

        for data_type in data_types {
            let result = unsafe {
                estimator.estimate_compressed_size(
                    test_data.as_ptr(),
                    test_data.len(),
                    data_type,
                    core::ptr::null_mut(),
                    0,
                )
            };
            assert!(result.is_ok());
            assert!(result.unwrap() > 0);
        }
    }
}
