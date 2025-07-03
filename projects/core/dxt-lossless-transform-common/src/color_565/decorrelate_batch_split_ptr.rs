//! Module for applying YCoCg-R color (de/re)correlation on arrays of [`Color565`] with split input pointers.
//!
//! This module contains functions identical to those in [`decorrelate_batch_ptr`], but these functions
//! have separate `src_ptr_0` and `src_ptr_1` parameters for the colour endpoints, allowing for more
//! flexible memory layout optimization in certain use cases.
//!
//! These functions include SIMD optimizations by generating multiple versions of the code
//! through the [`mod@multiversion`] crate.
//!
//! For a more ergonomic interface that accepts slices instead of raw pointers,
//! see the slice-based wrappers in [`decorrelate_batch_split_slice`].
//!
//! ## Split Pointer Design
//!
//! Unlike the regular pointer functions that take data from a single source, these functions
//! read from two separate source arrays and interleave the results. This pattern is useful
//! when color endpoint data is stored in separate memory regions.
//!
//! The interleaving pattern is:
//! ```ignore
//! dst[0] = transform(src_ptr_0[0]),
//! dst[1] = transform(src_ptr_1[0]),
//! dst[2] = transform(src_ptr_0[1]),
//! dst[3] = transform(src_ptr_1[1]),
//! // etc.
//! ```
//!
//! ## Safety
//!
//! All functions in this module are unsafe as they work with raw pointers and don't
//! perform bounds checking. Callers must ensure:
//!
//! - All pointers are valid and properly aligned
//! - Source and destination memory regions don't overlap
//! - Both source pointers point to initialized data
//! - `src_ptr_0` and `src_ptr_1` point to arrays with at least `num_items / 2` elements each
//! - `dst_ptr` points to an array with at least `num_items` elements
//! - `num_items` is even (functions process pairs of elements)
//!
//! ## Performance
//!
//! Functions utilize the multiversion crate to provide optimized implementations
//! for different CPU feature sets, including AVX2, AVX-512, and other SIMD instructions
//! where available. The implementations automatically select the best variant
//! based on the target CPU capabilities.
//!
//! For now, only x86 CPUs have been marked with [`mod@multiversion`], as I don't have access to
//! high end hardware of other architectures.
//!
//! [`decorrelate_batch_ptr`]: super::decorrelate_batch_ptr
//! [`decorrelate_batch_split_slice`]: super::decorrelate_batch_split_slice

use super::*;

