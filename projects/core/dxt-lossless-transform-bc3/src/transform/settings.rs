//! BC3 Transform Settings
//!
//! This module contains the configuration structures and related functionality
//! for BC3 transformation operations.

use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Settings for BC3 transform and untransform operations.
///
/// This struct contains the configuration for both transforming and untransforming BC3 data.
/// Each item transformed via [`crate::transform_bc3_with_settings`] will use an instance of this struct.
/// To undo the transform, pass the same settings to [`crate::untransform_bc3_with_settings`].
///
/// Note that color normalization is a preprocessing step that doesn't need to be reversed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc3TransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the alpha endpoints are to be split or not.
    ///
    /// This setting controls whether BC3 texture alpha endpoints are separated during processing,
    /// which can improve compression efficiency for many textures.
    pub split_alpha_endpoints: bool,

    /// Whether or not the colour endpoints are to be split or not.
    ///
    /// This setting controls whether BC3 texture color endpoints are separated during processing,
    /// which can improve compression efficiency for many textures.
    pub split_colour_endpoints: bool,
}

/// Type alias for backward compatibility.
///
/// [`Bc3UntransformSettings`] is now unified with [`Bc3TransformSettings`] since they were
/// structurally identical. Use [`Bc3TransformSettings`] for both transform and untransform operations.
pub type Bc3UntransformSettings = Bc3TransformSettings;

impl Default for Bc3TransformSettings {
    fn default() -> Self {
        // Best (on average) results, but of course not perfect, as is with brute-force method.
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_alpha_endpoints: true,
            split_colour_endpoints: true,
        }
    }
}

impl Bc3TransformSettings {
    /// Returns an iterator over all possible combinations of [`Bc3TransformSettings`] values.
    ///
    /// This function generates all possible combinations by iterating through:
    /// - All [`YCoCgVariant`] variants  
    /// - Both `true` and `false` values for `split_alpha_endpoints`
    /// - Both `true` and `false` values for `split_colour_endpoints`
    ///
    /// The total number of combinations is:
    /// [`YCoCgVariant`] variants × 2 bool values × 2 bool values
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_bc3::Bc3TransformSettings;
    ///
    /// let all_combinations: Vec<_> = Bc3TransformSettings::all_combinations().collect();
    /// println!("Total combinations: {}", all_combinations.len());
    ///
    /// for settings in Bc3TransformSettings::all_combinations() {
    ///     println!("{:?}", settings);
    /// }
    /// ```
    #[cfg(not(tarpaulin_include))]
    pub fn all_combinations() -> impl Iterator<Item = Bc3TransformSettings> {
        YCoCgVariant::all_values().iter().flat_map(|decorr_mode| {
            [true, false].into_iter().flat_map(move |split_alphas| {
                [true, false]
                    .into_iter()
                    .map(move |split_colours| Bc3TransformSettings {
                        decorrelation_mode: *decorr_mode,
                        split_alpha_endpoints: split_alphas,
                        split_colour_endpoints: split_colours,
                    })
            })
        })
    }
}

// TODO: Set this order based on real file results.
/// Test order for fast mode optimization (tests most important combinations)
pub(crate) static FAST_TEST_ORDER: &[(YCoCgVariant, bool, bool)] = &[
    (YCoCgVariant::None, false, false), // Standard/None/NoSplit/NoSplit
    (YCoCgVariant::None, true, false),  // Standard/None/SplitAlpha/NoSplit
    (YCoCgVariant::None, false, true),  // Standard/None/NoSplit/SplitColour
    (YCoCgVariant::None, true, true),   // Standard/None/SplitAlpha/SplitColour
    (YCoCgVariant::Variant1, false, false), // YCoCg1/NoSplit/NoSplit
    (YCoCgVariant::Variant1, true, false), // YCoCg1/SplitAlpha/NoSplit
    (YCoCgVariant::Variant1, false, true), // YCoCg1/NoSplit/SplitColour
    (YCoCgVariant::Variant1, true, true), // YCoCg1/SplitAlpha/SplitColour - most common, test last
];

/// Test order for comprehensive mode optimization (tests all combinations)
pub(crate) static COMPREHENSIVE_TEST_ORDER: &[(YCoCgVariant, bool, bool)] = &[
    (YCoCgVariant::Variant2, false, false), // YCoCg2/NoSplit/NoSplit
    (YCoCgVariant::Variant2, true, false),  // YCoCg2/SplitAlpha/NoSplit
    (YCoCgVariant::Variant2, false, true),  // YCoCg2/NoSplit/SplitColour
    (YCoCgVariant::Variant2, true, true),   // YCoCg2/SplitAlpha/SplitColour
    (YCoCgVariant::Variant3, false, false), // YCoCg3/NoSplit/NoSplit
    (YCoCgVariant::Variant3, true, false),  // YCoCg3/SplitAlpha/NoSplit
    (YCoCgVariant::Variant3, false, true),  // YCoCg3/NoSplit/SplitColour
    (YCoCgVariant::Variant3, true, true),   // YCoCg3/SplitAlpha/SplitColour
    (YCoCgVariant::None, false, false),     // Standard/None/NoSplit/NoSplit
    (YCoCgVariant::None, true, false),      // Standard/None/SplitAlpha/NoSplit
    (YCoCgVariant::None, false, true),      // Standard/None/NoSplit/SplitColour
    (YCoCgVariant::None, true, true),       // Standard/None/SplitAlpha/SplitColour
    (YCoCgVariant::Variant1, false, false), // YCoCg1/NoSplit/NoSplit
    (YCoCgVariant::Variant1, true, false),  // YCoCg1/SplitAlpha/NoSplit
    (YCoCgVariant::Variant1, false, true),  // YCoCg1/NoSplit/SplitColour
    (YCoCgVariant::Variant1, true, true), // YCoCg1/SplitAlpha/SplitColour - most common, test last
];
