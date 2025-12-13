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
> The project is organized into several main directories under `/src/`:

### Project Structure

- **`/src/core/`** - Houses the **unstable API** implementations (low-level crates without API stability guarantees)
- **`/src/api/`** - Houses the **stable API** (recommended for external users)
- **`/src/extensions/`** - Contains extensions that build on top of the stable API
- **`/src/tools/`** - Contains miscellaneous programs and utilities

Click on a project to navigate to its description.

### Stable API (`/src/api/`)

> [!TIP]
> These provide stable, backwards-compatible interfaces. Recommended for external users.

- [dxt-lossless-transform-api-common]: Shared functionality for all stable API crates.
- [dxt-lossless-transform-bc1-api]: A stable API for [dxt-lossless-transform-bc1].
- [dxt-lossless-transform-bc2-api]: A stable API for [dxt-lossless-transform-bc2].
- [dxt-lossless-transform-bc3-api]: A stable API for [dxt-lossless-transform-bc3].
- [dxt-lossless-transform-bc7-api]: A stable API for [dxt-lossless-transform-bc7].
- [dxt-lossless-transform-file-formats-api]: File format handling with stable API.

### Extensions (`/src/extensions/`)

#### File Format Support

- [dxt-lossless-transform-dds]: Adds DDS support for the dxt-lossless-transform crate.

#### Estimator Libraries

> [!TIP]
> 
> These libraries help you determine the best transform settings for each algorithm.

Speeds are based on a single core on a Ryzen 9950X3D CPU, and are approximate values.

- **ZStandard (zstd)**: Uses ZStandard compression to determine best transform settings.
  - Package: [dxt-lossless-transform-zstd]
  - Speed (1 Thread): ~1060MiB/s (level 1)
  - Accuracy: 79.2% (level 1) (vs ZStandard)
  - Recommended for zstd compression level 4 and above.
  - Level 1 is recommended for a 'balanced' profile; other levels deliver increasingly diminishing returns.

- **Lossless Transform Utils (ltu)**: Fast estimator for generic LZ-based compression algorithms. 
  - Package: [dxt-lossless-transform-ltu]
  - Speed: ~2565MiB/s single-threaded.
  - Accuracy: ~74.4% for high compression (vs ZStandard)
  - Recommended for zstd compression levels 1-3.
  - Optimized for speed while maintaining reasonable accuracy; suited for real-time compression scenarios.

For more information, [refer to my blog post](https://sewer56.dev/blog/2025/03/11/a-program-for-helping-create-lossless-transforms.html#estimator-accuracy-high-compression-level).

### Core Implementations (`/src/core/`) - Unstable API

> [!WARNING]
> These crates have the concrete implementations but **no stable API**. Use the stable API crates instead.

- [dxt-lossless-transform-common]: Code shared between multiple internal implementations.
- [dxt-lossless-transform-bc1]: Implementation of the lossless transform for the BC1 block format.
- [dxt-lossless-transform-bc2]: Implementation of the lossless transform for the BC2 block format.
- [dxt-lossless-transform-bc3]: Implementation of the lossless transform for the BC3 block format.
- [dxt-lossless-transform-bc7]: Implementation of the lossless transform for the BC7 block format.

They are low level crates optimized for maximum performance with frequent breaking changes.

### Tools (`/src/tools/`)

- CLI tools and debugging utilities
- Development and testing applications
- Example implementations and benchmarks

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

## Developer Manual

For step-by-step development guidance, see the [Developer Manual](https://reloaded-project.github.io/reloaded-templates-rust/manual/).

## Contributing

We welcome contributions! See the [Contributing Guide](https://reloaded-project.github.io/reloaded-templates-rust/manual/#contributing) for details.

## License

Licensed under [GPL v3 (with Reloaded FAQ)](./LICENSE).

[codecov]: https://about.codecov.io/
[crates-io-key]: https://crates.io/settings/tokens
[nuget-key]: https://www.nuget.org/account/apikeys
[docs]: https://dxt-lossless-transform.github.io/dxt-lossless-transform
[dxt-lossless-transform-common]: src/core/dxt-lossless-transform-common/README.MD
[dxt-lossless-transform-bc1]: src/core/dxt-lossless-transform-bc1/README.MD
[dxt-lossless-transform-bc2]: src/core/dxt-lossless-transform-bc2/README.MD
[dxt-lossless-transform-bc3]: src/core/dxt-lossless-transform-bc3/README.MD
[dxt-lossless-transform-bc7]: src/core/dxt-lossless-transform-bc7/README.MD
[dxt-lossless-transform-api-common]: src/api/dxt-lossless-transform-api-common/README.MD
[dxt-lossless-transform-bc1-api]: src/api/dxt-lossless-transform-bc1-api/README.MD
[dxt-lossless-transform-bc2-api]: src/api/dxt-lossless-transform-bc2-api/README.MD
[dxt-lossless-transform-bc3-api]: src/api/dxt-lossless-transform-bc3-api/README.MD
[dxt-lossless-transform-bc7-api]: src/api/dxt-lossless-transform-bc7-api/README.MD
[dxt-lossless-transform-dds]: src/extensions/file-formats/dxt-lossless-transform-dds/README.MD
[dxt-lossless-transform-file-formats-api]: src/api/dxt-lossless-transform-file-formats-api/README.md
[dxt-lossless-transform-zstd]: src/extensions/compressors/dxt-lossless-transform-zstd/README.MD
[dxt-lossless-transform-ltu]: src/extensions/estimators/dxt-lossless-transform-ltu/README.MD