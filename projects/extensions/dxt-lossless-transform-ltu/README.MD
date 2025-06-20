# DXT Lossless Transform - Lossless Transform Utils

This package provides a fast size estimation implementation for DXT lossless transform using the `lossless-transform-utils` library.

This is a fast estimator for LZ based compression algorithms.

## Overview

The lossless-transform-utils estimator provides fast compression size estimation by analyzing:

- LZ match patterns in the data
- Entropy characteristics through histogram analysis

This approach is significantly faster than performing actual compression while maintaining
reasonable accuracy for optimization purposes.

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

## Dependencies

- `lossless-transform-utils`: Core analysis functionality
- `dxt-lossless-transform-api-common`: Common API definitions
- `thiserror`: Error handling
