# dxt-lossless-transform-file-formats-api

This crate provides the high level API for adding support to texture file formats such as DDS to the
`dxt-lossless-transform` project.

## Cheat Sheet

If you just want to transform texture files, use the existing handlers like `DdsHandler` from `dxt-lossless-transform-dds` crate.

### Basic File Transform

```rust
use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_with_bundle};
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;
use dxt_lossless_transform_dds::DdsHandler;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
# let input = vec![0u8; 1024];
// Transform a DDS file in memory
let estimator = LosslessTransformUtilsSizeEstimation::new();
let bundle = TransformBundle::new()
    .with_bc1_auto(Bc1AutoTransformBuilder::new(estimator));
let mut output = vec![0u8; input.len()];
transform_slice_with_bundle(&DdsHandler, &input, &mut output, &bundle)?;
# Ok(())
# }
```

### BC1 Automatic Optimization

```rust
use dxt_lossless_transform_file_formats_api::TransformBundle;
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;

# fn example() {
let estimator = LosslessTransformUtilsSizeEstimation::new();
let bundle = TransformBundle::new()
    .with_bc1_auto(
        Bc1AutoTransformBuilder::new(estimator)
    );
# }
```

### Multiple Handlers (Unknown File Types)

When you have unknown file types, you can use the methods with prefix `multiple_handlers`;
this will go over every file format handler until one accepts the file.

```rust
use dxt_lossless_transform_file_formats_api::{transform_slice_with_multiple_handlers, TransformBundle};
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;
use dxt_lossless_transform_dds::DdsHandler;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
# let input = vec![0u8; 1024];
// Try handlers in order until one accepts the file
let handlers = [DdsHandler];
let estimator = LosslessTransformUtilsSizeEstimation::new();
let bundle = TransformBundle::new()
    .with_bc1_auto(Bc1AutoTransformBuilder::new(estimator));
let mut output = vec![0u8; input.len()];
transform_slice_with_multiple_handlers(handlers, &input, &mut output, &bundle)?;
# Ok(())
# }
```

### Untransforming Files

```rust
use dxt_lossless_transform_file_formats_api::{untransform_slice, untransform_slice_with_multiple_handlers};
use dxt_lossless_transform_dds::DdsHandler;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
# let input = vec![0u8; 1024];
// When you know the format
let mut output = vec![0u8; input.len()];
untransform_slice(&DdsHandler, &input, &mut output)?;

// When you don't know the format
let handlers = [DdsHandler];
let mut output = vec![0u8; input.len()];
untransform_slice_with_multiple_handlers(handlers, &input, &mut output)?;
# Ok(())
# }
```

### File I/O Operations

With the `file-io` feature:

```rust
use dxt_lossless_transform_file_formats_api::file_io::{
    transform_file_with_handler, untransform_file_with_handler
};
use dxt_lossless_transform_file_formats_api::TransformBundle;
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;
use dxt_lossless_transform_dds::DdsHandler;
use std::path::Path;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let estimator = LosslessTransformUtilsSizeEstimation::new();
let bundle = TransformBundle::new()
    .with_bc1_auto(Bc1AutoTransformBuilder::new(estimator));
transform_file_with_handler(&DdsHandler, Path::new("input.dds"), Path::new("output.dds"), &bundle)?;
untransform_file_with_handler(&DdsHandler, Path::new("output.dds"), Path::new("restored.dds"))?;
# Ok(())
# }
```

### Manual Transform Configuration

Not recommended, unless you're transforming in real-time with very low CPU overhead requirements.

(Or if you know optimal settings ahead of time.)

```rust
use dxt_lossless_transform_file_formats_api::TransformBundle;
use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
use dxt_lossless_transform_bc1_api::Bc1ManualTransformBuilder;

# fn example() {
let bundle = TransformBundle::<LosslessTransformUtilsSizeEstimation>::new()
    .with_bc1_manual(Bc1ManualTransformBuilder::new()); // Set settings by hand.
# }
```

## Implementing Custom File Format Handlers

To add support for new texture file formats, implement the handler traits.

Look at `DdsHandler` from `dxt-lossless-transform-dds` crate for inspiration.

### Basic Handler

All handlers must implement [`FileFormatHandler`]:

```rust
use dxt_lossless_transform_file_formats_api::{FileFormatHandler, TransformBundle, TransformResult};
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

struct MyFormatHandler;

impl FileFormatHandler for MyFormatHandler {
    fn transform_bundle<T>(
        &self, 
        input: &[u8], 
        output: &mut [u8], 
        bundle: &TransformBundle<T>
    ) -> TransformResult<()> 
    where 
        T: SizeEstimationOperations,
        T::Error: core::fmt::Debug,
    {
        // 0. Validate input & output buffer are large enough.
        // 1. Parse your file format header
        // 2. Detect BCx format (BC1, BC2, BC3, BC6H, BC7)
        // 3. Find texture data portion
        // 4. Call bundle.dispatch_transform() to process the texture data
        // 5. Embed Transformheader in original file header (usually by overwriting the 'MAGIC Header')
        todo!()
    }

    fn untransform(&self, input: &[u8], output: &mut [u8]) -> TransformResult<()> {
        // 0. Validate input & output buffer are large enough.
        // 1. Read the TransformHeader from the file header
        // 2. Parse your file format header with embedded transform data
        // 3. Extract transform details and texture data
        // 4. Call dispatch_untransform() to restore the texture data
        // 5. Restore original file header in output
        todo!()
    }
}
```

