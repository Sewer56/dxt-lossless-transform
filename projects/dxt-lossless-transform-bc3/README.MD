# dxt-lossless-transform-bc3

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform-bc3.svg)](https://crates.io/crates/dxt-lossless-transform-bc3)
[![Docs.rs](https://docs.rs/dxt-lossless-transform-bc3/badge.svg)](https://docs.rs/dxt-lossless-transform-bc3)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

## About

This crate provides a fast, visually lossless transform for the BC3 block format.

## Reference Numbers

Dataset is the following

```text
.
├── 202x-architecture-10.01
├── 202x-landscape-10.01
├── 3dnpc-loose-latest
├── book-of-silence-se-1.9.1
├── embers-xd-3.0.8
├── glorious-doors-of-skyrim-1.1
├── majestic-mountains-402
├── noble-skyrim-2k
├── ruins-clutter-improved-se-3.2
├── skyland-aio-4.32
└── skyrim-flora-overhaul-with-grass
```

A bunch of Skyrim mods, from Nexus, obtained by sorting by most downloaded under the 'texture'
category. All non .DDS files were removed.

Original data contained 1138 files, totalling 5.00GiB.

### Fast Mode (Split Transform Only)

#### Before

```text
Tool       Size            Ratio 
---------------------------------
zlib       3.56 GiB        71.09%
zstd 22    3.28 GiB        65.51%
bzip3 16m  2.89 GiB        57.73%
kanzi 7    2.89 GiB        57.88%
7z         2.98 GiB        59.64%
```

#### After

```text
Tool       Size            Ratio 
---------------------------------
zlib       3.12 GiB        62.37%
zstd 22    2.95 GiB        59.06%
bzip3 16m  2.75 GiB        54.92%
kanzi 7    2.75 GiB        55.07%
7z         2.83 GiB        56.59%
```

#### Performance

On a 5900X, with DDR4 3200, CL16 RAM the BC3 transform runs at approx:

Transform BC3 (`bc3_transform`):

- Mixed AVX2+Scalar (Intrinsics, x86-64): 17.343 GiB/s `avx2`
- Mixed AVX2+Scalar (Intrinsics, i686): 16.712 GiB/s `avx2`
- Compiler (x86-64, Rust + LLVM, no SIMD): 11.576 GiB/s `portable32 no-unroll`
- Compiler (i686, Rust + LLVM, no SIMD): 10.510 GiB/s `portable32 no-unroll`

Detransform BC3 (`bc3_detransform`):

- SSE2 (x86-64, 64-bit only): 22.234 GiB/s `u64 sse2`
- Compiler (x86-64, 64-bit only, Rust + LLVM, no SIMD): 19.031 GiB/s `portable64 no-unroll`
- Compiler (x86-64, 32 & 64-bit, Rust + LLVM, no SIMD): 15.851 GiB/s `portable32 v2`
- Compiler (x86-64, Rust + LLVM, no SIMD): 15.405 GiB/s `portable32 no-unroll`
- SSE2 (x86-64, 32-bit): 13.668 GiB/s `u32 sse2` (disabled by default)

BC3 is tricky, because it has 2 and 6 byte fields, making them SIMD unfriendly.
AVX2 has 'gather' for loading, but not 'scatter' for storing. 

Portable 64-bit implementation did not yield performance benefits for transform/untransform.

You can run the benchmarks like this (example):

```bash,ignore
cargo bench -p dxt-lossless-transform-bc3 --bench unsplit_blocks -- "u64 sse2"
```

Measured on Linux with `performance` governor. 8MiB file (see: `projects/dxt-lossless-transform-bc3/benches`).
These benches are sensitive to external load and CPU temperature, therefore we instead measured steady
state performance, i.e. 60 seconds warmup (heat up the parts) and 60 seconds measurement.

In general, the output of LLVM is on par with manually written routines, however the manually
written routines are smaller in code size; more friendly with cache, etc.

## Usage

This crate is not yet released.

## Testing (CLI)

You can use the `dxt-lossless-transform-cli` tool to perform this transform on DDS files.

```text
./dxt-lossless-transform-cli transform --input textures --output textures-transformed
```

You can install it via `cargo install dxt-lossless-transform-cli`, or build it from source.

## Development

For information on how to work with this codebase, see [README-DEV.MD][readme-dev].

## License

Licensed under [GPL v3 (with Reloaded FAQ)](./LICENSE).  
[Learn more about Reloaded's general choice of licensing for projects.][reloaded-license].  

[codecov]: https://about.codecov.io/
[crates-io-key]: https://crates.io/settings/tokens
[nuget-key]: https://www.nuget.org/account/apikeys
[docs]: https://dxt-lossless-transform.github.io/dxt-lossless-transform
[reloaded-license]: https://reloaded-project.github.io/Reloaded.MkDocsMaterial.Themes.R2/Pages/license.html
[readme-dev]: https://github.com/Sewer56/dxt-lossless-transform/blob/main/README-DEV.MD