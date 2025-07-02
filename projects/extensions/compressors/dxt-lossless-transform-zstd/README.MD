# dxt-lossless-transform-zstd

ZStandard size estimation implementation for DXT lossless transform library.

This crate provides a [`ZStandardSizeEstimation`] implementation of the 
[`SizeEstimationOperations`] trait from [`dxt-lossless-transform-api-common`].

## Usage

```rust
use dxt_lossless_transform_zstd::ZStandardSizeEstimation;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

let estimator = ZStandardSizeEstimation::new_default();
let data = b"Hello, world!";

// Get the maximum buffer size needed
let max_size = estimator
    .max_compressed_size(data.len())
    .unwrap();

// Allocate compression buffer
let mut output_buffer = vec![0u8; max_size];

// Estimate compressed size using the allocated buffer
let estimated_size = unsafe {
    estimator.estimate_compressed_size(
        data.as_ptr(), 
        data.len(), 
        output_buffer.as_mut_ptr(),
        max_size
    ).unwrap()
};
```