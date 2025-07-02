//! BC1 Transform Settings
//!
//! This module contains the configuration structures and related functionality
//! for BC1 transformation operations.

use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Settings for BC1 transform and untransform operations.
///
/// This struct contains the configuration for both transforming and untransforming BC1 data.
/// Each item transformed via [`crate::transform_bc1_with_settings`] will use an instance of this struct.
/// To undo the transform, pass the same settings to [`crate::untransform_bc1_with_settings`].
///
/// Note that color normalization is a preprocessing step that doesn't need to be reversed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1TransformSettings {
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

/// Type alias for backward compatibility.
///
/// [`Bc1UntransformSettings`] is now unified with [`Bc1TransformSettings`] since they were
/// structurally identical. Use [`Bc1TransformSettings`] for both transform and untransform operations.
pub type Bc1UntransformSettings = Bc1TransformSettings;

impl Default for Bc1TransformSettings {
    fn default() -> Self {
        // Best (on average) results, but of course not perfect, as is with brute-force method.
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl Bc1TransformSettings {
    /// Returns an iterator over all possible combinations of [`Bc1TransformSettings`] values.
    ///
    /// This function generates all possible combinations by iterating through:
    /// - All [`YCoCgVariant`] variants  
    /// - Both `true` and `false` values for `split_colour_endpoints`
    ///
    /// The total number of combinations is:
    /// [`YCoCgVariant`] variants × 2 bool values
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_bc1::Bc1TransformSettings;
    ///
    /// let all_combinations: Vec<_> = Bc1TransformSettings::all_combinations().collect();
    /// println!("Total combinations: {}", all_combinations.len());
    ///
    /// for settings in Bc1TransformSettings::all_combinations() {
    ///     println!("{:?}", settings);
    /// }
    /// ```
    #[cfg(not(tarpaulin_include))]
    pub fn all_combinations() -> impl Iterator<Item = Bc1TransformSettings> {
        YCoCgVariant::all_values().iter().flat_map(|decorr_mode| {
            [true, false]
                .into_iter()
                .map(move |split_endpoints| Bc1TransformSettings {
                    decorrelation_mode: *decorr_mode,
                    split_colour_endpoints: split_endpoints,
                })
        })
    }
}

/// Test order for fast mode optimization (tests most important combinations)
pub(crate) static FAST_TEST_ORDER: &[(YCoCgVariant, bool)] = &[
    (YCoCgVariant::None, false),     // None/NoSplit
    (YCoCgVariant::None, true),      // None/Split
    (YCoCgVariant::Variant1, false), // YCoCg1/NoSplit (17.9%)
    (YCoCgVariant::Variant1, true),  // YCoCg1/Split (71.1%) - most common, test last
];

/// Test order for comprehensive mode optimization (tests all combinations)
pub(crate) static COMPREHENSIVE_TEST_ORDER: &[(YCoCgVariant, bool)] = &[
    (YCoCgVariant::Variant2, false), // YCoCg2/NoSplit (0.9%)
    (YCoCgVariant::None, false),     // None/NoSplit (1.0%)
    (YCoCgVariant::None, true),      // None/Split (1.1%)
    (YCoCgVariant::Variant3, false), // YCoCg3/NoSplit (1.9%)
    (YCoCgVariant::Variant3, true),  // YCoCg3/Split (2.7%)
    (YCoCgVariant::Variant2, true),  // YCoCg2/Split (3.5%)
    (YCoCgVariant::Variant1, false), // YCoCg1/NoSplit (17.9%)
    (YCoCgVariant::Variant1, true),  // YCoCg1/Split (71.1%) - most common, test last
];
