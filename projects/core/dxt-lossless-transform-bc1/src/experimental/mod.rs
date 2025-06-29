//! Experimental features, not ready for prime time yet.
//! Use at your own risk! Expect API to be very unstable.

pub use crate::experimental::normalize_blocks::*;
pub mod normalize_blocks;

use crate::{Bc1TransformSettings, Bc1UntransformSettings, YCoCgVariant};

/// The information about the BC1 transform that was just performed with experimental normalization support.
/// Each item transformed via [`normalize_blocks::transform_bc1_with_normalize_blocks`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass [`Bc1UntransformSettings`] to [`crate::untransform_bc1_with_settings`],
/// which can be obtained from this struct using the `into` method.
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

impl From<Bc1TransformDetailsWithNormalization> for Bc1UntransformSettings {
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
    /// use dxt_lossless_transform_bc1::experimental::Bc1TransformDetailsWithNormalization;
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
