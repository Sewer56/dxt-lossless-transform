# BC1 Lossless Transform API

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
use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_api_common::allocate::allocate_cache_line_aligned;

// Use cache-line aligned allocation for best performance
let bc1_buffer = allocate_cache_line_aligned(8 * 500)?; // 500 BC1 blocks
// Fill with your BC1 data...
let mut output_buffer = allocate_cache_line_aligned(8 * 500)?;

// Transform with automatically optimized settings
let estimator = LosslessTransformUtilsSizeEstimation::new();
let untransform_builder = Bc1AutoTransformBuilder::new(estimator)
    .use_all_decorrelation_modes(false) // Fast mode
    .transform(bc1_buffer.as_slice(), output_buffer.as_mut_slice())?;

// output_buffer now contains the optimally transformed data
# Ok(())
# }
```

Run the transform before passing it to your compressor.
Run the untransform after decompressing.

Recommended estimators ***for ZStandard compression***:

| Estimator                                | Crate                         | Speed*      | Accuracy | Use Case                                 |
| ---------------------------------------- | ----------------------------- | ----------- | -------- | ---------------------------------------- |
| [`LosslessTransformUtilsSizeEstimation`] | `dxt-lossless-transform-ltu`  | ~1018 MiB/s | Ok       | Fast optimization, real-time use         |
| [`ZStandardSizeEstimation`] level 1      | `dxt-lossless-transform-zstd` | ~265 MiB/s  | Higher   | Optimal compression ratio/speed tradeoff |

<sub>*Speed measured on Ryzen 9950X3D, single thread</sub>

**Quick recommendation**: Use [`LosslessTransformUtilsSizeEstimation`] for real-time packing (zstd level < 3), else [`ZStandardSizeEstimation`] with level 1.

Higher compression ratios are possible with [`ZStandardSizeEstimation`], but the gains beyond level 1 are minimal; 
they should be used for final publishing only alongside other brute force options.

[`LosslessTransformUtilsSizeEstimation`]: https://docs.rs/dxt-lossless-transform-ltu/latest/dxt_lossless_transform_ltu/struct.LosslessTransformUtilsSizeEstimation.html
[`ZStandardSizeEstimation`]: https://docs.rs/dxt-lossless-transform-zstd/latest/dxt_lossless_transform_zstd/struct.ZStandardSizeEstimation.html

## API Reference

### Core Transform Functions

- [`Bc1AutoTransformBuilder::transform`] - Transform BC1 data with automatically optimized settings
- [`Bc1ManualTransformBuilder::transform`] - Transform BC1 data using configured settings
- [`Bc1ManualTransformBuilder::untransform`] - Untransform BC1 data using configured settings

### Automatic Optimization

- [`Bc1AutoTransformBuilder`] - Builder pattern for automatic optimization settings
  - `new(estimator)` - Create a new automatic transform builder with the provided estimator
  - `use_all_decorrelation_modes(bool)` - Configure thoroughness vs speed tradeoff
  - `transform(input, output)` - Transform data with optimal settings and return a configured manual builder for untransformation

### Manual Configuration

- [`Bc1ManualTransformBuilder`] - Builder pattern for manual transform configuration
  - `new()` - Create a new manual transform builder  
  - `decorrelation_mode(YCoCgVariant)` - Set the decorrelation mode
  - `split_colour_endpoints(bool)` - Set whether to split color endpoints
  - `transform(input, output)` - Transform BC1 data using configured settings
  - `untransform(input, output)` - Untransform BC1 data using configured settings

### Core Types

- [`YCoCgVariant`] - Color decorrelation mode variants

### Advanced API (Core Crate)

For advanced users who need direct access to transform functions:

- **Core Crate Functions**: For ABI-unstable but safe slice-based functions, see `dxt-lossless-transform-bc1::transform::safe`
- **Maximum Performance**: For unsafe pointer-based functions, see `dxt-lossless-transform-bc1::transform`

**⚠️ Note**: Functions in the core crate may have breaking changes between versions. For production use, prefer the builder patterns above.

### Error Handling

All functions return [`Result`] types with detailed error information:

- [`Bc1Error::InvalidLength`] - Input data not divisible by 8 bytes
- [`Bc1Error::OutputBufferTooSmall`] - Output buffer insufficient for operation
- [`Bc1Error::AllocationFailed`] - Memory allocation error
- [`Bc1Error::SizeEstimationFailed`] - Size estimation error during optimization (preserves the actual estimator error)

## Usage Examples

### Manual Transform Configuration

When you know the optimal settings for your data, you can configure transforms manually. The builder can be reused as needed:

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use dxt_lossless_transform_bc1_api::{
    Bc1ManualTransformBuilder, YCoCgVariant
};

let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
let mut transformed = vec![0u8; bc1_data.len()];
let mut restored = vec![0u8; bc1_data.len()];

// Create reusable builder with specific settings
let builder = Bc1ManualTransformBuilder::new()
    .decorrelation_mode(YCoCgVariant::Variant1)
    .split_colour_endpoints(true);

// Transform the data
builder.transform(&bc1_data, &mut transformed)?;

// Later, after decompression, restore original data
builder.untransform(&transformed, &mut restored)?;

// Restored data should match original
assert_eq!(bc1_data, restored);
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
# use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;
# use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
# let data = vec![0u8; 8 * 100];
# let mut output_fast = vec![0u8; 8 * 100];
# let mut output_thorough = vec![0u8; 8 * 100];

let estimator = LosslessTransformUtilsSizeEstimation::new();

// Fast mode: ~2x faster, tests common configurations
let fast_untransform_builder = Bc1AutoTransformBuilder::new(estimator)
    .use_all_decorrelation_modes(false)
    .transform(&data, &mut output_fast)?;

// Thorough mode: tests all decorrelation modes for best compression
// (typical gains are <0.1%; consider using estimator level closer to final compression level instead)
let estimator2 = LosslessTransformUtilsSizeEstimation::new();
let thorough_untransform_builder = Bc1AutoTransformBuilder::new(estimator2)
    .use_all_decorrelation_modes(true)
    .transform(&data, &mut output_thorough)?;
# Ok(())
# }
```

