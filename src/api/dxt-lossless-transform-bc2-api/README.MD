# BC2 Lossless Transform API

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform-bc2-api.svg)](https://crates.io/crates/dxt-lossless-transform-bc2-api)
[![Docs.rs](https://docs.rs/dxt-lossless-transform-bc2-api/badge.svg)](https://docs.rs/dxt-lossless-transform-bc2-api)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

This crate provides a fast, visually lossless transform for the BC2 block format.
This is the high level API for the BC2 transform.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
dxt-lossless-transform-bc2-api = "0.1"
dxt-lossless-transform-ltu = "0.1"  # For size estimation (if needed)
```

### Basic Usage

How to transform/untransform BC2 data:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dxt_lossless_transform_bc2_api::Bc2AutoTransformBuilder;
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_api_common::allocate::allocate_cache_line_aligned;

// Use cache-line aligned allocation for best performance
let bc2_buffer = allocate_cache_line_aligned(16 * 500)?; // 500 BC2 blocks
// Fill with your BC2 data...
let mut output_buffer = allocate_cache_line_aligned(16 * 500)?;

// Transform with automatically optimized settings
let estimator = LosslessTransformUtilsSizeEstimation::new();
let untransform_builder = Bc2AutoTransformBuilder::new(estimator)
    .use_all_decorrelation_modes(false) // Fast mode
    .transform(bc2_buffer.as_slice(), output_buffer.as_mut_slice())?;

// output_buffer now contains the optimally transformed data
# Ok(())
# }
```

Run the transform before passing it to your compressor.
Run the untransform after decompressing.

Recommended estimators ***for ZStandard compression***:

| Estimator                                | Crate                         | Speed*       | Accuracy | Use Case                                 |
| ---------------------------------------- | ----------------------------- | ------------ | -------- | ---------------------------------------- |
| [`LosslessTransformUtilsSizeEstimation`] | `dxt-lossless-transform-ltu`  | ~2036 MiB/s  | Ok       | Fast optimization, real-time use         |
| [`ZStandardSizeEstimation`] level 1      | `dxt-lossless-transform-zstd` | ~530 MiB/s   | Higher   | Optimal compression ratio/speed tradeoff |

<sub>*Speed measured on Ryzen 9950X3D, single thread.</sub>

**Quick recommendation**: Use [`LosslessTransformUtilsSizeEstimation`] for real-time packing (zstd level < 4), else [`ZStandardSizeEstimation`] with level 1.

Higher compression ratios are possible with [`ZStandardSizeEstimation`], but the gains beyond level 1 are minimal; 
they should be used for final publishing only alongside other brute force options.

[`LosslessTransformUtilsSizeEstimation`]: https://docs.rs/dxt-lossless-transform-ltu/latest/dxt_lossless_transform_ltu/struct.LosslessTransformUtilsSizeEstimation.html
[`ZStandardSizeEstimation`]: https://docs.rs/dxt-lossless-transform-zstd/latest/dxt_lossless_transform_zstd/struct.ZStandardSizeEstimation.html

## API Reference

### Core Transform Functions

- [`Bc2AutoTransformBuilder::transform`] - Transform BC2 data with automatically optimized settings
- [`Bc2ManualTransformBuilder::transform`] - Transform BC2 data using configured settings
- [`Bc2ManualTransformBuilder::untransform`] - Untransform BC2 data using configured settings

### Automatic Optimization

- [`Bc2AutoTransformBuilder`] - Builder pattern for automatic optimization settings
  - `new(estimator)` - Create a new automatic transform builder with the provided estimator
  - `use_all_decorrelation_modes(bool)` - Configure thoroughness vs speed tradeoff
  - `transform(input, output)` - Transform data with optimal settings and return a configured manual builder for untransformation

### Manual Configuration

- [`Bc2ManualTransformBuilder`] - Builder pattern for manual transform configuration
  - `new()` - Create a new manual transform builder  
  - `decorrelation_mode(YCoCgVariant)` - Set the decorrelation mode
  - `split_colour_endpoints(bool)` - Set whether to split color endpoints
  - `split_alpha_blocks(bool)` - Set whether to split alpha blocks
  - `transform(input, output)` - Transform BC2 data using configured settings
  - `untransform(input, output)` - Untransform BC2 data using configured settings

### Core Types

- [`YCoCgVariant`] - Color decorrelation mode variants

### Advanced API (Core Crate)

For advanced users who need direct access to transform functions:

- **Core Crate Functions**: For ABI-unstable but safe slice-based functions, see `dxt-lossless-transform-bc2::transform::safe`
- **Maximum Performance**: For unsafe pointer-based functions, see `dxt-lossless-transform-bc2::transform`

**⚠️ Note**: Functions in the core crate may have breaking changes between versions. For production use, prefer the builder patterns above.

### Error Handling

All functions return [`Result`] types with detailed error information:

- [`Bc2Error::InvalidLength`] - Input data not divisible by 16 bytes
- [`Bc2Error::OutputBufferTooSmall`] - Output buffer insufficient for operation
- [`Bc2Error::AllocationFailed`] - Memory allocation error
- [`Bc2Error::SizeEstimationFailed`] - Size estimation error during optimization (preserves the actual estimator error)

## Usage Examples

### Manual Transform Configuration

When you know the optimal settings for your data, you can configure transforms manually. The builder can be reused as needed:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dxt_lossless_transform_bc2_api::{
    Bc2ManualTransformBuilder, YCoCgVariant
};

let bc2_data = vec![0u8; 16 * 100]; // 100 BC2 blocks
let mut transformed = vec![0u8; bc2_data.len()];
let mut restored = vec![0u8; bc2_data.len()];

// Create reusable builder with specific settings
let builder = Bc2ManualTransformBuilder::new()
    .decorrelation_mode(YCoCgVariant::Variant1)
    .split_colour_endpoints(true);

// Transform the data
builder.transform(&bc2_data, &mut transformed)?;

// Later, after decompression, restore original data
builder.untransform(&transformed, &mut restored)?;

// Restored data should match original
assert_eq!(bc2_data, restored);
# Ok(())
# }
```

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
# use dxt_lossless_transform_bc2_api::Bc2AutoTransformBuilder;
# use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
# let data = vec![0u8; 16 * 100];
# let mut output_fast = vec![0u8; 16 * 100];
# let mut output_thorough = vec![0u8; 16 * 100];

let estimator = LosslessTransformUtilsSizeEstimation::new();

// Fast mode: ~2x faster, tests common configurations
let fast_untransform_builder = Bc2AutoTransformBuilder::new(estimator)
    .use_all_decorrelation_modes(false)
    .transform(&data, &mut output_fast)?;

// Thorough mode: tests all decorrelation modes for best compression
// (typical gains are <0.1%; consider using estimator level closer to final compression level instead)
let estimator2 = LosslessTransformUtilsSizeEstimation::new();
let thorough_untransform_builder = Bc2AutoTransformBuilder::new(estimator2)
    .use_all_decorrelation_modes(true)
    .transform(&data, &mut output_thorough)?;
# Ok(())
# }
```

## Integration Examples

### Error Handling Patterns

```rust
# fn main() {
use dxt_lossless_transform_bc2_api::{Bc2AutoTransformBuilder, Bc2Error};
# use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
# let input = vec![0u8; 16 * 10];
# let mut output = vec![0u8; 16 * 10];

let estimator = LosslessTransformUtilsSizeEstimation::new();

match Bc2AutoTransformBuilder::new(estimator)
    .use_all_decorrelation_modes(false)
    .transform(&input, &mut output) {
    Ok(untransform_builder) => println!("Transform successful"),
    Err(Bc2Error::InvalidLength(len)) => {
        eprintln!("Invalid input: {} bytes (must be divisible by 16)", len);
    }
    Err(Bc2Error::OutputBufferTooSmall { needed, actual }) => {
        eprintln!("Buffer too small: need {} bytes, have {}", needed, actual);
    }
    Err(e) => eprintln!("Transform failed: {}", e),
}
# }
```

## CLI Tool

You can use the `dxt-lossless-transform-cli` tool to perform transforms on DDS files:

```bash
cargo install dxt-lossless-transform-cli

# Transform textures in a directory
dxt-lossless-transform-cli transform --input textures --output textures-transformed
```

## Expected Performance Results

### Compression Stats

*Coming soon - comprehensive compression statistics will be provided after benchmarking is complete.*

### Load Times

*Coming soon - detailed load time benchmarks will be provided after benchmarking is complete.*

## Features

- `std` (default): Enables standard library support
- `c-exports`: Enables C API exports for FFI compatibility

## License

Licensed under [GPL v3 (with Reloaded FAQ)](https://github.com/Sewer56/dxt-lossless-transform/blob/main/LICENSE).

For more information about licensing, see the [Reloaded project licensing guide][reloaded-license].

## C API (FFI)

For C/C++/FFI usage, see the `C API documentation` in the `c_api` module.

## Development

For information on how to work with this codebase, see the [Developer Manual](https://reloaded-project.github.io/reloaded-templates-rust/manual/).

[reloaded-license]: https://reloaded-project.github.io/License/GPLv3/about.html