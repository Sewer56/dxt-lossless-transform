# DXT Lossless Transform - Lossless Transform Utils

This package provides a fast size estimation implementation for DXT lossless transform using the `lossless-transform-utils` library.

This is a fast estimator for LZ based compression algorithms.

## Overview

The lossless-transform-utils estimator provides fast compression size estimation by analyzing:

- LZ match patterns in the data
- Entropy characteristics through histogram analysis

This approach is significantly faster than performing actual compression while maintaining
reasonable accuracy for optimization purposes.

## Important: Texture-Specific Implementation

**This estimator is specifically tuned for DXT/BC texture data and may not work well with generic data.**
The default parameters have been carefully calibrated for texture compression patterns.
The profile is inherited from the [`DataType`] field passed in via the API.

**Using custom parameters via `new_with_params()` or `new_with_settings()` is discouraged** unless you have conducted thorough testing with your specific data type and understand the estimation model. The default settings via `new()` should be used in most cases.

## Usage

```rust
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_api_common::estimate::{SizeEstimationOperations, DataType};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create estimator with default parameters
let estimator = LosslessTransformUtilsSizeEstimation::new();

// Or with custom parameters
let estimator = LosslessTransformUtilsSizeEstimation::new_with_params(0.6, 1.2);

// Example data
let data = vec![0u8; 64];

// Use with the common API - no buffer needed since max_compressed_size is always 0
let estimated_size = unsafe {
    estimator.estimate_compressed_size(
        data.as_ptr(),
        data.len(),
        DataType::Bc1Colours,
        core::ptr::null_mut(),
        0,
    )?
};
# Ok(())
# }
```

## Default Parameters

The default parameters are determined based on the [`DataType`].

These were brute forced from a large dataset of various textures and images to provide decent accuracy.

## C API

When compiled with the `c-exports` feature, this crate provides C-compatible exports for using the LTU estimator from C/C++ code:

```c
// Create an LTU size estimator
DltSizeEstimator* ltu_estimator = dltltu_new_size_estimator();

// Use it with BC1 transform optimization
Dltbc1Result result = dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
    builder,
    bc1_data,
    data_len,
    ltu_estimator,  // Use LTU for fast estimation
    context
);

// Free the estimator when done
dltltu_free_size_estimator(ltu_estimator);
```

### C API Functions

- `dltltu_new_size_estimator()` - Create with default settings
- `dltltu_new_size_estimator_with_params(lz_match_multiplier, entropy_multiplier)` - Create with custom parameters
- `dltltu_new_size_estimator_with_settings(settings)` - Create with settings struct
- `dltltu_free_size_estimator(estimator)` - Free the estimator

## Dependencies

- `lossless-transform-utils`: Core analysis functionality
- `dxt-lossless-transform-api-common`: Common API definitions
- `thiserror`: Error handling
