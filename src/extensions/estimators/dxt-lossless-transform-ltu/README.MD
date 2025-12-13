# DXT Lossless Transform - Lossless Transform Utils

This package provides a fast size estimation implementation for DXT lossless transform using the `lossless-transform-utils` library.

This is a fast estimator for LZ based compression algorithms.

## Overview

The lossless-transform-utils estimator provides fast compression size estimation by analyzing LZ match patterns in the data.

This approach is significantly faster than performing actual compression while maintaining
reasonable accuracy for optimization purposes.

## Important: Relative Comparison Only

This estimator is designed for relative comparison between different transforms of the same data.
The absolute values returned are not meaningful - only the relative ordering matters for determining which transform compresses better.

In other words, only compare against other results from this estimator.

The estimation is based on a simple formula: `data.len().saturating_sub(num_lz_matches)`.
More LZ matches indicate better compressibility, resulting in a lower estimated size.

## Usage

```rust
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create estimator
let estimator = LosslessTransformUtilsSizeEstimation::new();

// Example data
let data = vec![0u8; 64];

// Use with the common API - no buffer needed since max_compressed_size is always 0
let estimated_size = unsafe {
    estimator.estimate_compressed_size(
        data.as_ptr(),
        data.len(),
        core::ptr::null_mut(),
        0,
    )?
};
# Ok(())
# }
```

## Algorithm

The estimator uses a simple but effective approach:

1. Analyze the input data for LZ match patterns using `lossless-transform-utils`
2. Return `data.len().saturating_sub(num_lz_matches)` as the estimated size
3. More matches = lower estimated size = "better compression potential"

This provides good relative ordering for optimization while being very fast to compute.

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

- `dltltu_new_size_estimator()` - Create a new estimator
- `dltltu_free_size_estimator(estimator)` - Free the estimator

## Dependencies

- `lossless-transform-utils`: Core analysis functionality
- `dxt-lossless-transform-api-common`: Common API definitions
- `thiserror`: Error handling
