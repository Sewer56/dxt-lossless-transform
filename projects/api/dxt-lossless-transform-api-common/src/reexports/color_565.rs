//! # Color565 YCoCg-R Decorrelation Variants (Stable Re-export)
//!
//! This module provides stable API definitions for YCoCg-R (reversible YCoCg) color space
//! decorrelation variants specifically for [`Color565`] values.
//!
//! ## Stability Notice
//!
//! This is a **stable re-export** of types from the internal `dxt-lossless-transform-common`
//! crate. While the internal implementation may change, this API maintains backward
//! compatibility.
//!
//! The internal [`YCoCgVariant`] enum may be refactored, renamed, or moved,
//! but this stable version will continue to work with conversion functions handling any
//! internal changes transparently.
//!
//! ## Overview
//!
//! YCoCg-R is a lifting-based variation of the YCoCg color space that offers perfect reversibility,
//! making it ideal for lossless compression scenarios where [`Color565`] endpoint decorrelation
//! is applied to improve compression efficiency.
//!
//! The YCoCg-R transformation is designed to decorrelate RGB color components to improve compression
//! efficiency for DXT/BC texture formats. The variants differ only in how the transformed bits are
//! arranged within the [`Color565`] format.
//!
//! On real files, the compression differences between variants are negligible. These variants exist
//! primarily for brute-forcing the absolute best possible compression results for people who want
//! to squeeze every last bit of space.
//!
//! ## Variants
//!
//! - **[`YCoCgVariant::Variant1`]**: Standard arrangement  
//!   `Y(11-15) | Co(6-10) | g_low(5) | Cg(0-4)`
//!
//! - **[`YCoCgVariant::Variant2`]**: Low bit at top  
//!   `g_low(15) | Y(10-14) | Co(5-9) | Cg(0-4)`
//!
//! - **[`YCoCgVariant::Variant3`]**: Low bit at bottom  
//!   `Y(11-15) | Co(6-10) | Cg(1-5) | g_low(0)`
//!
//! - **[`YCoCgVariant::None`]**: No transformation (pass-through)
//!
//! [`Color565`]: https://docs.rs/dxt-lossless-transform-common/latest/dxt_lossless_transform_common/color_565/struct.Color565.html

/// Represents a YCoCg-R decorrelation variant for [`Color565`] values.
///
/// The variants differ only in how the transformed bits are arranged within the [`Color565`] format.
/// On real files, the compression differences are negligible. These variants exist primarily for
/// brute-forcing the absolute best possible compression result in specific scenarios.
///
/// This enum is designed for stable API usage where [`Color565`] endpoint decorrelation is needed
/// for DXT/BC texture compression optimization.
///
/// ## Stability Guarantee
///
/// This type provides a stable API boundary. Even if the internal `YCoCgVariant` type changes,
/// this enum will maintain backward compatibility through conversion functions.
///
/// [`Color565`]: https://docs.rs/dxt-lossless-transform-common/latest/dxt_lossless_transform_common/color_565/struct.Color565.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YCoCgVariant {
    /// Variant 1: Standard bit arrangement
    ///
    /// Uses the standard YCoCg-R bit layout:
    /// `Y(11-15) | Co(6-10) | g_low(5) | Cg(0-4)`
    Variant1,

    /// Variant 2: Alternative bit arrangement with low bit placed at top
    ///
    /// Alternative layout with green low bit at the top:
    /// `g_low(15) | Y(10-14) | Co(5-9) | Cg(0-4)`
    Variant2,

    /// Variant 3: Alternative bit arrangement with low bit at bottom
    ///
    /// Alternative layout with green low bit at the bottom:
    /// `Y(11-15) | Co(6-10) | Cg(1-5) | g_low(0)`
    Variant3,

    /// None: No transformation (original RGB values preserved)
    ///
    /// Disables YCoCg-R transformation entirely, preserving original RGB values.
    /// This is useful for content where decorrelation doesn't improve compression.
    None,
}

impl Default for YCoCgVariant {
    /// Returns the default variant (None) which preserves original RGB values.
    fn default() -> Self {
        Self::None
    }
}

impl YCoCgVariant {
    /// Returns all available variants for iteration or testing.
    ///
    /// # Returns
    ///
    /// A slice containing all four variants: [`Variant1`], [`Variant2`], [`Variant3`], and [`None`].
    ///
    /// [`Variant1`]: Self::Variant1
    /// [`Variant2`]: Self::Variant2
    /// [`Variant3`]: Self::Variant3
    /// [`None`]: Self::None
    pub const fn all_variants() -> &'static [Self] {
        &[Self::Variant1, Self::Variant2, Self::Variant3, Self::None]
    }

    /// Returns whether this variant applies any transformation.
    ///
    /// # Returns
    ///
    /// `true` if this variant applies YCoCg-R transformation, `false` for [`None`].
    ///
    /// [`None`]: Self::None
    pub const fn is_transforming(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Converts this stable API variant to the internal common variant.
    ///
    /// This allows the stable API to interface with the internal transformation functions.
    /// This conversion function isolates the stable API from internal type changes.
    ///
    /// # Returns
    ///
    /// The corresponding `YCoCgVariant` from `dxt-lossless-transform-common`.
    pub fn to_internal_variant(self) -> dxt_lossless_transform_common::color_565::YCoCgVariant {
        use dxt_lossless_transform_common::color_565::YCoCgVariant;
        match self {
            Self::Variant1 => YCoCgVariant::Variant1,
            Self::Variant2 => YCoCgVariant::Variant2,
            Self::Variant3 => YCoCgVariant::Variant3,
            Self::None => YCoCgVariant::None,
        }
    }

    /// Creates this stable API variant from the internal common variant.
    ///
    /// This allows conversion from internal types to the stable API.
    /// This conversion function isolates the stable API from internal type changes.
    ///
    /// # Parameters
    ///
    /// - `variant`: The internal `YCoCgVariant` to convert from
    ///
    /// # Returns
    ///
    /// The corresponding stable API variant.
    pub fn from_internal_variant(
        variant: dxt_lossless_transform_common::color_565::YCoCgVariant,
    ) -> Self {
        use dxt_lossless_transform_common::color_565::YCoCgVariant;
        match variant {
            YCoCgVariant::Variant1 => Self::Variant1,
            YCoCgVariant::Variant2 => Self::Variant2,
            YCoCgVariant::Variant3 => Self::Variant3,
            YCoCgVariant::None => Self::None,
        }
    }
}
