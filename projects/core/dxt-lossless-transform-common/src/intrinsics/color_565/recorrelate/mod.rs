//! YCoCg-R recorrelation intrinsics for [`Color565`] values.
//!
//! This module provides SIMD-optimized intrinsic functions for performing YCoCg-R recorrelation
//! transformations on packed [`Color565`] data. These functions are designed to work with
//! registers containing multiple color values, offering high-performance batch processing.
//!
//! # Available Functions
//!
//! ## AVX512BW Functions
//!
//! - [`avx512bw::recorrelate_ycocg_r_var1_avx512bw`] - Applies YCoCg-R variant 1 recorrelation
//! - [`avx512bw::recorrelate_ycocg_r_var2_avx512bw`] - Applies YCoCg-R variant 2 recorrelation
//! - [`avx512bw::recorrelate_ycocg_r_var3_avx512bw`] - Applies YCoCg-R variant 3 recorrelation
//!
//! ## AVX2 Functions
//!
//! - [`avx2::recorrelate_ycocg_r_var1_avx2`] - Applies YCoCg-R variant 1 recorrelation
//! - [`avx2::recorrelate_ycocg_r_var2_avx2`] - Applies YCoCg-R variant 2 recorrelation
//! - [`avx2::recorrelate_ycocg_r_var3_avx2`] - Applies YCoCg-R variant 3 recorrelation
//!
//! ## SSE2 Functions
//!
//! - [`sse2::recorrelate_ycocg_r_var1_sse2`] - Applies YCoCg-R variant 1 recorrelation
//! - [`sse2::recorrelate_ycocg_r_var2_sse2`] - Applies YCoCg-R variant 2 recorrelation
//! - [`sse2::recorrelate_ycocg_r_var3_sse2`] - Applies YCoCg-R variant 3 recorrelation
//!
//! [`Color565`]: crate::color_565::Color565
//! [`__m512i`]: core::arch::x86_64::__m512i

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512bw;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;
