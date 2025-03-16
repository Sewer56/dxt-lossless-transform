# dxt-lossless-transform-bc3

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform-bc3.svg)](https://crates.io/crates/dxt-lossless-transform-bc3)
[![Docs.rs](https://docs.rs/dxt-lossless-transform-bc3/badge.svg)](https://docs.rs/dxt-lossless-transform-bc3)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

## About

This crate provides the implementation of a fast, visually lossless transform for the BC3 block format.  
This is the low level implementation for the crate. For the high level API, see [dxt-lossless-transform-bc3-api].  

## Low Level Details

This describes the tricks used under the hood, in a short concise manner, for in depth explanation,
navigate to the corresponding links in each section.

### Splitting Blocks

We split BC3 blocks into two separate arrays of colours and indices.

#### Original BC3 data layout (as received from file)

Two 16-bit colours (4 bytes total) followed by 4 bytes of indices:

```text
Address: 0       2        8       12      16  16      18        24      28      32
         +-------+--------+-------+-------+   +-------+---------+-------+-------+
Data:    | A0-A1 |AIdx0-47| C0-C1 |I0-I15 |   | A2-A3 |AIdx48-95| C2-C3 |I16-I31|
         +-------+--------+-------+-------+   +-------+---------+-------+-------+
```

Each 16-byte block contains:
- 8 bytes of explicit alpha (sixteen 4-bit alpha values)
- 4 bytes colours (2x RGB565 values)
- 4 bytes of packed color indices (sixteen 2-bit indices)

#### Optimized layout

Separates colours and indices into continuous streams:

```text
+-------+-------+-------+     +-------+  } Alpha endpoints section
| A0-A1 | A2-A3 | A4-A5 | ... | AN    |  } (2 bytes per block: 2x 8-bit)
+-------+-------+-------+     +-------+
+-------+-------+-------+     +-------+  } Alpha indices section
|AI0-47 |AI48-95|  ...  | ... |AI N   |  } (6 bytes per block: 16x 3-bit)
+-------+-------+-------+     +-------+
+-------+-------+-------+     +-------+  } Colours section
|C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
+-------+-------+-------+     +-------+
+-------+-------+-------+     +-------+  } Indices section
| Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
+-------+-------+-------+     +-------+
```

### Making Solid Blocks Represented the Same Across Texture

BC3 can represent solid color blocks in multiple ways, which hinders compression.

- TODO

### Decorrelating Colours

An increase in one colour channel often means an increase in others. 
i.e. They are `correlated`.

We can often apply `decorrelation` to improve compression ratio in such cases.
We apply the [YCoCg-R] transform to decorrelate the colours.

Since the colours are stored in `565`, we use the upper 5 bits of the green endpoint, leaving the
remaining bit untouched.

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
[dxt-lossless-transform-bc3-api]: https://github.com/Sewer56/dxt-lossless-transform/tree/main/projects/dxt-lossless-transform-bc3-api