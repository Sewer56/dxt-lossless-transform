# dxt-lossless-transform-bc2

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform-bc2.svg)](https://crates.io/crates/dxt-lossless-transform-bc2)
[![Docs.rs](https://docs.rs/dxt-lossless-transform-bc2/badge.svg)](https://docs.rs/dxt-lossless-transform-bc2)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

## About

This crate provides the implementation of a fast, visually lossless transform for the BC2 block format.  
This is the low level implementation for the crate. For the high level API, see [dxt-lossless-transform-bc2-api].  

## Low Level Details

This describes the tricks used under the hood, in a short concise manner, for in depth explanation,
navigate to the corresponding links in each section.

### Splitting Blocks

We split BC2 blocks into two separate arrays of colours and indices.

#### Original BC2 data layout (as received from file)

Two 16-bit colours (4 bytes total) followed by 4 bytes of indices:

```text
Address: 0       8       12      16  16      24      28      32
         +-------+-------+-------+   +-------+-------+--------+
Data:    |A0-A15 | C0-C1 | I0-I15 |  |A16-A31| C2-C3 | I6-I31 |
         +-------+-------+-------+   +-------+-------+--------+
```

Each 16-byte block contains:
- 8 bytes of explicit alpha (sixteen 4-bit alpha values)
- 4 bytes colours (2x RGB565 values)
- 4 bytes of packed color indices (sixteen 2-bit indices)

#### Optimized layout

Separates colours and indices into continuous streams:

```text
+-------+-------+-------+     +-------+  } Alpha section
| A0    | A1    | A2    | ... | AN    |  } (8 bytes per block: 16x 4-bit)
+-------+-------+-------+     +-------+
+-------+-------+-------+     +-------+  } Colours section
|C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
+-------+-------+-------+     +-------+
+-------+-------+-------+     +-------+  } Indices section
| Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
+-------+-------+-------+     +-------+
```

### Making Solid Blocks Represented the Same Across Texture

BC2 can represent solid color blocks in multiple ways, which hinders compression.

Recall BC2 block format:

```text
Address: 0         8         12       16
         +---------+---------+---------+
Data:    | A00-A15 | C00-C01 | I00-I15 |
         +---------+---------+---------+
```

Sometimes there is a clean conversion between `8888` and `565` colour, meaning that it's possible
to represent a colour using only 1 out of the 2 colour components, i.e. only `C0` or `C1`.

When an entire block is solid, and a clean conversion between `8888` and `565` is possible, we set
colour `C0` to the colour of the block, and then set `C1`, `I0-I3` all to `0x00`. This results in a
nice repetition of `0x00` across 6 bytes. In some cases, it's beneficial to replicate
the colour across `C0` and `C1`. We determine on whichever performs better.

Note: With BC2, BC3 it's important that we put the colour in `C0`, because the 'alternate alpha mode' of
BC1 where `c0 <= c1` is unsupported; it leads to undefined behaviour on some GPUs.

There isn't much more we can do for BC2, so we stop here:

- For fully transparent blocks, the `alphas` have the value `0`. Since they are in their own isolated section,
we can't really improve the compression there.

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
[dxt-lossless-transform-bc2-api]: https://github.com/Sewer56/dxt-lossless-transform/tree/main/projects/dxt-lossless-transform-bc2-api