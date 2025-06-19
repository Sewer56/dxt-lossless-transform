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

// Estimate compressed size
let estimated_size = unsafe {
    estimator.estimate_compressed_size(data.as_ptr(), data.len()).unwrap()
};
```