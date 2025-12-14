# dxt-lossless-transform-common

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform-common.svg)](https://crates.io/crates/dxt-lossless-transform-common)
[![Docs.rs](https://docs.rs/dxt-lossless-transform-common/badge.svg)](https://docs.rs/dxt-lossless-transform-common)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

## About

This crate provides shared components between various implementations of [dxt-lossless-transform].
This contains shared private implementation detail. This crate is not intended to provide a stable API for external use.

## Core Types

- [Color565](./src/color_565.rs): A color type representing a 5-bit red, 6-bit green, and 5-bit blue component.
- [Color8888](./src/color_8888.rs): A color type representing an 8-bit red, 8-bit green, 8-bit blue, and 8-bit alpha component.
- [Decoded4x4Block](./src/decoded_4x4_block.rs): A structure representing a decompressed 4x4 pixel block used in DXT/BC texture compression formats.

## CPU Feature Detection

- [cpu_detect](./src/cpu_detect.rs): Runtime CPU feature detection for SIMD instruction sets (SSE2, AVX2, AVX-512) to enable optimal code paths based on available hardware capabilities.

## Development

For information on how to work with this codebase, see the [Developer Manual](https://reloaded-project.github.io/reloaded-templates-rust/manual/).

## License

Licensed under [GPL v3 (with Reloaded FAQ)](https://github.com/Sewer56/dxt-lossless-transform/blob/main/LICENSE).  
[Learn more about Reloaded's general choice of licensing for projects.][reloaded-license].  

[reloaded-license]: https://reloaded-project.github.io/License/GPLv3/about.html
[dxt-lossless-transform]: https://github.com/Sewer56/dxt-lossless-transform