#[cfg(not(tarpaulin_include))] // These are just innocent wrapper functions that are tested elsewhere indirectly.
impl Color565 {
    /// Raw pointer implementation of the YCoCg-R variant 1 recorrelation with split inputs for maximum performance.
    ///
    /// Takes two separate input raw pointers and reads YCoCg-R encoded colors from both sources,
    /// applies [`Self::recorrelate_ycocg_r_var1`] transformation to convert them back to RGB,
    /// and interleaves the recorrelated RGB results into a single output array.
    ///
    /// It is the raw pointer equivalent of [`Self::recorrelate_ycocg_r_var1_slice_split`].
    ///
    /// i.e. it is
    ///
    /// ```ignore
    /// dst[0] = recorrelate(src_ptr_0[0]),
    /// dst[1] = recorrelate(src_ptr_1[0]),
    /// dst[2] = recorrelate(src_ptr_0[1]),
    /// etc.
    /// ```
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_ptr_0`: Pointer to the first source array of [`Color565`] items to transform
    /// - `src_ptr_1`: Pointer to the second source array of [`Color565`] items to transform
    /// - `dst_ptr`: Pointer to the destination array where interleaved transformed items will be stored
    /// - `num_items`: Total number of [`Color565`] items to write to destination (must be even)
    ///
    /// # Safety
    ///
    /// This function is unsafe because it takes raw pointers and doesn't check bounds.
    /// Caller must ensure that:
    /// - All pointers are properly aligned and valid for their respective operations
    /// - `src_ptr_0` and `src_ptr_1` point to arrays with at least `num_items / 2` elements each
    /// - `dst_ptr` points to an array with at least `num_items` elements
    /// - The memory regions don't overlap
    /// - All source pointers point to initialized data
    /// - `num_items` is even (function processes pairs of elements)
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var1_ptr_split(
        src_ptr_0: *const Self,
        src_ptr_1: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
        debug_assert!(
            num_items.is_multiple_of(2),
            "num_items must be even for split operations"
        );

        #[cfg_attr(
            not(feature = "nightly"), 
            multiversion(targets(
                // avx512 only in nightly.
                // x86-64-v3 without lahfsahf
                "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v2 without lahfsahf
                "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
            ))
        )]
        #[cfg_attr(
            feature = "nightly",
            multiversion(targets(
                // x86-64-v4 without lahfsahf
                "x86_64+avx+avx2+avx512bw+avx512cd+avx512dq+avx512f+avx512vl+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v3 without lahfsahf
                "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v2 without lahfsahf
                "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
            ))
        )]
        unsafe fn recorr(
            src_ptr_0: *const Color565,
            src_ptr_1: *const Color565,
            dst_ptr: *mut Color565,
            num_items: usize,
        ) {
            // hack around Multiversion
            for x in 0..num_items / 2 {
                unsafe {
                    let color_0 = &src_ptr_0.add(x).read_unaligned();
                    let color_1 = &src_ptr_1.add(x).read_unaligned();
                    dst_ptr
                        .add(x * 2)
                        .write_unaligned(color_0.recorrelate_ycocg_r_var1());
                    dst_ptr
                        .add((x * 2) + 1)
                        .write_unaligned(color_1.recorrelate_ycocg_r_var1());
                }
            }
        }

        recorr(src_ptr_0, src_ptr_1, dst_ptr, num_items);
    }

    /// Raw pointer implementation of the YCoCg-R variant 2 recorrelation with split inputs for maximum performance.
    ///
    /// Takes two separate input raw pointers and reads YCoCg-R encoded colors from both sources,
    /// applies [`Self::recorrelate_ycocg_r_var2`] transformation to convert them back to RGB,
    /// and interleaves the recorrelated RGB results into a single output array.
    ///
    /// It is the raw pointer equivalent of [`Self::recorrelate_ycocg_r_var2_slice_split`].
    ///
    /// i.e. it is
    ///
    /// ```ignore
    /// dst[0] = recorrelate(src_ptr_0[0]),
    /// dst[1] = recorrelate(src_ptr_1[0]),
    /// dst[2] = recorrelate(src_ptr_0[1]),
    /// etc.
    /// ```
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_ptr_0`: Pointer to the first source array of [`Color565`] items to transform
    /// - `src_ptr_1`: Pointer to the second source array of [`Color565`] items to transform
    /// - `dst_ptr`: Pointer to the destination array where interleaved transformed items will be stored
    /// - `num_items`: Total number of [`Color565`] items to write to destination (must be even)
    ///
    /// # Safety
    ///
    /// This function is unsafe because it takes raw pointers and doesn't check bounds.
    /// Caller must ensure that:
    /// - All pointers are properly aligned and valid for their respective operations
    /// - `src_ptr_0` and `src_ptr_1` point to arrays with at least `num_items / 2` elements each
    /// - `dst_ptr` points to an array with at least `num_items` elements
    /// - The memory regions don't overlap
    /// - All source pointers point to initialized data
    /// - `num_items` is even (function processes pairs of elements)
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var2_ptr_split(
        src_ptr_0: *const Self,
        src_ptr_1: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
        debug_assert!(
            num_items.is_multiple_of(2),
            "num_items must be even for split operations"
        );

        #[cfg_attr(
            not(feature = "nightly"), 
            multiversion(targets(
                // avx512 only in nightly.
                // x86-64-v3 without lahfsahf
                "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v2 without lahfsahf
                "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
            ))
        )]
        #[cfg_attr(
            feature = "nightly",
            multiversion(targets(
                // x86-64-v4 without lahfsahf
                "x86_64+avx+avx2+avx512bw+avx512cd+avx512dq+avx512f+avx512vl+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v3 without lahfsahf
                "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v2 without lahfsahf
                "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
            ))
        )]
        unsafe fn recorr(
            src_ptr_0: *const Color565,
            src_ptr_1: *const Color565,
            dst_ptr: *mut Color565,
            num_items: usize,
        ) {
            // hack around Multiversion
            for x in 0..num_items / 2 {
                unsafe {
                    let color_0 = &src_ptr_0.add(x).read_unaligned();
                    let color_1 = &src_ptr_1.add(x).read_unaligned();
                    dst_ptr
                        .add(x * 2)
                        .write_unaligned(color_0.recorrelate_ycocg_r_var2());
                    dst_ptr
                        .add((x * 2) + 1)
                        .write_unaligned(color_1.recorrelate_ycocg_r_var2());
                }
            }
        }

        recorr(src_ptr_0, src_ptr_1, dst_ptr, num_items);
    }

    /// Raw pointer implementation of the YCoCg-R variant 3 recorrelation with split inputs for maximum performance.
    ///
    /// Takes two separate input raw pointers and reads YCoCg-R encoded colors from both sources,
    /// applies [`Self::recorrelate_ycocg_r_var3`] transformation to convert them back to RGB,
    /// and interleaves the recorrelated RGB results into a single output array.
    ///
    /// It is the raw pointer equivalent of [`Self::recorrelate_ycocg_r_var3_slice_split`].
    ///
    /// i.e. it is
    ///
    /// ```ignore
    /// dst[0] = recorrelate(src_ptr_0[0]),
    /// dst[1] = recorrelate(src_ptr_1[0]),
    /// dst[2] = recorrelate(src_ptr_0[1]),
    /// etc.
    /// ```
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_ptr_0`: Pointer to the first source array of [`Color565`] items to transform
    /// - `src_ptr_1`: Pointer to the second source array of [`Color565`] items to transform
    /// - `dst_ptr`: Pointer to the destination array where interleaved transformed items will be stored
    /// - `num_items`: Total number of [`Color565`] items to write to destination (must be even)
    ///
    /// # Safety
    ///
    /// This function is unsafe because it takes raw pointers and doesn't check bounds.
    /// Caller must ensure that:
    /// - All pointers are properly aligned and valid for their respective operations
    /// - `src_ptr_0` and `src_ptr_1` point to arrays with at least `num_items / 2` elements each
    /// - `dst_ptr` points to an array with at least `num_items` elements
    /// - The memory regions don't overlap
    /// - All source pointers point to initialized data
    /// - `num_items` is even (function processes pairs of elements)
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var3_ptr_split(
        src_ptr_0: *const Self,
        src_ptr_1: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
        debug_assert!(
            num_items.is_multiple_of(2),
            "num_items must be even for split operations"
        );

        #[cfg_attr(
            not(feature = "nightly"), 
            multiversion(targets(
                // avx512 only in nightly.
                // x86-64-v3 without lahfsahf
                "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v2 without lahfsahf
                "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
            ))
        )]
        #[cfg_attr(
            feature = "nightly",
            multiversion(targets(
                // x86-64-v4 without lahfsahf
                "x86_64+avx+avx2+avx512bw+avx512cd+avx512dq+avx512f+avx512vl+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v3 without lahfsahf
                "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
                // x86-64-v2 without lahfsahf
                "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
            ))
        )]
        unsafe fn recorr(
            src_ptr_0: *const Color565,
            src_ptr_1: *const Color565,
            dst_ptr: *mut Color565,
            num_items: usize,
        ) {
            // hack around Multiversion
            for x in 0..num_items / 2 {
                unsafe {
                    let color_0 = &src_ptr_0.add(x).read_unaligned();
                    let color_1 = &src_ptr_1.add(x).read_unaligned();
                    dst_ptr
                        .add(x * 2)
                        .write_unaligned(color_0.recorrelate_ycocg_r_var3());
                    dst_ptr
                        .add((x * 2) + 1)
                        .write_unaligned(color_1.recorrelate_ycocg_r_var3());
                }
            }
        }

        recorr(src_ptr_0, src_ptr_1, dst_ptr, num_items);
    }

    /// Raw pointer implementation for applying the specified recorrelation variant to a block of
    /// colors with split inputs.
    ///
    /// Takes two separate input raw pointers, applies the transformation to colors from both sources,
    /// and interleaves the results into a single output array.
    ///
    /// It is the raw pointer equivalent of [`Self::recorrelate_ycocg_r_slice_split`].
    ///
    /// # Parameters
    ///
    /// - `src_ptr_0`: Pointer to the first source array of [`Color565`] items to transform
    /// - `src_ptr_1`: Pointer to the second source array of [`Color565`] items to transform
    /// - `dst_ptr`: Pointer to the destination array where interleaved transformed items will be stored
    /// - `num_items`: Total number of [`Color565`] items to write to destination (must be even)
    /// - `variant`: The [`YCoCgVariant`] to use for the transformation
    ///
    /// # Safety
    ///
    /// This function is unsafe because it takes raw pointers and doesn't check bounds.
    /// Caller must ensure that:
    /// - All pointers are properly aligned and valid for their respective operations
    /// - `src_ptr_0` and `src_ptr_1` point to arrays with at least `num_items / 2` elements each
    /// - `dst_ptr` points to an array with at least `num_items` elements
    /// - The memory regions don't overlap
    /// - All source pointers point to initialized data
    /// - `num_items` is even (function processes pairs of elements)
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_ptr_split(
        src_ptr_0: *const Self,
        src_ptr_1: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
        variant: YCoCgVariant,
    ) {
        match variant {
            YCoCgVariant::Variant1 => {
                Self::recorrelate_ycocg_r_var1_ptr_split(src_ptr_0, src_ptr_1, dst_ptr, num_items)
            }
            YCoCgVariant::Variant2 => {
                Self::recorrelate_ycocg_r_var2_ptr_split(src_ptr_0, src_ptr_1, dst_ptr, num_items)
            }
            YCoCgVariant::Variant3 => {
                Self::recorrelate_ycocg_r_var3_ptr_split(src_ptr_0, src_ptr_1, dst_ptr, num_items)
            }
            YCoCgVariant::None => {
                // For None variant, we just interleave without transformation
                if num_items > 0 {
                    debug_assert!(
                        num_items.is_multiple_of(2),
                        "num_items must be even for split operations"
                    );

                    for x in 0..num_items / 2 {
                        let color_0 = *src_ptr_0.add(x);
                        let color_1 = *src_ptr_1.add(x);
                        *dst_ptr.add(x * 2) = color_0;
                        *dst_ptr.add((x * 2) + 1) = color_1;
                    }
                }
            }
        }
    }
}
