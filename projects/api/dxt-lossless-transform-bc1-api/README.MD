# dxt-lossless-transform-bc1-api

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform-bc1-api.svg)](https://crates.io/crates/dxt-lossless-transform-bc1-api)
[![Docs.rs](https://docs.rs/dxt-lossless-transform-bc1-api/badge.svg)](https://docs.rs/dxt-lossless-transform-bc1-api)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

This crate provides a fast, visually lossless transform for the BC1 block format.
This is the high level API for the BC1 transform.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
dxt-lossless-transform-bc1-api = "0.1"
dxt-lossless-transform-ltu = "0.1"  # For size estimation (if needed)
```

### Basic Usage

How to transform/untransform BC1 data:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dxt_lossless_transform_bc1_api::{transform_bc1_slice, untransform_bc1_slice};
use dxt_lossless_transform_bc1::{Bc1TransformDetails, Bc1DetransformDetails};

// Your BC1 texture data (must be divisible by 8 bytes)
let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
let mut transformed = vec![0u8; bc1_data.len()];
let mut restored = vec![0u8; bc1_data.len()];

// Transform with default options
let transform_options = Bc1TransformDetails::default();
transform_bc1_slice(&bc1_data, &mut transformed, transform_options)?;

// Untransform back to original
let detransform_options = Bc1DetransformDetails::from(transform_options);
untransform_bc1_slice(&transformed, &mut restored, detransform_options)?;

assert_eq!(bc1_data, restored);
# Ok(())
# }
```

Run the transform before passing it to your compressor.
Run the untransform after decompressing.

### Automatic Optimization

This finds the optimal transform parameters for your data.

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dxt_lossless_transform_bc1_api::{determine_optimal_transform, transform_bc1_allocating};
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_api_common::allocate::allocate_cache_line_aligned;

// Use cache-line aligned allocation for best performance
let mut bc1_buffer = allocate_cache_line_aligned(8 * 500)?; // 500 BC1 blocks
// Fill with your BC1 data...

// Automatically determine the best transform parameters
let estimator = LosslessTransformUtilsSizeEstimation::new();
let optimal_options = determine_optimal_transform(bc1_buffer.as_slice(), estimator, false)?;

// Apply the optimal transform with automatic allocation
let transformed = transform_bc1_allocating(bc1_buffer.as_slice(), optimal_options)?;
# Ok(())
# }
```

Recommended estimators ***for ZStandard compression***:

| Estimator                                | Crate                         | Speed*     | Accuracy | Use Case                                 |
| ---------------------------------------- | ----------------------------- | ---------- | -------- | ---------------------------------------- |
| [`LosslessTransformUtilsSizeEstimation`] | `dxt-lossless-transform-ltu`  | ~641 MiB/s | Good     | Fast optimization, real-time use         |
| [`ZStandardSizeEstimation`] level 1      | `dxt-lossless-transform-zstd` | ~265 MiB/s | Higher   | Optimal compression ratio/speed tradeoff |

<sub>*Speed measured on Ryzen 9950X3D, single thread</sub>

**Quick recommendation**: Use [`LosslessTransformUtilsSizeEstimation`] for real-time packing (zstd level < 3), else [`ZStandardSizeEstimation`] with level 1.

Higher compression ratios are possible with [`ZStandardSizeEstimation`], but the gains beyond level 1 are minimal; 
they should be used for final publishing only alongside other brute force options.

[`LosslessTransformUtilsSizeEstimation`]: https://docs.rs/dxt-lossless-transform-ltu/latest/dxt_lossless_transform_ltu/struct.LosslessTransformUtilsSizeEstimation.html
[`ZStandardSizeEstimation`]: https://docs.rs/dxt-lossless-transform-zstd/latest/dxt_lossless_transform_zstd/struct.ZStandardSizeEstimation.html

## API Reference

### Core Transform Functions

- [`transform_bc1_slice`] - Transform BC1 data using provided buffers
- [`transform_bc1_allocating`] - Transform BC1 data with automatic allocation
- [`untransform_bc1_slice`] - Restore BC1 data using provided buffers  
- [`untransform_bc1_allocating`] - Restore BC1 data with automatic allocation

### Optimization Functions

- [`mod@determine_optimal_transform`] - Automatically find best transform parameters
- [`Bc1TransformOptionsBuilder`] - Builder pattern for custom configuration

### Error Handling

All functions return [`Result`] types with detailed [`Bc1Error`] information:

- [`Bc1Error::InvalidLength`] - Input data not divisible by 8 bytes
- [`Bc1Error::OutputBufferTooSmall`] - Output buffer insufficient for operation
- [`Bc1Error::AllocationFailed`] - Memory allocation error
- [`Bc1Error::SizeEstimationFailed`] - Size estimation error during optimization (preserves the actual estimator error)

## Performance Considerations

### Memory Alignment

For optimal SIMD performance, use cache-line aligned allocations:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dxt_lossless_transform_api_common::allocate::allocate_cache_line_aligned;

# let data_size = 1024;
// Optimal: cache-line aligned (64 bytes on x86/ARM)
let buffer = allocate_cache_line_aligned(data_size)?;

// Suboptimal: heap allocated, zeroed, may not be aligned
let buffer = vec![0u8; data_size];
# Ok(())
# }
```

### Transform Optimization

Choose between speed and quality:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use dxt_lossless_transform_bc1_api::determine_optimal_transform;
# use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
# let data = vec![0u8; 8 * 100];
# let estimator = LosslessTransformUtilsSizeEstimation::new();
// Fast mode: ~2-3x faster, tests common configurations
let fast_options = determine_optimal_transform(&data, estimator, false)?;

// Thorough mode: tests all decorrelation modes for best compression
# let estimator = LosslessTransformUtilsSizeEstimation::new();
let thorough_options = determine_optimal_transform(&data, estimator, true)?;
# Ok(())
# }
```

### Memory Management

Choose the right API for your use case:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use dxt_lossless_transform_bc1_api::{transform_bc1_slice, transform_bc1_allocating};
# use dxt_lossless_transform_bc1::Bc1TransformDetails;
# let input = vec![0u8; 8 * 10];
# let mut output = vec![0u8; 8 * 10];
# let options = Bc1TransformDetails::default();
// Use slice API when you manage memory
transform_bc1_slice(&input, &mut output, options)?;

// Use allocating API for convenience
let result = transform_bc1_allocating(&input, options)?;
# Ok(())
# }
```

## Integration Examples

### With Compression Libraries

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dxt_lossless_transform_bc1_api::{determine_optimal_transform, transform_bc1_allocating};
# use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;

# fn compress_with_zstd(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
#     Ok(data.to_vec()) // Simulate compression
# }

fn compress_bc1_texture(bc1_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Step 1: Find optimal transform
    let estimator = LosslessTransformUtilsSizeEstimation::new();
    let transform_options = determine_optimal_transform(bc1_data, estimator, false)?;
    
    // Step 2: Apply transform
    let transformed = transform_bc1_allocating(bc1_data, transform_options)?;
    
    // Step 3: Compress with your preferred library
    let compressed = compress_with_zstd(transformed.as_slice())?;
    
    Ok(compressed)
}

# let bc1_data = vec![0u8; 8 * 10];
# let result = compress_bc1_texture(&bc1_data)?;
# Ok(())
# }
```

### Error Handling Patterns

```rust
# fn main() {
use dxt_lossless_transform_bc1_api::{transform_bc1_slice, Bc1Error};
# use dxt_lossless_transform_bc1::Bc1TransformDetails;
# let input = vec![0u8; 8 * 10];
# let mut output = vec![0u8; 8 * 10];
# let options = Bc1TransformDetails::default();

match transform_bc1_slice(&input, &mut output, options) {
    Ok(()) => println!("Transform successful"),
    Err(Bc1Error::InvalidLength(len)) => {
        eprintln!("Invalid input: {} bytes (must be divisible by 8)", len);
    }
    Err(Bc1Error::OutputBufferTooSmall { needed, actual }) => {
        eprintln!("Buffer too small: need {} bytes, have {}", needed, actual);
    }
    Err(e) => eprintln!("Transform failed: {}", e),
}
# }
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with all features (requires nightly rust):

```bash
cargo test --all-features
```

## CLI Tool

You can use the `dxt-lossless-transform-cli` tool to perform transforms on DDS files:

```bash
cargo install dxt-lossless-transform-cli

# Transform textures in a directory
dxt-lossless-transform-cli transform --input textures --output textures-transformed
```

## License

Licensed under [GPL v3 (with Reloaded FAQ)](./LICENSE).

For more information about licensing, see the [Reloaded project licensing guide][reloaded-license].

## Development

For information on how to work with this codebase, see [README-DEV.MD][readme-dev].

[reloaded-license]: https://reloaded-project.github.io/Reloaded.MkDocsMaterial.Themes.R2/Pages/license.html
[readme-dev]: https://github.com/Sewer56/dxt-lossless-transform/blob/main/README-DEV.MD