# dxt-lossless-transform

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform.svg)](https://crates.io/crates/dxt-lossless-transform)
[![Docs.rs](https://docs.rs/dxt-lossless-transform/badge.svg)](https://docs.rs/dxt-lossless-transform)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

[Note: Currently in development. Only BC1 done. BC2/3/7 coming soon.]

## About

This crates provide fast lossless transforms for DDS files that improve their compression ratio
once compressed by external compressors.

## Example

Applying filter on every BC1 DDS file of `-Skyrim 202X 10.0.1 - Landscape PART 2-2347-10-0-1-1710489616.rar`.

### Before

```ignore
Tool       Size            Ratio 
---------------------------------
original   2.33 GiB        100.00%
zstd 22    1.48 GiB        63.75%
kanzi 7    1.31 GiB        56.14%
bzip3 16m  1.30 GiB        56.02%
7z         1.34 GiB        57.37%
```

### After

```ignore
Tool       Size            Ratio     
----------------------------------
original   2.33 GiB        100.00%
zstd 22    1.35 GiB        57.91% 
kanzi 7    1.20 GiB        51.50%
bzip3 16m  1.20 GiB        51.41%
7z         1.28 GiB        55.02%
```

BC2, BC3, BC7 are currently in development.
Expect similar savings for BC2, BC3. BC7 is likely to be less effective.

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

- SSE2 (x86-64, 64-bit only, Rust + LLVM, no SIMD): 22.234 GiB/s `u64 sse2`
- Compiler (x86-64, 64-bit only, Rust + LLVM, no SIMD): 19.031 GiB/s `portable64 no-unroll`
- Compiler (x86-64, 32 & 64-bit, Rust + LLVM, no SIMD): 15.851 GiB/s `portable32 v2`
- Compiler (x86-64, Rust + LLVM, no SIMD): 15.405 GiB/s `portable32 no-unroll`

BC3 is tricky, due to unusual memory layout that is not register size or SIMD friendly.
AVX2 has the capability to make it somewhat 'work', but SSE is not feature rich enough.

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

```ignore
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