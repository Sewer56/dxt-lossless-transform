//! Helpers for CPU feature detection without using std.

#[inline]
#[cfg(feature = "nightly")]
pub fn has_avx512f() -> bool {
    cpufeatures::new!(cpuid_avx512, "avx512f");
    cpuid_avx512::get()
}

#[inline]
pub fn has_avx2() -> bool {
    cpufeatures::new!(cpuid_avx2, "avx2");
    cpuid_avx2::get()
}

#[inline]
pub fn has_sse2() -> bool {
    cpufeatures::new!(cpuid_sse2, "sse2");
    cpuid_sse2::get()
}
