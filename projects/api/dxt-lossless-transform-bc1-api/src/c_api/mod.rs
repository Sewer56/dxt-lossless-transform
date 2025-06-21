//! C API exports for BC1 transform operations.
//!
//! This module provides C-compatible exports for BC1 texture compression transforms.
//! All functions are prefixed with `dltbc1_` for uniqueness.
//!
//! ## ABI Stability
//!
//! This module provides two categories of functions:
//!
//! ### ABI-Stable Functions (Recommended)
//! - Functions following the builder pattern or using opaque contexts
//! - These maintain ABI stability across versions
//! - Examples: `dltbc1_TransformContext_Transform`, `dltbc1_TransformContext_*` functions
//!
//! ### ABI-Unstable Functions
//! - Functions prefixed with `dltbc1_unstable_*`
//! - Accept transform details structures directly as parameters
//! - May break between versions if structures change
//! - Provide maximum performance by avoiding context overhead
//! - Examples: `dltbc1_unstable_transform`, `dltbc1_unstable_determine_optimal`

mod determine_optimal_transform;
pub mod error;
pub mod transform;

use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
use dxt_lossless_transform_bc1::{Bc1DetransformDetails, Bc1TransformDetails};

/// FFI-safe version of [`Bc1TransformDetails`] for C API.
///
/// This struct mirrors the internal [`Bc1TransformDetails`] but is guaranteed
/// to have stable ABI layout for C interoperability.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dltbc1TransformDetails {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,
    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

/// FFI-safe version of [`Bc1DetransformDetails`] for C API.
///
/// This struct mirrors the internal [`Bc1DetransformDetails`] but is guaranteed
/// to have stable ABI layout for C interoperability.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dltbc1DetransformDetails {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,
    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

impl Default for Dltbc1TransformDetails {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl Default for Dltbc1DetransformDetails {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

// Conversion implementations
impl From<Bc1TransformDetails> for Dltbc1TransformDetails {
    fn from(details: Bc1TransformDetails) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(details.decorrelation_mode),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Dltbc1TransformDetails> for Bc1TransformDetails {
    fn from(details: Dltbc1TransformDetails) -> Self {
        Self {
            decorrelation_mode: details.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Bc1DetransformDetails> for Dltbc1DetransformDetails {
    fn from(details: Bc1DetransformDetails) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(details.decorrelation_mode),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Dltbc1DetransformDetails> for Bc1DetransformDetails {
    fn from(details: Dltbc1DetransformDetails) -> Self {
        Self {
            decorrelation_mode: details.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}