### Transform Detection

For automatic format detection during transform, implement [`FileFormatDetection`]:

```rust
use dxt_lossless_transform_file_formats_api::FileFormatDetection;

# struct MyFormatHandler;
# use dxt_lossless_transform_file_formats_api::{FileFormatHandler, TransformBundle, TransformResult};
# use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
# impl FileFormatHandler for MyFormatHandler {
#     fn transform_bundle<T>(&self, input: &[u8], output: &mut [u8], bundle: &TransformBundle<T>) -> TransformResult<()>
#     where T: SizeEstimationOperations, T::Error: core::fmt::Debug, { todo!() }
#     fn untransform(&self, input: &[u8], output: &mut [u8]) -> TransformResult<()> { todo!() }
# }
impl FileFormatDetection for MyFormatHandler {
    fn can_handle(&self, input: &[u8], file_extension: Option<&str>) -> bool {
        // Check file extension first (faster)
        if let Some(ext) = file_extension {
            if ext != "myformat" {
                return false;
            }
        }
        
        // Then check file header magic bytes
        input.len() >= 4 && &input[0..4] == b"MFMT" // "MyFormat"
    }
}
```

### Untransform Detection

For automatic format detection during untransform, implement [`FileFormatUntransformDetection`]:

```rust
use dxt_lossless_transform_file_formats_api::FileFormatUntransformDetection;

# struct MyFormatHandler;
# use dxt_lossless_transform_file_formats_api::{FileFormatHandler, TransformBundle, TransformResult};
# use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
# impl FileFormatHandler for MyFormatHandler {
#     fn transform_bundle<T>(&self, input: &[u8], output: &mut [u8], bundle: &TransformBundle<T>) -> TransformResult<()>
#     where T: SizeEstimationOperations, T::Error: core::fmt::Debug, { todo!() }
#     fn untransform(&self, input: &[u8], output: &mut [u8]) -> TransformResult<()> { todo!() }
# }
impl FileFormatUntransformDetection for MyFormatHandler {
    fn can_handle_untransform(&self, input: &[u8], file_extension: Option<&str>) -> bool {
        // Insert logic here that validates the file.
        // Bear in mind, that the transform details are in the place they were placed 
        // during `transform_bundle` call. So a part of the header would be overwritten.
        true // placeholder
    }
}
```

### Header Sizes

The size of the `TransformHeader` is defined by the constant:

```rust
use dxt_lossless_transform_file_formats_api::embed::TRANSFORM_HEADER_SIZE; // 4 bytes
```

However, transforms for some formats require additional space beyond the 4-byte header:

- **BC6H**: Requires an additional 80 bytes (`BC6H_ADDITIONAL_SPACE`)
- **BC7**: Requires an additional 48 bytes (`BC7_ADDITIONAL_SPACE`)

```rust
use dxt_lossless_transform_file_formats_api::embed::{
    TRANSFORM_HEADER_SIZE,       // 4 bytes
    BC6H_ADDITIONAL_SPACE,       // 80 bytes
    BC7_ADDITIONAL_SPACE,        // 48 bytes
};
```
Where to store this additional data is up to the handler implementation.
Options include:

- Start of file (before anything else)
- End of file (after everything)
- After main header (e.g. after DDS header)
    - This is where the DDS implementation places the texture data 😉

### Alignment Recommendation

It's recommended to pad the header + additional space so that the texture data starts at a
64-byte aligned offset for optimal performance.

## API Reference

### Transform Functions

- [`transform_slice_with_bundle`] - Transform with specific handler
- [`transform_slice_with_multiple_handlers`] - Try multiple handlers
- [`untransform_slice`] - Untransform with specific handler  
- [`untransform_slice_with_multiple_handlers`] - Try multiple handlers

### File I/O Functions (with `file-io` feature)

- [`file_io::transform_file_with_handler`] - Transform file with input and output file path
- [`file_io::transform_file_with_multiple_handlers`] - Try multiple handlers
- [`file_io::untransform_file_with_handler`] - Untransform file with input and output file path
- [`file_io::untransform_file_with_multiple_handlers`] - Try multiple handlers

### Handler Traits

- [`FileFormatHandler`] - Basic transform/untransform support
- [`FileFormatDetection`] - Transform-time format detection
- [`FileFormatUntransformDetection`] - Untransform-time format detection

The following low level functions are provided to aid handler implementation:

- [`dispatch_transform`] - Transform texture data only
- [`dispatch_untransform`] - Untransform texture data only

### Bundle Configuration

- [`TransformBundle::new()`] - Empty bundle
- [`TransformBundle::default_all()`] - Default settings for supported formats
- [`TransformBundle::with_bc1_manual()`] - Add manual BC1 settings
- [`TransformBundle::with_bc1_auto()`] - Add automatic BC1 optimization

## Error Types

- [`TransformResult<T>`] - Main result type
- [`TransformError`] - Transform operation errors
- [`FormatHandlerError`] - File format handler errors

Common errors:

- [`TransformError::NoSupportedHandler`] - No handler accepted the file
- [`FormatHandlerError::NoBuilderForFormat`] - Bundle missing required format
- [`TransformError::InvalidDataAlignment`] - Texture data size invalid

## Supported Formats

- **BC1**: Full support (manual and automatic optimization)
- **BC2, BC3, BC6H, BC7**: Planned

## Features

- `std` (default): Standard library support
- `file-io`: File I/O operations with memory mapping 