# dxt-lossless-transform-bc1

[![Crates.io](https://img.shields.io/crates/v/dxt-lossless-transform-bc1.svg)](https://crates.io/crates/dxt-lossless-transform-bc1)
[![Docs.rs](https://docs.rs/dxt-lossless-transform-bc1/badge.svg)](https://docs.rs/dxt-lossless-transform-bc1)
[![CI](https://github.com/Sewer56/dxt-lossless-transform/actions/workflows/rust.yml/badge.svg)](https://github.com/Sewer56/dxt-lossless-transform/actions)

## About

This crate provides the implementation of a fast, visually lossless transform for the BC1 block format.  
This is the low level implementation for the crate. For the high level API, see [dxt-lossless-transform-bc1-api].  

## Low Level Details

This describes the tricks used under the hood, in a short concise manner, for in depth explanation,
navigate to the corresponding links in each section.

### Splitting Blocks

We split BC1 blocks into two separate arrays of colours and indices.

#### Original BC1 data layout (as received from file)

Two 16-bit colours (4 bytes total) followed by 4 bytes of indices:

```text
Address: 0       4       8   8      12      16
         +-------+-------+   +-------+-------+
Data:    | C0-C1 | I0-I3 |   | C2-C3 | I4-I8 |
         +-------+-------+   +-------+-------+
```

Each 8-byte block contains:
- 4 bytes colours (2x RGB565 values)
- 4 bytes of packed indices (sixteen 2-bit indices)

#### Optimized layout

Separates colours and indices into continuous streams:

```text
+-------+-------+-------+     +-------+  } colours section
|C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
+-------+-------+-------+     +-------+
+-------+-------+-------+     +-------+  } Indices section
| Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
+-------+-------+-------+     +-------+
```

### Splitting the Colour Endpoints

Each BC1 texture has 2 colour endpoints, `color0` and `color1`.
It is sometimes beneficial to separate these, i.e. store them separately.

**File Size**: This technique reduces file size around 78% of the time.

Take our optimized layout from earlier:

```text
+-------+-------+-------+
|C0  C1 |C0  C1 |C0  C1 |
+-------+-------+-------+
```

We can split the colour endpoints

```text
+-------+-------+ +-------+-------+
|C0  C0 |C0  C0 | |C1  C1 |C1  C1 |
+-------+-------+ +-------+-------+
```

### Making Solid Blocks Represented the Same Across Texture (Normalizing Blocks) `[Experimental]`

BC1 can represent solid color blocks in multiple ways.
We make this representation consistent whenever possible.

Recall BC1 block format:

```text
Address: 0       4       8   8      12      16
         +-------+-------+   +-------+-------+
Data:    | C0-C1 | I0-I3 |   | C2-C3 | I4-I8 |
         +-------+-------+   +-------+-------+
```

Sometimes there is a clean conversion between `8888` and `565` colour, meaning that it's possible
to represent a colour using only 1 out of the 2 colour components, i.e. only `C0` or `C1`.

When an entire block is solid, and a clean conversion between `8888` and `565` is possible, we set
colour `C0` to the colour of the block, and then set `C1`, `I0-I3` all to `0x00`.
This results in a nice repetition of `0x00` across 6 bytes. In some cases, it's beneficial to replicate
the colour across `C0` and `C1`. We determine based on whichever performs better by compressing the data.

For fully transparent blocks, we instead set all the bits to `1`, so the block becomes filled with `0xFF`. 

If the block consists of both transparent and non-transparent pixels, i.e. `color0 <= color1` 
and there is a mix of `11` and non `11` indices, then we leave the block intact; we can't do much
with it.

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
[dxt-lossless-transform-bc1-api]: https://github.com/Sewer56/dxt-lossless-transform/tree/main/projects/dxt-lossless-transform-bc1-api
[YCoCg-R]: https://en.wikipedia.org/wiki/YCoCg#The_lifting-based_YCoCg-R_variation