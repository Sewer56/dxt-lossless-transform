//! # Block Normalization Process
//!
//! This module contains the code used to normalize BC1 blocks to improve compression ratio
//! by making solid color blocks and transparent blocks have consistent representations.
//!
//! ## BC1 Block Format
//!
//! First, let's recall the BC1 block format:
//!
//! ```text
//! Address: 0        2        4       8
//!          +--------+--------+--------+
//! Data:    | Color0 | Color1 | Indices|
//!          +--------+--------+--------+
//! ```
//!
//! Where:
//! - `Color0` and `Color1` are 16-bit RGB565 color values (2 bytes each)
//! - `Indices` are 4 bytes containing sixteen 2-bit indices (one for each pixel in the 4x4 block)
//!
//! ## Normalization Rules
//!
//! The normalization process applies the following rules to improve compression:
//!
//! ### 1. Solid Color Blocks
//!
//! When an entire block represents a single solid color with a clean conversion between RGBA8888
//! and RGB565, we standardize the representation:
//!
//! ```text
//! +--------+--------+--------+
//! | Color  | 0x0000 |  0x00  |
//! +--------+--------+--------+
//! ```
//!
//! We place the block's color in `Color0`, set `Color1` to zero, and set all indices to zero
//! (represented by all-zero bytes in the indices section). Can also be repeat of same byte.
//!
//! The implementation checks for this case by:
//! 1. Decoding the block to get all 16 pixels
//! 2. Checking that all pixels have the same color
//! 3. Verifying the color can be cleanly round-tripped through RGB565 encoding
//! 4. Constructing a new normalized block with the pattern above
//!
//! ### 2. Fully Transparent Blocks
//!
//! For blocks that are completely transparent (common in textures with alpha), we standardize
//! the representation to all 1's:
//!
//! ```text
//! +--------+--------+--------+
//! | 0xFFFF | 0xFFFF | 0xFFFF |
//! +--------+--------+--------+
//! ```
//!
//! The implementation detects transparent blocks by:
//! 1. Decoding all 16 pixels in the block
//! 2. Checking if all pixels have alpha=0 (check if first pixel is transparent, after checking if all are equal)
//! 3. Setting the entire block content to 0xFF bytes
//!
//! ### 3. Mixed Transparency Blocks
//!
//! In BC1, when `Color0 <= Color1`, the block is in "punch-through alpha" mode, where index `11`
//! represents a transparent pixel. Blocks containing both opaque and transparent pixels
//! (mixed alpha) use this mode.
//!
//! For these blocks, we can't apply significant normalization without changing the visual
//! appearance, so we preserve them unchanged.
//!
//! ## Implementation Details
//!
//! The normalization process uses the BC1 decoder to analyze the block content, then rebuilds
//! blocks according to the rules above.
//!
//! When normalizing blocks, we:
//!
//! 1. Look at the RGB565 color values to determine if we're in alpha mode (`Color0 <= Color1`)
//! 2. Decode the block to get the 16 pixels with their colors
//! 3. Apply one of the three normalization cases based on the block properties
//! 4. Write the normalized block to the output

/// See [`super`] for the exact details.
pub mod normalize;
pub mod transform;

pub use normalize::*;
pub use transform::*;

use crate::{Bc1DetransformSettings, Bc1TransformSettings, YCoCgVariant};

/// The information about the BC1 transform that was just performed with experimental normalization support.
/// Each item transformed via [`transform_bc1_with_normalize_blocks`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass [`Bc1DetransformSettings`] to [`crate::untransform_bc1_with_settings`],
/// which can be obtained from this struct using the `into` method.
///
/// [`Bc1DetransformSettings`]: crate::Bc1DetransformSettings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1TransformDetailsWithNormalization {
    /// The color normalization mode that was used to normalize the data.
    pub color_normalization_mode: ColorNormalizationMode,

    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    ///
    /// This setting controls whether BC1 texture color endpoints are separated during processing,
    /// which can improve compression efficiency for many textures.
    ///
    /// **File Size**: This setting reduces file size around 78% of the time.
    pub split_colour_endpoints: bool,
}

impl From<Bc1TransformDetailsWithNormalization> for Bc1DetransformSettings {
    fn from(transform_details: Bc1TransformDetailsWithNormalization) -> Self {
        Self {
            decorrelation_mode: transform_details.decorrelation_mode,
            split_colour_endpoints: transform_details.split_colour_endpoints,
        }
    }
}

impl Default for Bc1TransformDetailsWithNormalization {
    fn default() -> Self {
        // Best (on average) results, but of course not perfect, as is with brute-force method.
        Self {
            color_normalization_mode: ColorNormalizationMode::None,
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl From<Bc1TransformSettings> for Bc1TransformDetailsWithNormalization {
    fn from(transform_details: Bc1TransformSettings) -> Self {
        Self {
            color_normalization_mode: ColorNormalizationMode::None,
            decorrelation_mode: transform_details.decorrelation_mode,
            split_colour_endpoints: transform_details.split_colour_endpoints,
        }
    }
}

impl Bc1TransformDetailsWithNormalization {
    /// Returns an iterator over all possible combinations of [`Bc1TransformDetailsWithNormalization`] values.
    ///
    /// This function generates all possible combinations by iterating through:
    /// - All [`ColorNormalizationMode`] variants
    /// - All [`YCoCgVariant`] variants  
    /// - Both `true` and `false` values for `split_colour_endpoints`
    ///
    /// The total number of combinations is:
    /// [`ColorNormalizationMode`] variants × [`YCoCgVariant`] variants × 2 bool values
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_bc1::experimental::normalize_blocks::Bc1TransformDetailsWithNormalization;
    ///
    /// let all_combinations: Vec<_> = Bc1TransformDetailsWithNormalization::all_combinations().collect();
    /// println!("Total combinations: {}", all_combinations.len());
    ///
    /// for details in Bc1TransformDetailsWithNormalization::all_combinations() {
    ///     println!("{:?}", details);
    /// }
    /// ```
    #[cfg(not(tarpaulin_include))]
    pub fn all_combinations() -> impl Iterator<Item = Bc1TransformDetailsWithNormalization> {
        ColorNormalizationMode::all_values()
            .iter()
            .flat_map(|color_mode| {
                YCoCgVariant::all_values()
                    .iter()
                    .flat_map(move |decorr_mode| {
                        [true, false].into_iter().map(move |split_endpoints| {
                            Bc1TransformDetailsWithNormalization {
                                color_normalization_mode: *color_mode,
                                decorrelation_mode: *decorr_mode,
                                split_colour_endpoints: split_endpoints,
                            }
                        })
                    })
            })
    }
}
