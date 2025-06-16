//! Benchmark functions re-exported for external benchmarks.
//!
//! This module re-exposes internal benchmark functions that were changed to `pub(crate)` visibility
//! so that external benchmarks can still access them when the `bench` feature is enabled.
#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]

pub mod transform {
    //! Transform benchmark functions

    pub mod standard {
        //! Standard transform benchmark functions

        // Wrapper functions for transform benchmark APIs

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn shufps_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::shufps_unroll_4(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn shuffle_permute_unroll_2(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::transform::bench::shuffle_permute_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::u32(input_ptr, output_ptr, len)
        }

        #[cfg(feature = "nightly")]
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn permute_512(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::permute_512(input_ptr, output_ptr, len)
        }

        // SSE2 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn punpckhqdq_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::sse2::punpckhqdq_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn punpckhqdq_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::sse2::punpckhqdq_unroll_4(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn shufps_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::sse2::shufps_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        // AVX2 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn gather(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::avx2::gather(input_ptr, output_ptr, len)
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn permute(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::avx2::permute(input_ptr, output_ptr, len)
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn permute_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::avx2::permute_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn shuffle_permute(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::avx2::shuffle_permute(
                input_ptr, output_ptr, len,
            )
        }

        // AVX512 functions
        #[cfg(all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")))]
        pub unsafe fn permute_512_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::avx512::permute_512_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")))]
        pub unsafe fn permute_512_unroll_2_with_separate_pointers(
            input_ptr: *const u8,
            colors_ptr: *mut u32,
            indices_ptr: *mut u32,
            len: usize,
        ) {
            crate::transforms::standard::transform::bench::avx512::permute_512_unroll_2_with_separate_pointers(
                input_ptr, colors_ptr, indices_ptr, len
            )
        }

        // Portable32 functions
        pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable32::u32_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn u32_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable32::u32_unroll_4(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn u32_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable32::u32_unroll_8(
                input_ptr, output_ptr, len,
            )
        }

        // Portable64 functions
        pub unsafe fn portable(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable64::portable(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn shift(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable64::shift(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn shift_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable64::shift_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn shift_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable64::shift_unroll_4(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn shift_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable64::shift_unroll_8(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn shift_with_count(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::transform::bench::portable64::shift_with_count(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn shift_with_count_unroll_2(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::transform::bench::portable64::shift_with_count_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn shift_with_count_unroll_4(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::transform::bench::portable64::shift_with_count_unroll_4(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn shift_with_count_unroll_8(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::transform::bench::portable64::shift_with_count_unroll_8(
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

        pub unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::untransform::bench::u32_detransform(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn unpck_detransform_unroll_2(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::unpck_detransform_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn permd_detransform_unroll_2(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::permd_detransform_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")))]
        pub unsafe fn permute_512_detransform_unroll_2(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::permute_512_detransform_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        // SSE2 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn unpck_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::untransform::bench::sse2::unpck_detransform(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(target_arch = "x86_64")]
        pub unsafe fn unpck_detransform_unroll_4(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::sse2::unpck_detransform_unroll_4(
                input_ptr, output_ptr, len,
            )
        }

        // AVX2 functions
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn permd_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transforms::standard::untransform::bench::avx2::permd_detransform(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx2_unpck_detransform(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::avx2::unpck_detransform(
                input_ptr, output_ptr, len,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx2_unpck_detransform_unroll_2(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::avx2::unpck_detransform_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        // AVX512 functions
        #[cfg(all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")))]
        pub unsafe fn permute_512_detransform_unroll_2_intrinsics(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::avx512::permute_512_detransform_unroll_2_intrinsics(input_ptr, output_ptr, len)
        }

        #[cfg(all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")))]
        pub unsafe fn permute_512_detransform_unroll_2_with_components_intrinsics(
            output_ptr: *mut u8,
            len: usize,
            indices_ptr: *const u8,
            colors_ptr: *const u8,
        ) {
            crate::transforms::standard::untransform::bench::avx512::permute_512_detransform_unroll_2_with_components_intrinsics(
                output_ptr, len, indices_ptr, colors_ptr
            )
        }

        // Portable32 functions
        pub unsafe fn u32_detransform_unroll_2(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::portable32::u32_detransform_unroll_2(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn u32_detransform_unroll_4(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::portable32::u32_detransform_unroll_4(
                input_ptr, output_ptr, len,
            )
        }

        pub unsafe fn u32_detransform_unroll_8(
            input_ptr: *const u8,
            output_ptr: *mut u8,
            len: usize,
        ) {
            crate::transforms::standard::untransform::bench::portable32::u32_detransform_unroll_8(
                input_ptr, output_ptr, len,
            )
        }
    }
}
