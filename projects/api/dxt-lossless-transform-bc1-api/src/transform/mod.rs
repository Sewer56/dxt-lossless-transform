//! BC1 Transform API
//!
//! This module provides high-level builders for BC1 texture transformation:
//!
//! ## Automatic Optimization
//! - [`Bc1AutoTransformBuilder`] - Automatically optimizes transform settings using a size estimator
//!
//! ## Manual Configuration  
//! - [`Bc1ManualTransformBuilder`] - Allows precise control over transform parameters
//!
//! ## Low-level Functions
//! For advanced use cases, the module also provides direct access to the underlying transform functions.

pub(crate) mod auto_transform_builder;
pub(crate) mod manual_transform_builder;

// Re-export the BUILDERS FIRST at the module level for prominence
pub use auto_transform_builder::Bc1AutoTransformBuilder;
pub use manual_transform_builder::Bc1ManualTransformBuilder;

// Re-export stable API types
pub use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

// Import unstable types for internal use only (not re-exported)
use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};
use dxt_lossless_transform_bc1::{
    Bc1DetransformSettings as InternalBc1DetransformSettings,
    Bc1EstimateSettings as InternalBc1EstimateSettings,
    Bc1TransformSettings as InternalBc1TransformSettings,
    DetermineBestTransformError as InternalDetermineBestTransformError,
};
use dxt_lossless_transform_common::allocate::AllocateError;

// Stable API Types
// ================

/// The information about the BC1 transform that was just performed.
///
/// Each item transformed will produce an instance of this struct.
/// To undo the transform, you'll need to pass [`Bc1DetransformSettings`] to the detransform function,
/// which can be obtained from this struct using the `into` method.
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

/// Settings required to detransform BC1 data.
///
/// This struct contains only the information needed to reverse the transform operation.
/// Note that color normalization is a preprocessing step that doesn't need to be reversed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1DetransformSettings {
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

/// The settings for automatic BC1 transform optimization, regarding how the estimation is done,
/// and other related factors.
pub struct Bc1EstimateSettings<T>
where
    T: SizeEstimationOperations,
{
    /// A trait-based size estimator that provides size estimation operations.
    ///
    /// # Remarks
    ///
    /// The estimator should have its compression level and other parameters already configured.
    /// This allows for more flexible usage patterns where different estimators can have
    /// completely different configuration approaches.
    ///
    /// For minimizing file size, use the exact same compression algorithm as the final file will
    /// be compressed with.
    ///
    /// Otherwise consider using a slightly lower level of the same compression function, both to
    /// maximize speed of automatic optimization, and to improve decompression speed
    /// by reducing the size of the sliding window (so more data in cache) and increasing minimum
    /// match length.
    pub size_estimator: T,

    /// Controls which decorrelation modes are tested during optimization.
    ///
    /// When `false` (default), only tests [`YCoCgVariant::Variant1`] and [`YCoCgVariant::None`]
    /// for faster optimization with good results.
    ///
    /// When `true`, tests all available decorrelation modes ([`YCoCgVariant::Variant1`],
    /// [`YCoCgVariant::Variant2`], [`YCoCgVariant::Variant3`], and [`YCoCgVariant::None`])
    /// for potentially better compression at the cost of twice as long optimization
    /// time (tests 4 options instead of 2).
    ///
    /// **Note**: The typical improvement from testing all decorrelation modes is <0.1% in practice.
    /// For better compression gains, it's recommended to use a compression level on the
    /// estimator (e.g., ZStandard estimator) closer to your final compression level instead.
    pub use_all_decorrelation_modes: bool,
}

/// An error that happened during transform determination.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DetermineBestTransformError<E> {
    /// An error that happened in memory allocation within the library
    #[error(transparent)]
    AllocateError(#[from] AllocateError),

    /// An error that happened during size estimation
    #[error("Size estimation failed: {0:?}")]
    SizeEstimationError(E),
}

// Implementations for stable types
// ================================

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
    /// use dxt_lossless_transform_bc1_api::transform::Bc1TransformSettings;
    ///
    /// let all_combinations: Vec<_> = Bc1TransformSettings::all_combinations().collect();
    /// println!("Total combinations: {}", all_combinations.len());
    ///
    /// for settings in Bc1TransformSettings::all_combinations() {
    ///     println!("{:?}", settings);
    /// }
    /// ```
    pub fn all_combinations() -> impl Iterator<Item = Bc1TransformSettings> {
        YCoCgVariant::all_variants().iter().flat_map(|decorr_mode| {
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

// Conversion functions between stable and internal types
// ======================================================

impl From<Bc1TransformSettings> for InternalBc1TransformSettings {
    fn from(stable: Bc1TransformSettings) -> Self {
        Self {
            decorrelation_mode: stable.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: stable.split_colour_endpoints,
        }
    }
}

impl From<InternalBc1TransformSettings> for Bc1TransformSettings {
    fn from(internal: InternalBc1TransformSettings) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(internal.decorrelation_mode),
            split_colour_endpoints: internal.split_colour_endpoints,
        }
    }
}

impl From<Bc1DetransformSettings> for InternalBc1DetransformSettings {
    fn from(stable: Bc1DetransformSettings) -> Self {
        Self {
            decorrelation_mode: stable.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: stable.split_colour_endpoints,
        }
    }
}

impl From<InternalBc1DetransformSettings> for Bc1DetransformSettings {
    fn from(internal: InternalBc1DetransformSettings) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(internal.decorrelation_mode),
            split_colour_endpoints: internal.split_colour_endpoints,
        }
    }
}

impl<T> From<Bc1EstimateSettings<T>> for InternalBc1EstimateSettings<T>
where
    T: SizeEstimationOperations,
{
    fn from(stable: Bc1EstimateSettings<T>) -> Self {
        Self {
            size_estimator: stable.size_estimator,
            use_all_decorrelation_modes: stable.use_all_decorrelation_modes,
        }
    }
}

impl<T> From<InternalBc1EstimateSettings<T>> for Bc1EstimateSettings<T>
where
    T: SizeEstimationOperations,
{
    fn from(internal: InternalBc1EstimateSettings<T>) -> Self {
        Self {
            size_estimator: internal.size_estimator,
            use_all_decorrelation_modes: internal.use_all_decorrelation_modes,
        }
    }
}

impl<E> From<DetermineBestTransformError<E>> for InternalDetermineBestTransformError<E> {
    fn from(stable: DetermineBestTransformError<E>) -> Self {
        match stable {
            DetermineBestTransformError::AllocateError(err) => Self::AllocateError(err),
            DetermineBestTransformError::SizeEstimationError(err) => Self::SizeEstimationError(err),
        }
    }
}

impl<E> From<InternalDetermineBestTransformError<E>> for DetermineBestTransformError<E> {
    fn from(internal: InternalDetermineBestTransformError<E>) -> Self {
        match internal {
            InternalDetermineBestTransformError::AllocateError(err) => Self::AllocateError(err),
            InternalDetermineBestTransformError::SizeEstimationError(err) => {
                Self::SizeEstimationError(err)
            }
        }
    }
}

// Stable types are automatically public since they're defined with `pub` above
