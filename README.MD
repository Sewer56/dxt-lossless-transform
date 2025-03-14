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

- [dxt-lossless-transform-api]: A 'master' crate that adds a stable API
- [dxt-lossless-transform-bc1-api]: A stable API for [dxt-lossless-transform-bc1].
- [dxt-lossless-transform-bc2-api]: A stable API for [dxt-lossless-transform-bc2].
- [dxt-lossless-transform-bc3-api]: A stable API for [dxt-lossless-transform-bc3].
- [dxt-lossless-transform-bc7-api]: A stable API for [dxt-lossless-transform-bc7].

### Utility Libraries

- [dxt-lossless-transform-dds]: Adds DDS support for the dxt-lossless-transform crate.

### Implementations

> [!TIP]
> These crates have the concrete implementations

- [dxt-lossless-transform-common]: Code shared between multiple internal implementations.
- [dxt-lossless-transform-bc1]: Implementation of the lossless transform for the BC1 block format.
- [dxt-lossless-transform-bc2]: Implementation of the lossless transform for the BC2 block format.
- [dxt-lossless-transform-bc3]: Implementation of the lossless transform for the BC3 block format.
- [dxt-lossless-transform-bc7]: Implementation of the lossless transform for the BC7 block format.

They are low level crates; without a stable API.

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
[reloaded-license]: https://reloaded-project.github.io/Reloaded.MkDocsMaterial.Themes.R2/Pages/license.html
[dxt-lossless-transform-common]: projects/dxt-lossless-transform-common/README.MD
[dxt-lossless-transform-bc1]: projects/dxt-lossless-transform-bc1/README.MD
[dxt-lossless-transform-bc2]: projects/dxt-lossless-transform-bc2/README.MD
[dxt-lossless-transform-bc3]: projects/dxt-lossless-transform-bc3/README.MD
[dxt-lossless-transform-bc7]: projects/dxt-lossless-transform-bc7/README.MD
[dxt-lossless-transform-api]: projects/dxt-lossless-transform-api/README.MD
[dxt-lossless-transform-bc1-api]: projects/dxt-lossless-transform-bc1-api/README.MD
[dxt-lossless-transform-bc2-api]: projects/dxt-lossless-transform-bc2-api/README.MD
[dxt-lossless-transform-bc3-api]: projects/dxt-lossless-transform-bc3-api/README.MD
[dxt-lossless-transform-bc7-api]: projects/dxt-lossless-transform-bc7-api/README.MD