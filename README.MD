# dxt-lossless-transform

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform.svg)](https://crates.io/crates/dxt-lossless-transform)
[![Docs.rs](https://docs.rs/dxt-lossless-transform/badge.svg)](https://docs.rs/dxt-lossless-transform)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

[Note: Currently in development. Only BC1 done. BC2/3/7 coming soon.]

## About

This crates provide fast lossless transforms for DDS files that improve their compression ratio
once compressed by external compressors.

## Reference Numbers

Dataset is the following:

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

### BC1

Original data contained 1943 files, totalling 6.84GiB.

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

BC2, BC3, BC7 are currently in development.
Expect similar savings for BC2, BC3. BC7 is likely to be less effective.

### BC2

Original data contained 209 files, totalling 750.58MiB.

#### Before

```text
Tool       Size            Ratio 
---------------------------------
zlib       232.34 MiB      30.95%
zstd 22    210.06 MiB      27.99%
bzip3 16m  178.95 MiB      23.84%
kanzi 7    179.77 MiB      23.95%
7z         189.47 MiB      25.24%
```

#### After

```text
Tool       Size            Ratio 
---------------------------------
zlib       199.34 MiB      26.56%
zstd 22    186.78 MiB      24.89%
bzip3 16m  167.29 MiB      22.29%
kanzi 7    167.71 MiB      22.34%
7z         177.30 MiB      23.62%
```

### BC3

Original data contained 1138 files, totalling 5.00GiB.

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

Alternatively, you can use the BC2 transform on BC3 data for 3.5-4.5x throughput at a small minimal
compression ratio loss.

```text
Tool       Size            Ratio 
---------------------------------
zlib       3.19 GiB        63.85%
zstd 22    3.01 GiB        60.13%
bzip3 16m  2.77 GiB        55.35%
kanzi 7    2.78 GiB        55.52%
7z         2.83 GiB        56.56%
```

## Performance

On a 5900X, with DDR4 3200, CL16 RAM the BC1 transform runs at approx:

Transform BC1 (`bc1_transform`):

- AVX2 (Assembly): 58.329 GiB/s `avx2 shuffle_permute unroll 2`
- SSE2 (Assembly): 56.302 GiB/s `sse2 shufps unroll 4`
- Compiler v3 (AVX2, Rust + LLVM on `x86-64-v3`): 58.163 GiB/s `portable32 no-unroll`

Untransform BC1 (`bc1_detransform`):

- AVX2 (Assembly): 54.211 GiB/s `avx2 permd unroll 2`
- SSE2 (Assembly): 47.392 GiB/s `sse2 unpck unroll 2`
- Compiler v3 (AVX2, Rust + LLVM on `x86-64-v3`): 53.013 GiB/s `portable32 no-unroll`

Transform BC2 (`bc2_transform`):

- AVX2 (Assembly): 59.946 GiB/s `avx2 shuffle`
- SSE2 (Assembly, x86-64): 55.965 GiB/s `sse2 shuffle v3`
- SSE2 (Assembly, i686): 55.407 GiB/s `sse2 shuffle v2`
- Compiler v3 (AVX2, Rust + LLVM on `x86-64-v3`): 32.408 GiB/s `portable32 no-unroll`

Untransform BC2 (`bc2_detransform`):

- AVX2 (Assembly, x86-64): 57.076 GiB/s `avx2 shuffle`
- AVX2 (Assembly, i686): 56.103 GiB/s `avx2 shuffle`
- SSE2 (Assembly): 49.670 GiB/s `sse2 shuffle`
- Compiler v3 (AVX2, Rust + LLVM on `x86-64-v3`): 23.602 GiB/s `portable32 no-unroll`

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

Benchmark command:

```bash,ignore
# Adjust as needed according to benchmark project and name.
cargo bench -p dxt-lossless-transform --bench bc2_detransform -- "sse2 shuffle"
```

Measured on Linux with `performance` governor. 8MiB file (see: `projects/dxt-lossless-transform/benches`).
These benches are sensitive to external load and CPU temperature, therefore we instead measured steady
state performance, i.e. 60 seconds warmup (heat up the parts) and 60 seconds measurement.

In general, the output of LLVM is on par with manually written routines, however the manually
written routines are smaller in code size; more friendly with cache, etc.

## Usage

This crate is not yet released.

## Testing (CLI)

This project contains a CLI tool that can be used to test the transforms.

```text
./dxt-lossless-transform-cli transform --input textures --output textures-transformed
```

Build like a regular rust project, i.e. `cargo build --release`, and find it at `./target/release/dxt-lossless-transform-cli`. 

## Development

For information on how to work with this codebase, see [README-DEV.MD](README-DEV.MD).

## License

Licensed under [GPL v3 (with Reloaded FAQ)](./LICENSE).  

[Learn more about Reloaded's general choice of licensing for projects.][reloaded-license].  

[codecov]: https://about.codecov.io/
[crates-io-key]: https://crates.io/settings/tokens
[nuget-key]: https://www.nuget.org/account/apikeys
[docs]: https://dxt-lossless-transform.github.io/dxt-lossless-transform
[reloaded-license]: https://reloaded-project.github.io/Reloaded.MkDocsMaterial.Themes.R2/Pages/license/