## Integration Examples

### Error Handling Patterns

```rust
# fn main() {
use dxt_lossless_transform_bc1_api::{Bc1AutoTransformBuilder, Bc1Error};
# use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
# let input = vec![0u8; 8 * 10];
# let mut output = vec![0u8; 8 * 10];

let estimator = LosslessTransformUtilsSizeEstimation::new();

match Bc1AutoTransformBuilder::new(estimator)
    .use_all_decorrelation_modes(false)
    .transform(&input, &mut output) {
    Ok(untransform_builder) => println!("Transform successful"),
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

## CLI Tool

You can use the `dxt-lossless-transform-cli` tool to perform transforms on DDS files:

```bash
cargo install dxt-lossless-transform-cli

# Transform textures in a directory
dxt-lossless-transform-cli transform --input textures --output textures-transformed
```

## Expected Performance Results

Dataset is a bunch of textures from most popular Skyrim mods on [Nexus Mods](https://www.nexusmods.com/skyrimspecialedition/mods).

```bash
.
├── 202x-architecture-10.01
├── 202x-landscape-10.01
├── 202x-other-10.01
├── 3dnpc-loose-latest
├── book-of-silence-se-1.9.1
├── glorious-doors-of-skyrim-1.1
├── majestic-mountains-402
├── noble-skyrim-2k
├── ruins-clutter-improved-se-3.2
├── skyland-aio-4.32
└── skyrim-flora-overhaul-with-grass
```

2130 files, of size 8.5GiB.

### Compression Stats

| Metric                    | LTU Estimator       | Zstd Level 1 Estimator |
| ------------------------- | ------------------- | ---------------------- |
| **Files analyzed**        | 2,130               | 2,130                  |
| **Original uncompressed** | 8.489 GiB           | 8.489 GiB              |
| **Original compressed**   | 5.695 GiB           | 5.695 GiB              |
| **Default compressed**    | 4.857 GiB           | 4.857 GiB              |
| **API recommended**       | 4.856 GiB           | 4.852 GiB              |
| **Best possible**         | 4.849 GiB           | 4.849 GiB              |
| **Space saved (default)** | 857.474 MiB (14.7%) | 857.474 MiB (14.7%)    |
| **Space saved (API)**     | 858.490 MiB (14.7%) | 862.578 MiB (14.8%)    |
| **Space saved (best)**    | 866.048 MiB (14.9%) | 866.048 MiB (14.9%)    |

**Test Commands:**

- **LTU Estimator:** `cargo run --bin dxt-lossless-transform-cli --release --features "debug-bc1 nightly" debug-bc1 compression-stats "/home/sewer/Temp/texture-stuff/bc1-raw" --compression-algorithm zstd --compression-level 16 --estimate-compression-algorithm ltu`
- **Zstd Level 1 Estimator:** `cargo run --bin dxt-lossless-transform-cli --release --features "debug-bc1 nightly" debug-bc1 compression-stats "/home/sewer/Temp/texture-stuff/bc1-raw" --compression-algorithm zstd --compression-level 16 --estimate-compression-algorithm zstd --estimate-compression-level 1 --estimate-compression-algorithm zstd`

### Load Times

```bash
cargo run --bin dxt-lossless-transform-cli --release --features "debug-bc1" debug-bc1 benchmark "/home/sewer/Temp/texture-stuff/bc1-raw" --compression-level 16 --estimate-compression-level 1 --warmup-iterations 1 --iterations 3 --estimate-compression-algorithm ltu
```

```text
📊 Overall Benchmark Statistics:
═══════════════════════════════════════════════════════════════
Files benchmarked: 2130
Total data processed: 8692.896 MiB

