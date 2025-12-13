//! Benchmark functions re-exported for external benchmarks.
//!
//! This module re-exposes internal benchmark functions that were changed to `pub(crate)` visibility
//! so that external benchmarks can still access them when the `bench` feature is enabled.
#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]
#![allow(missing_docs)]

pub mod transform {
    //! Transform benchmark functions

    pub mod standard {
        //! Standard transform benchmark functions

        // Wrapper functions for transform benchmark APIs

        pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::u32(input_ptr, output_ptr, len)
        }

        pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::u32_unroll_2(input_ptr, output_ptr, len)
        }

        pub unsafe fn u32_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::u32_unroll_4(input_ptr, output_ptr, len)
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx2_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::avx2_shuffle(input_ptr, output_ptr, len)
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn sse2_shuffle_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::sse2_shuffle_v2(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(target_arch = "x86_64")]
        pub unsafe fn sse2_shuffle_v3(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::sse2_shuffle_v3(
                input_ptr, output_ptr, len,
            )
        }

        // Additional SSE2 functions that were removed from benchmarks
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn shuffle_v1(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::shuffle_v1(input_ptr, output_ptr, len)
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn shuffle_v1_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::shuffle_v1_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        // AVX512 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn permute_512_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::permute_512_v2(input_ptr, output_ptr, len)
        }

        // Additional AVX512 function that was removed from benchmarks
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn permute_512(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::transform::bench::permute_512(input_ptr, output_ptr, len)
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

        // SSE2 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn sse2_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::sse2_shuffle(input_ptr, output_ptr, len)
        }

        // AVX2 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx2_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::avx2_shuffle(input_ptr, output_ptr, len)
        }

        // AVX512 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx512_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::avx512_shuffle(
                input_ptr, output_ptr, len,
            )
        }
    }
}
