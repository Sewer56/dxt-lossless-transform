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
- 2 bytes of alpha endpoints (8 bits each)
- 6 bytes of alpha indices (16x 3-bit)
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

### Making Single Colour Blocks Represented the Same Across Texture

BC3 can represent single color blocks in multiple ways, which hinders compression.

Recall BC3 block format:

```text
Address: 0       2          8   8        12        16
         +-------+----------+   +---------+---------+
Data:    | A0-A1 | AI0-AI15 |   | C00-C01 | I00-I15 |
         +-------+----------+   +---------+---------+
```

Two bytes of Alpha (8 bits each) followed by 6 bytes of indices (3 bits each).
The indices are used to interpolate the alpha values.

There are 2 steps to normalizing here:

#### Normalizing the Alphas

Sometimes all items within a block have the same alpha component, for example, all pixels in a block
are fully opaque (`0xFF`), or are semi transparent (`0xD0`). 

When a block is fully opaque, there are multiple well compressible representations which can be used:

- All 8 bytes set to `0xFF` (`0xFFFFFFFF 0xFFFFFFFF`).
  - Because `A0` <= `A1`, index 0xFF is hardcoded to opaque on decoder side.
- Zero alphas but indices set to `0xFF` (`0x0000FFFF 0xFFFFFFFF`).

For all other values (including `0xFF`), the following representation is possible:

- `A0` set to `0xFF` and everything else set to `0x00` (`0xFF000000 0x00000000`).
  - Everything uses colour from the first item.

Which means there's `3` modes for fully opaque, and single `1` mode for everything else.

#### Normalizing the Colours

(This is the same as BC2)

Sometimes there is a clean conversion between `8888` and `565` colour, meaning that it's possible
to represent a colour using only 1 out of the 2 colour components, i.e. only `C0` or `C1`.

When an entire block is solid, and a clean conversion between `8888` and `565` is possible, we set
colour `C0` to the colour of the block, and then set `C1`, `I0-I3` all to `0x00`. This results in a
nice repetition of `0x00` across 6 bytes. In some cases, it's beneficial to replicate
the colour across `C0` and `C1`. We determine based on whichever performs better by compressing the data.

Note: With BC2, BC3 it's important that we put the colour in `C0` because the 'alternate alpha mode' of
BC1 where `c0 <= c1` is unsupported; it leads to undefined behaviour on some GPUs.

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