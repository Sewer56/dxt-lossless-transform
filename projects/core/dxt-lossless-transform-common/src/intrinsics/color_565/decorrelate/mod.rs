//! YCoCg-R decorrelation intrinsics for [`Color565`] values.
//!
//! This module provides SIMD-optimized intrinsic functions for performing YCoCg-R decorrelation
//! transformations on packed [`Color565`] data. These functions are designed to work with
//! registers containing multiple color values, offering high-performance batch processing.
//!
//! # Available Functions
#![cfg_attr(feature = "nightly", doc = "")]
#![cfg_attr(
    feature = "nightly",
    doc = "## AVX512 Functions (requires `nightly` feature)"
)]
#![cfg_attr(feature = "nightly", doc = "")]
#![cfg_attr(
    feature = "nightly",
    doc = "- [`avx512::decorrelate_ycocg_r_var1_avx512`] - Applies YCoCg-R variant 1 decorrelation"
)]
#![cfg_attr(
    feature = "nightly",
    doc = "- [`avx512::decorrelate_ycocg_r_var2_avx512`] - Applies YCoCg-R variant 2 decorrelation"
)]
#![cfg_attr(
    feature = "nightly",
    doc = "- [`avx512::decorrelate_ycocg_r_var3_avx512`] - Applies YCoCg-R variant 3 decorrelation"
)]
//!
//! ## AVX2 Functions
//!
//! - [`avx2::decorrelate_ycocg_r_var1_avx2`] - Applies YCoCg-R variant 1 decorrelation
//! - [`avx2::decorrelate_ycocg_r_var2_avx2`] - Applies YCoCg-R variant 2 decorrelation
//! - [`avx2::decorrelate_ycocg_r_var3_avx2`] - Applies YCoCg-R variant 3 decorrelation
//!
//! ## SSE2 Functions
//!
//! - [`sse2::decorrelate_ycocg_r_var1_sse2`] - Applies YCoCg-R variant 1 decorrelation
//! - [`sse2::decorrelate_ycocg_r_var2_sse2`] - Applies YCoCg-R variant 2 decorrelation
//! - [`sse2::decorrelate_ycocg_r_var3_sse2`] - Applies YCoCg-R variant 3 decorrelation
//!
//! [`Color565`]: crate::color_565::Color565
//! [`__m512i`]: core::arch::x86_64::__m512i

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;
