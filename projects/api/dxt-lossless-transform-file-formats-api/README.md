# dxt-lossless-transform-file-formats-api

File format aware API for DXT lossless transform operations.

This crate provides high-level APIs that automatically handle file format detection, header embedding, and restoration during DXT transform operations.

## Features

- **Automatic format detection**: Supports DDS files (with more formats coming)
- **Seamless header embedding**: Transform details are stored in file headers
- **Type-safe transform bundles**: Configure different settings for each BCx format
- **Memory-mapped file support**: Efficient file I/O operations
- **Slice-based API**: Work with data in memory without file I/O

## Usage

### Basic Transform with Auto Settings

```rust
use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_bundle};
use dxt_lossless_transform_dds::DdsHandler;

// Create a bundle with auto settings for all formats
let bundle = TransformBundle::auto_all();

// Transform a DDS file in memory
let mut output = vec![0u8; input.len()];
transform_slice_bundle(&DdsHandler, &input, &mut output, &bundle)?;
```

### Custom Settings Per Format

```rust
use dxt_lossless_transform_file_formats_api::TransformBundle;
use dxt_lossless_transform_bc1_api::Bc1ManualTransformBuilder;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

let bundle = TransformBundle::new()
    .with_bc1_manual(
        Bc1ManualTransformBuilder::new()
            .with_split_colour_endpoints(true)
            .with_decorrelation_mode(YCoCgVariant::Variant2)
    )
    .with_bc2(Bc2AutoTransformBuilder::new())
    // BC3 files will be skipped (no builder provided)
    ;
```

### File I/O Operations

With the `file-io` feature enabled:

```rust
use dxt_lossless_transform_file_formats_api::file_io::{transform_file, untransform_file};
use dxt_lossless_transform_dds::DdsHandler;

// Transform file to file
let bundle = TransformBundle::auto_all();
transform_file(&DdsHandler, "input.dds", "output.dds", &bundle)?;

// Untransform (settings are extracted from header)
untransform_file(&DdsHandler, "output.dds", "restored.dds")?;
```

## How It Works

1. **Transform**: The file format handler detects the BCx format, applies the appropriate transform from the bundle, and embeds the transform details in the file header.

2. **Untransform**: The handler extracts the transform details from the header, restores the original file format magic, and applies the reverse transform.

## Supported Formats

- **File Formats**: DDS (DirectDraw Surface)
- **Compression Formats**: BC1, BC2, BC3 (BC7 placeholder for future)

## Features

- `std` (default): Standard library support
- `file-io`: Enables memory-mapped file I/O functions 