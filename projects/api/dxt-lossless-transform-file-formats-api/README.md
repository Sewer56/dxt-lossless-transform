# dxt-lossless-transform-file-formats-api

File format aware API for DXT lossless transform operations.

This crate provides high-level APIs that automatically handle file format detection, header embedding,
and restoration during texture transform operations.

## Features

- **Automatic format detection**: Supports DDS files (with more formats coming)
- **Seamless header embedding**: Transform details are stored in file headers
- **Type-safe transform bundles**: Configure different settings for each BCx format
- **Memory-mapped file support**: Efficient file I/O operations
- **Slice-based API**: Work with data in memory without file I/O

## Usage

### Basic Transform with Default Settings

```rust
use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_bundle};
use dxt_lossless_transform_dds::DdsHandler;

// Create a bundle with default settings for supported formats
let bundle = TransformBundle::default_all();

// Transform a DDS file in memory
let mut output = vec![0u8; input.len()];
transform_slice_bundle(&DdsHandler, &input, &mut output, &bundle)?;
```

### Custom BC1 Settings

```rust
use dxt_lossless_transform_file_formats_api::TransformBundle;
use dxt_lossless_transform_bc1_api::Bc1ManualTransformBuilder;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

let bundle = TransformBundle::new()
    .with_bc1_manual(
        Bc1ManualTransformBuilder::new()
            .with_split_colour_endpoints(true)
            .with_decorrelation_mode(YCoCgVariant::Variant2)
    );
    // Only BC1 is currently supported with configurable options
```

### File I/O Operations

With the `file-io` feature enabled:

```rust
use dxt_lossless_transform_file_formats_api::file_io::{transform_file_bundle, untransform_file_with};
use dxt_lossless_transform_dds::DdsHandler;
use std::path::Path;

// Transform file to file
let bundle = TransformBundle::default_all();
transform_file_bundle(&DdsHandler, Path::new("input.dds"), Path::new("output.dds"), &bundle)?;

// Untransform (settings are extracted from embedded header)
untransform_file_with(&DdsHandler, Path::new("output.dds"), Path::new("restored.dds"))?;
```

### Untransforming Files

```rust
use dxt_lossless_transform_file_formats_api::untransform_slice_with;
use dxt_lossless_transform_dds::DdsHandler;

// Untransform data in memory
let mut output = vec![0u8; input.len()];
untransform_slice_with(&DdsHandler, &input, &mut output)?;
```

## How It Works

1. **Transform**: The file format handler detects the BCx format, applies the appropriate transform from the bundle, and embeds the transform details in the file header (replacing the original magic bytes).

2. **Untransform**: The handler extracts the transform details from the embedded header, restores the original file format magic bytes, and applies the reverse transform using the embedded settings.

## Supported Formats

- **File Formats**: DDS (DirectDraw Surface)
- **Compression Formats**: BC1 (manual mode with configurable options)

**Note**: BC2, BC3, and BC7 support is planned but not yet available with configurable transform options.

## Transform Bundle Configuration

The `TransformBundle` allows you to configure transform settings:

- **BC1**: Full configuration support via `Bc1ManualTransformBuilder`
  - Split color endpoints
  - Color decorrelation modes
  - Other transform optimizations

Create bundles using:
- `TransformBundle::new()` - Empty bundle, configure as needed
- `TransformBundle::default_all()` - Default settings for supported formats

If a format is encountered that isn't configured in the bundle, the transform will fail with an error.

## Features

- `std` (default): Standard library support
- `file-io`: Enables memory-mapped file I/O functions using `lightweight-mmap`

## Error Handling

The API uses `FileFormatResult<T>` for error handling, which can indicate:

- Invalid file data
- Unsupported formats
- Missing builders for detected formats
- I/O errors (when using file operations)

## DDS Handler Details

The `DdsHandler` from the `dxt-lossless-transform-dds` crate:

- Detects BC1/BC2/BC3/BC7 formats within DDS files
- Embeds transform metadata in the 4-byte DDS magic header
- Supports both transform and untransform operations
- Can detect transformed files for untransform operations
- Currently only BC1 files can be transformed with configurable settings 