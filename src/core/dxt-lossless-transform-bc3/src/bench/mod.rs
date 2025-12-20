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

    pub mod with_split_alphas {
        //! Split alphas transform benchmark functions

        pub unsafe fn generic_transform_with_split_alphas(
            input_ptr: *const u8,
            alpha0_out: *mut u8,
            alpha1_out: *mut u8,
            alpha_indices_out: *mut u16,
            colors_out: *mut u32,
            color_indices_out: *mut u32,
            block_count: usize,
        ) {
            crate::transform::with_split_alphas::transform::generic::transform_with_split_alphas(
                input_ptr,
                alpha0_out,
                alpha1_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                block_count,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx2_transform_with_split_alphas(
            input_ptr: *const u8,
            alpha0_out: *mut u8,
            alpha1_out: *mut u8,
            alpha_indices_out: *mut u16,
            colors_out: *mut u32,
            color_indices_out: *mut u32,
            block_count: usize,
        ) {
            crate::transform::with_split_alphas::transform::avx2::transform_with_split_alphas(
                input_ptr,
                alpha0_out,
                alpha1_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                block_count,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn avx512_transform_with_split_alphas(
            input_ptr: *const u8,
            alpha0_out: *mut u8,
            alpha1_out: *mut u8,
            alpha_indices_out: *mut u16,
            colors_out: *mut u32,
            color_indices_out: *mut u32,
            block_count: usize,
        ) {
            crate::transform::with_split_alphas::transform::avx512vbmi::transform_with_split_alphas(
                input_ptr,
                alpha0_out,
                alpha1_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                block_count,
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
        pub unsafe fn avx512_untransform_32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
            crate::transform::standard::untransform::bench::avx512_untransform_32(
                input_ptr, output_ptr, len,
            )
        }
    }

    pub mod with_split_alphas {
        //! Split alphas untransform benchmark functions

        pub unsafe fn generic_untransform_with_split_alphas(
            alpha0_ptr: *const u8,
            alpha1_ptr: *const u8,
            alpha_indices_ptr: *const u16,
            colors_ptr: *const u32,
            color_indices_ptr: *const u32,
            output_ptr: *mut u8,
            block_count: usize,
        ) {
            crate::transform::with_split_alphas::untransform::generic::untransform_with_split_alphas(
                alpha0_ptr,
                alpha1_ptr,
                alpha_indices_ptr,
                colors_ptr,
                color_indices_ptr,
                output_ptr,
                block_count,
            )
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        pub unsafe fn sse2_untransform_with_split_alphas(
            alpha0_ptr: *const u8,
            alpha1_ptr: *const u8,
            alpha_indices_ptr: *const u16,
            colors_ptr: *const u32,
            color_indices_ptr: *const u32,
            output_ptr: *mut u8,
            block_count: usize,
        ) {
            crate::transform::with_split_alphas::untransform::sse2::untransform_with_split_alphas_sse2(
                alpha0_ptr,
                alpha1_ptr,
                alpha_indices_ptr,
                colors_ptr,
                color_indices_ptr,
                output_ptr,
                block_count,
            )
        }
    }
}
