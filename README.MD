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

Transform BC1:

- AVX2 (Assembly): 8.4053 GiB/s `avx2 shuffle_permute unroll 2`
- SSE2 (Assembly): 8.4042 GiB/s `sse2 shufps unroll 4`
- Compiler v3 (AVX2, Rust + LLVM on `x86-64-v3`): 8.3918 GiB/s `portable32 no-unroll`
- Compiler v3 Unroll-8 (Rust + LLVM on `x86-64-v3`): 1.5411 GiB/s `portable32 unroll-8`

Untransform BC1:

- AVX2 (Assembly): 8.3298 GiB/s `avx2 unpck unroll 2`
- SSE2 (Assembly): 8.2970 GiB/s `sse2 unpck unroll 2`
- Compiler v3 (AVX2, Rust + LLVM on `x86-64-v3`): 8.3116 GiB/s `portable32 no-unroll`
- Compiler v3 Unroll-8 (Rust + LLVM on `x86-64-v3`): 8.01 GiB/s `portable32 unroll-8`

Measured on Linux with `performance` governor. Sensitive to external load and CPU temperature, therefore
benched with 60 second warmup and 60 seconds measurement. 

In general, the output of LLVM is on par with manually written routines, however the manually
written routines are smaller in code size; more friendly with cache, etc.

## Usage

This crate is not yet released.

## Testing (CLI)

This project contains a CLI tool that can be used to test the transforms.

```
./dxt-lossless-transform-cli transform --input textures --output textures-transformed
```

Build like a regular rust project, i.e. `cargo build --release`, and find it at `./target/release/dxt-lossless-transform-cli`. 

## How it Works



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