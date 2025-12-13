//! Benchmark functions re-exported for external benchmarks.
//!
//! This module re-exposes internal benchmark functions that were changed to `pub(crate)` visibility
//! so that external benchmarks can still access them when the `bench` feature is enabled.
#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]
#![allow(missing_docs)]
#![allow(dead_code)]

pub mod transform {
    //! Transform benchmark functions

    pub mod standard {
        //! Standard transform benchmark functions

        // Wrapper functions for transform benchmark APIs

        pub unsafe fn u32_transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::u32_transform(input_ptr, output_ptr, len)
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn u32_avx2_transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::u32_avx2_transform(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::portable32::u32_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx512_vbmi_transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::avx512_vbmi_transform(
                input_ptr, output_ptr, len,
            )
        }
    }
}

pub mod untransform {
    //! Untransform benchmark functions

    pub mod standard {
        //! Standard untransform benchmark functions

        // Wrapper functions for untransform benchmark APIs

        pub unsafe fn u32_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::u32_untransform(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn u32_untransform_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::u32_untransform_v2(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn u64_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::u64_untransform(
                input_ptr, output_ptr, len,
            )
        }

        // SSE2 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn u32_untransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::u32_untransform_sse2(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn u64_untransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::u64_untransform_sse2(
                input_ptr, output_ptr, len,
            )
        }

        // AVX512 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx512_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::avx512_untransform(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx512_untransform_32_vbmi(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transform::standard::untransform::bench::avx512_untransform_32_vbmi(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx512_untransform_32_vl(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transform::standard::untransform::bench::avx512_untransform_32_vl(
                input_ptr, output_ptr, len,
            )
        }
    }
}
