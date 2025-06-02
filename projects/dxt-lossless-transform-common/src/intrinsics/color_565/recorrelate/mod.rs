//! YCoCg-R recorrelation intrinsics for [`Color565`] values.
//!
//! This module provides SIMD-optimized intrinsic functions for performing YCoCg-R recorrelation
//! transformations on packed [`Color565`] data. These functions are designed to work with
//! registers containing multiple color values, offering high-performance batch processing.
//!
//! # Available Functions
//!
//! ## AVX512 Functions (requires `nightly` feature)
//!
//! - [`avx512::recorrelate_ycocg_r_variant1_avx512`] - Applies YCoCg-R variant 1 recorrelation
//! - [`avx512::recorrelate_ycocg_r_variant2_avx512`] - Applies YCoCg-R variant 2 recorrelation  
//! - [`avx512::recorrelate_ycocg_r_variant3_avx512`] - Applies YCoCg-R variant 3 recorrelation
//!
//! Each function takes a [`__m512i`] register containing 32 [`Color565`] values (packed as 16 u32 pairs)
//! and returns a register with the colors recorrelated using the respective YCoCg-R variant.
//!
//! [`Color565`]: crate::color_565::Color565
//! [`__m512i`]: core::arch::x86_64::__m512i

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;
