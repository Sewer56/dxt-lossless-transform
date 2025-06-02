//! YCoCg-R decorrelation intrinsics for [`Color565`] values

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;
