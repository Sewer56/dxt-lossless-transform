# dxt-lossless-transform-bc1

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform-bc1.svg)](https://crates.io/crates/dxt-lossless-transform-bc1)
[![Docs.rs](https://docs.rs/dxt-lossless-transform-bc1/badge.svg)](https://docs.rs/dxt-lossless-transform-bc1)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

## About

This crate provides a fast, visually lossless transform for the BC1 block format.

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

The original data set contained 1943 files, totalling 6.84GiB.

### Fast Mode (Split Transform Only)

#### Before

```text
Tool       Size            Ratio 
---------------------------------
zlib       4.77 GiB        69.77%
zstd 22    4.23 GiB        61.91%
bzip3 16m  3.83 GiB        56.00%
kanzi 7    3.85 GiB        56.32%
7z         3.83 GiB        56.05%
```

#### After

```text
Tool       Size            Ratio 
---------------------------------
zlib       4.29 GiB        62.70%
zstd 22    3.89 GiB        56.95%
bzip3 16m  3.51 GiB        51.39%
kanzi 7    3.53 GiB        51.67%
7z         3.69 GiB        54.03%
```

#### Performance

On a 5900X, with DDR4 3200, CL16 RAM the BC1 transform runs at approx:

Transform BC1 (`bc1_split_blocks`):

- AVX2 (Assembly): 58.329 GiB/s `avx2 shuffle_permute unroll 2`
- SSE2 (Assembly): 56.302 GiB/s `sse2 shufps unroll 4`
- Compiler v3 (AVX2, Rust + LLVM on `x86-64-v3`): 58.163 GiB/s `portable32 no-unroll`

Untransform BC1 (`bc1_unsplit_blocks`):

- AVX2 (Assembly): 54.211 GiB/s `avx2 permd unroll 2`
- SSE2 (Assembly): 47.392 GiB/s `sse2 unpck unroll 2`
- Compiler v3 (AVX2, Rust + LLVM on `x86-64-v3`): 53.013 GiB/s `portable32 no-unroll`

You can run the benchmarks like this (example):

```bash,ignore
cargo bench -p dxt-lossless-transform-bc1 --bench unsplit_blocks -- "avx2 shuffle_permute"
```

Measured on Linux with `performance` governor. 8MiB file (see: `projects/dxt-lossless-transform-bc1/benches`).
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
[reloaded-license]: https://reloaded-project.github.io/Reloaded.MkDocsMaterial.Themes.R2/Pages/license/
[readme-dev]: https://github.com/Sewer56/dxt-lossless-transform/blob/main/README-DEV.MD