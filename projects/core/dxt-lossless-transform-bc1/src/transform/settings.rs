//! BC1 Transform Settings
//!
//! This module contains the configuration structures and related functionality
//! for BC1 transformation operations.

use dxt_lossless_transform_api_common::estimate::DataType;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// The information about the BC1 transform that was just performed.
/// Each item transformed via [`crate::transform_bc1_with_settings`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass [`Bc1DetransformSettings`] to [`crate::untransform_bc1_with_settings`],
/// which can be obtained from this struct using the `into` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1TransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

/// Settings required to detransform BC1 data.
///
/// This struct contains only the information needed to reverse the transform operation.
/// Note that color normalization is a preprocessing step that doesn't need to be reversed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1DetransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

impl From<Bc1TransformSettings> for Bc1DetransformSettings {
    fn from(transform_settings: Bc1TransformSettings) -> Self {
        Self {
            decorrelation_mode: transform_settings.decorrelation_mode,
            split_colour_endpoints: transform_settings.split_colour_endpoints,
        }
    }
}

impl Default for Bc1DetransformSettings {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

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

    /// Determines the appropriate [`DataType`] for size estimation based on the transform options.
    ///
    /// This method maps the transform configuration to the corresponding data type that
    /// should be used for compression size estimation and caching.
    ///
    /// # Returns
    /// The [`DataType`] that represents the data format after applying these transform options
    pub fn to_data_type(&self) -> DataType {
        match (self.decorrelation_mode, self.split_colour_endpoints) {
            (YCoCgVariant::None, false) => DataType::Bc1Colours,
            (YCoCgVariant::None, true) => DataType::Bc1SplitColours,
            (_, true) => DataType::Bc1SplitDecorrelatedColours, // Split colours with decorrelation
            (_, false) => DataType::Bc1DecorrelatedColours,     // Decorrelated but not split
        }
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
