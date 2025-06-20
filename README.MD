# dxt-lossless-transform

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform.svg)](https://crates.io/crates/dxt-lossless-transform)
[![Docs.rs](https://docs.rs/dxt-lossless-transform/badge.svg)](https://docs.rs/dxt-lossless-transform)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

[Note: Currently in development. We're far from done. BC1-BC3 mostly done, BC7 barely started]

## About

This crates provide fast lossless transforms for DDS files that improve their compression ratio
once compressed by external compressors. These transforms reduce the size of the files, once
compressed, and improve load speed, by making the data faster to decompress.

## Project Layout

> [!TIP]
> The project is split up into the following crates.

Click on a project to navigate to its description.

### High Level API

- [dxt-lossless-transform-bc1-api]: A stable API for [dxt-lossless-transform-bc1].
- [dxt-lossless-transform-bc2-api]: A stable API for [dxt-lossless-transform-bc2].
- [dxt-lossless-transform-bc3-api]: A stable API for [dxt-lossless-transform-bc3].
- [dxt-lossless-transform-bc7-api]: A stable API for [dxt-lossless-transform-bc7].
- [dxt-lossless-transform-api-common]: A stable API for 'add-ons' shared between the other 'api' crates.

### Utility Libraries

- [dxt-lossless-transform-dds]: Adds DDS support for the dxt-lossless-transform crate.

### Estimator Libraries

> [!TIP]
> 
> These libraries help you determine the best transform settings for each algorithm.

Speeds are based off of a single core on a Ryzen 9950X3D CPU, and are approximate values.

- **ZStandard (zstd)**: Uses ZStandard compression to determine best transform settings.
  - Package: [dxt-lossless-transform-zstd]
  - Speed (1 Thread): ~1060MiB/s (level 1)
  - Accuracy: 79.2% (level 1) (vs ZStandard)
  - Use same level as final ZStandard compression level to maximize compression ratio. (but slow!)
  - Otherwise, Level 1 is recommended for 'balanced' profile.
      - Other levels deliver increasingly diminishing returns.

- **Lossless Transform Utils (ltu)**: Fast estimator for generic LZ-based compression algorithms. 
  - Package: [dxt-lossless-transform-zstd]
  - Speed: ~2565MiB/s single-threaded.
  - Accuracy: ~74.4% for high compression (vs ZStandard)
  - Optimized for speed while maintaining reasonable accuracy
  - Suited for real time compression scenarios.

For more information, [refer to my blog post](https://sewer56.dev/blog/2025/03/11/a-program-for-helping-create-lossless-transforms.html#estimator-accuracy-high-compression-level).

### Implementations

> [!TIP]
> These crates have the concrete implementations

- [dxt-lossless-transform-common]: Code shared between multiple internal implementations.
- [dxt-lossless-transform-bc1]: Implementation of the lossless transform for the BC1 block format.
- [dxt-lossless-transform-bc2]: Implementation of the lossless transform for the BC2 block format.
- [dxt-lossless-transform-bc3]: Implementation of the lossless transform for the BC3 block format.
- [dxt-lossless-transform-bc7]: Implementation of the lossless transform for the BC7 block format.

They are low level crates; without a stable API.

## Fuzzing

This project contains a fuzzing crate that can be used to test some of the code against other
implementations.

```text
# You may need to install nightly compiler first:
# https://rust-fuzz.github.io/book/cargo-fuzz/setup.html
# Install cargo-fuzz
cargo install cargo-fuzz

# Run the fuzz target
CARGO_PROFILE_RELEASE_LTO=false cargo fuzz run bc1_decode
```

To find the targets, look at the `fuzz` folder, `Cargo.toml`.

## Usage

This crate is not yet released.

## Testing (CLI)

This project contains a CLI tool that can be used to test the transforms.

```text
./dxt-lossless-transform-cli transform --input textures --output textures-transformed
```

You can either:

1. Build and run the CLI:
   ```bash
   cargo build --release --features "debug-bc1"
   ./target/release/dxt-lossless-transform-cli transform --input textures --output textures-transformed
   ```

2. Run directly via Cargo:
   ```bash
   cargo run --release --bin dxt-lossless-transform-cli --features "debug-bc1" transform --input textures --output textures-transformed
   ```

## Development

For information on how to work with this codebase, see [README-DEV.MD](README-DEV.MD).

## License

Licensed under [GPL v3 (with Reloaded FAQ)](./LICENSE).  
[Learn more about Reloaded's general choice of licensing for projects.][reloaded-license].  

[codecov]: https://about.codecov.io/
[crates-io-key]: https://crates.io/settings/tokens
[nuget-key]: https://www.nuget.org/account/apikeys
[docs]: https://dxt-lossless-transform.github.io/dxt-lossless-transform
[reloaded-license]: https://reloaded-project.github.io/Reloaded.MkDocsMaterial.Themes.R2/Pages/license.html
[dxt-lossless-transform-common]: projects/dxt-lossless-transform-common/README.MD
[dxt-lossless-transform-bc1]: projects/dxt-lossless-transform-bc1/README.MD
[dxt-lossless-transform-bc2]: projects/dxt-lossless-transform-bc2/README.MD
[dxt-lossless-transform-bc3]: projects/dxt-lossless-transform-bc3/README.MD
[dxt-lossless-transform-bc7]: projects/dxt-lossless-transform-bc7/README.MD
[dxt-lossless-transform-api-common]: projects/dxt-lossless-transform-api-common/README.MD
[dxt-lossless-transform-bc1-api]: projects/dxt-lossless-transform-bc1-api/README.MD
[dxt-lossless-transform-bc2-api]: projects/dxt-lossless-transform-bc2-api/README.MD
[dxt-lossless-transform-bc3-api]: projects/dxt-lossless-transform-bc3-api/README.MD
[dxt-lossless-transform-bc7-api]: projects/dxt-lossless-transform-bc7-api/README.MD
[dxt-lossless-transform-dds]: projects/dxt-lossless-transform-dds/README.MD
[dxt-lossless-transform-zstd]: projects/extensions/dxt-lossless-transform-zstd/README.MD
[dxt-lossless-transform-ltu]: projects/extensions/dxt-lossless-transform-ltu/README.MD