📈 NoSplit/YCoCg1:
  Decompress: avg 1.93 GiB/s, total 4389.38 ms
  Untransform: avg 30.17 GiB/s, total 281.42 ms
  Combined: avg 1.82 GiB/s, total 4670.80 ms

📈 NoSplit/None:
  Decompress: avg 1.90 GiB/s, total 4456.52 ms
  Untransform: avg 34.86 GiB/s, total 243.53 ms
  Combined: avg 1.81 GiB/s, total 4700.06 ms

📈 Split/None:
  Decompress: avg 1.91 GiB/s, total 4452.17 ms
  Untransform: avg 33.37 GiB/s, total 254.37 ms
  Combined: avg 1.80 GiB/s, total 4706.55 ms

📈 API Recommended:
  Decompress: avg 1.92 GiB/s, total 4412.05 ms
  Untransform: avg 28.04 GiB/s, total 302.70 ms
  Combined: avg 1.80 GiB/s, total 4714.75 ms

📈 Split/YCoCg1:
  Decompress: avg 1.93 GiB/s, total 4409.77 ms
  Untransform: avg 27.64 GiB/s, total 307.18 ms
  Combined: avg 1.80 GiB/s, total 4716.95 ms

📈 Untransformed:
  Decompress: avg 1.53 GiB/s, total 5559.60 ms
  Untransform: avg 0 B/s, total 0.00 ms
  Combined: avg 1.53 GiB/s, total 5559.60 ms

═══════════════════════════════════════════════════════════════
```

In general, over large textures, `API Recommended` wins.

Over some smaller textures, the less complex `NoSplit/YCoCg1` setting may sometimes be faster; due to less
complexity while keeping similar compression ratios.

## Features

- `std` (default): Enables standard library support
- `c-exports`: Enables C API exports for FFI compatibility

## License

Licensed under [GPL v3 (with Reloaded FAQ)](https://github.com/Sewer56/dxt-lossless-transform/blob/main/LICENSE).

For more information about licensing, see the [Reloaded project licensing guide][reloaded-license].

## C API (FFI)

For C/C++/FFI usage, see the `C API documentation` in the `c_api` module.

## Development

For information on how to work with this codebase, see [README-DEV.MD][readme-dev].

[reloaded-license]: https://reloaded-project.github.io/Reloaded.MkDocsMaterial.Themes.R2/Pages/license.html
[readme-dev]: https://github.com/Sewer56/dxt-lossless-transform/blob/main/README-DEV.MD