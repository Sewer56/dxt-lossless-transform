//! Helpers for CPU feature detection without using std.
//!
//! This module provides CPU feature detection for SIMD instruction sets using the
//! `cpufeatures` crate. These functions are used to determine at runtime which optimized code paths
//! can be safely executed on the current CPU.
//!
//! The functions are minimal overhead, they have an init that's called once, and every subsequent
//! call simply loads and compares a bool.

/// Checks if the CPU supports AVX512F (AVX-512 Foundation) instructions.
///
/// This function is only available when compiling with the `nightly` feature enabled.
/// AVX-512F provides 512-bit wide vectors and operations which can significantly
/// accelerate DXT compression transformations when available.
///
/// # Returns
/// `true` if the CPU supports AVX512F instructions, `false` otherwise.
#[inline]
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn has_avx512f() -> bool {
    cpufeatures::new!(cpuid_avx512, "avx512f");
    cpuid_avx512::get()
}

/// Checks if the CPU supports AVX512VBMI (AVX-512 Vector Byte Manipulation Instructions) instructions.
///
/// This function is only available when compiling with the `nightly` feature enabled.
/// AVX512VBMI extends AVX-512 with additional byte manipulation instructions, which can
/// be beneficial for certain block formats.
///
/// # Returns
/// `true` if the CPU supports AVX512VBMI instructions, `false` otherwise.
#[inline]
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn has_avx512vbmi() -> bool {
    cpufeatures::new!(cpuid_avx512vbmi, "avx512vbmi");
    cpuid_avx512vbmi::get()
}

/// Checks if the CPU supports AVX512VL (AVX-512 Vector Length) instructions.
///
/// This function is only available when compiling with the `nightly` feature enabled.
/// AVX512VL extends AVX-512 with additional vector length instructions, giving AVX-512
/// instructions to smaller register sizes. This can be beneficial for certain block formats.
///
/// # Returns
/// `true` if the CPU supports AVX512VL instructions, `false` otherwise.
#[inline]
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn has_avx512vl() -> bool {
    cpufeatures::new!(cpuid_avx512vl, "avx512vl");
    cpuid_avx512vl::get()
}

/// Checks if the CPU supports AVX512VL (AVX-512 Vector Length) instructions.
///
/// This function is only available when compiling with the `nightly` feature enabled.
/// AVX512VL extends AVX-512 with additional vector length instructions, giving AVX-512
/// instructions to smaller register sizes. This can be beneficial for certain block formats.
///
/// # Returns
/// `true` if the CPU supports AVX512VL instructions, `false` otherwise.
#[inline]
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn has_avx512bw() -> bool {
    cpufeatures::new!(cpuid_avx512bw, "avx512bw");
    cpuid_avx512bw::get()
}

/// Checks if the CPU supports SSSE3 (Supplemental SSE3) instructions.
///
/// SSSE3 isn't really used anywhere currently; just in some unused routines which ended up
/// being obsoleted.
///
/// # Returns
/// `true` if the CPU supports SSSE3 instructions, `false` otherwise.
#[inline]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn has_ssse3() -> bool {
    cpufeatures::new!(cpuid_ssse3, "ssse3");
    cpuid_ssse3::get()
}

/// Checks if the CPU supports AVX2 (Advanced Vector Extensions 2) instructions.
///
/// AVX2 extends AVX by providing 256-bit integer SIMD instructions.
/// Performance of certain transforms are improved due to more flexibility with output registers
/// and larger register size.
///
/// # Returns
/// `true` if the CPU supports AVX2 instructions, `false` otherwise.
#[inline]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn has_avx2() -> bool {
    cpufeatures::new!(cpuid_avx2, "avx2");
    cpuid_avx2::get()
}

/// Checks if the CPU supports SSE2 (Streaming SIMD Extensions 2) instructions.
///
/// SSE2 is widely available on virtually all x86-64 processors and provides basic
/// SIMD operations that form the baseline implementation for DXT compression transformations.
///
/// # Returns
/// `true` if the CPU supports SSE2 instructions, `false` otherwise.
#[inline]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn has_sse2() -> bool {
    cpufeatures::new!(cpuid_sse2, "sse2");
    cpuid_sse2::get()
}
