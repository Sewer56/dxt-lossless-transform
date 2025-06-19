//! Color-related intrinsic operations on SIMD registers.
//!
//! This module provides intrinsic functions for performing color-related operations
//! directly on SIMD registers containing [`Color565`] data.
//!
//! [`Color565`]: crate::color_565::Color565

pub mod decorrelate;
pub mod recorrelate;
