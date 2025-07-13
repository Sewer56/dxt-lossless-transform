# dxt-lossless-transform-file-formats-debug

Debug and research utilities for dxt-lossless-transform file format handling.

## Overview

This is an **optional** crate that provides debug-only functionality for working with compressed
texture file formats. It contains utilities that are useful for research, analysis, debugging, and
CLI tools, but are **not intended for production use**.

The functionality in this crate is specifically separated from the stable
`dxt-lossless-transform-file-formats-api` to ensure that debug-only code doesn't get included in
production builds.

## Purpose

This crate provides:

- **Block extraction utilities**: Extract raw block data from file formats for analysis
- **Format detection utilities**: Determine the [`TransformFormat`] of texture files without extracting block data

## Usage

This crate is primarily used by:

- The CLI tool for debug commands (`dxt-lossless-transform-cli`)
- Research and development

## Important Notes

- **Not for production**: This crate should never be included in production applications
- **Opt-in implementation**: File format handlers (like `DdsHandler`) must explicitly implement the debug traits
- **Feature-gated**: Most functionality requires explicit feature flags to be enabled

## Example

```rust
use dxt_lossless_transform_file_formats_debug::{
    FileFormatBlockExtraction,
    TransformFormatCheck,
    TransformFormatFilter,
    ExtractedBlocks,
};
use dxt_lossless_transform_file_formats_api::{
    embed::TransformFormat,
    error::TransformResult,
};

// File format handlers can optionally implement debug traits
impl TransformFormatCheck for MyFormatHandler {
    fn get_transform_format(
        &self,
        data: &[u8],
        filter: TransformFormatFilter,
    ) -> TransformResult<Option<TransformFormat>> {
        // Implementation for format detection
        Ok(Some(TransformFormat::Bc1))
    }
}

impl FileFormatBlockExtraction for MyFormatHandler {
    fn extract_blocks<'a>(
        &self,
        data: &'a [u8],
        filter: TransformFormatFilter,
    ) -> TransformResult<Option<ExtractedBlocks<'a>>> {
        // Implementation for extracting raw blocks
        Ok(None)
    }
}
```

## Features

- `std` (default): Enable standard library support
- `file-io` (default): Enable file I/O operations for block extraction 