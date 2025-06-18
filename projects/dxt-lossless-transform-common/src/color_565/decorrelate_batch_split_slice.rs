//! Module for applying YCoCg-R color (de/re)correlation on slices of [`Color565`] with split input sources.
//!
//! This module provides slice-based wrapper functions for the raw pointer implementations
//! in [`decorrelate_batch_split_ptr`]. These functions offer a more ergonomic interface
//! compared to raw pointers, but are still unsafe for maximum performance.
//!
//! ## Split Slice Design
//!
//! Unlike regular slice functions that process data from a single source slice,
//! these functions take two separate input slices and interleave their processed
//! results into a single output slice. This pattern is useful when color endpoint
//! data is stored in separate memory regions or when you need to combine results
//! from two different sources.
//!
//! The interleaving pattern is:
//! ```ignore
//! dst[0] = transform(src_0[0]),
//! dst[1] = transform(src_1[0]),
//! dst[2] = transform(src_0[1]),
//! dst[3] = transform(src_1[1]),
//! // etc.
//! ```
//!
//! ## Safety
//!
//! ***These functions are still unsafe*** and require careful usage. While they accept
//! slice parameters instead of raw pointers, they only perform debug assertions on
//! slice sizes - no runtime bounds checking is performed in release builds.
//!
//! ## Performance
//!
//! These wrapper functions maintain the same performance characteristics as their
//! raw pointer counterparts, including SIMD optimizations through the [`multiversion`]
//! crate. The overhead of the wrapper layer is minimal and typically optimized away
//! by the compiler.
//!
//! Functions may introduce unrolling optimizations. For detailed performance
//! characteristics, refer to the underlying raw pointer implementations in
//! [`decorrelate_batch_split_ptr`].
//!
//! [`decorrelate_batch_split_ptr`]: super::decorrelate_batch_split_ptr

use super::*;

#[cfg(not(tarpaulin_include))] // These are just innocent wrapper functions that are tested elsewhere indirectly.
impl Color565 {
    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var1`] to elements from two input slices,
    /// interleaving the recorrelated results into a single output slice.
    ///
    /// ```ignore
    /// dst[0] = recorrelate(src_0[0]),
    /// dst[1] = recorrelate(src_1[0]),
    /// dst[2] = recorrelate(src_0[1]),
    /// dst[3] = recorrelate(src_1[1])
    /// etc.
    /// ```
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_0`: First source slice of [`Color565`] items to transform
    /// - `src_1`: Second source slice of [`Color565`] items to transform  
    /// - `dst`: Destination slice where interleaved transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - Both input slices have the same length
    /// - The destination slice is at least as large as the combined length of both input slices
    /// - All slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// Debug assertions check slice sizes, but these are not present in release builds.
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var1_slice_split(src_0: &[Self], src_1: &[Self], dst: &mut [Self]) {
        debug_assert_eq!(
            src_0.len(),
            src_1.len(),
            "Both source slices must have the same length"
        );
        debug_assert!(
            dst.len() >= src_0.len() + src_1.len(),
            "Destination slice must be at least as large as the combined source slices"
        );

        // Call the raw pointer implementation
        Self::recorrelate_ycocg_r_var1_ptr_split(
            src_0.as_ptr(),
            src_1.as_ptr(),
            dst.as_mut_ptr(),
            src_0.len() + src_1.len(),
        );
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var2`] to elements from two input slices,
    /// interleaving the recorrelated results into a single output slice.
    ///
    /// ```ignore
    /// dst[0] = recorrelate(src_0[0]),
    /// dst[1] = recorrelate(src_1[0]),
    /// dst[2] = recorrelate(src_0[1]),
    /// dst[3] = recorrelate(src_1[1])
    /// etc.
    /// ```
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_0`: First source slice of [`Color565`] items to transform
    /// - `src_1`: Second source slice of [`Color565`] items to transform  
    /// - `dst`: Destination slice where interleaved transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - Both input slices have the same length
    /// - The destination slice is at least as large as the combined length of both input slices
    /// - All slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// Debug assertions check slice sizes, but these are not present in release builds.
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var2_slice_split(src_0: &[Self], src_1: &[Self], dst: &mut [Self]) {
        debug_assert_eq!(
            src_0.len(),
            src_1.len(),
            "Both source slices must have the same length"
        );
        debug_assert!(
            dst.len() >= src_0.len() + src_1.len(),
            "Destination slice must be at least as large as the combined source slices"
        );

        // Call the raw pointer implementation
        Self::recorrelate_ycocg_r_var2_ptr_split(
            src_0.as_ptr(),
            src_1.as_ptr(),
            dst.as_mut_ptr(),
            src_0.len() + src_1.len(),
        );
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var3`] to elements from two input slices,
    /// interleaving the recorrelated results into a single output slice.
    ///
    /// ```ignore
    /// dst[0] = recorrelate(src_0[0]),
    /// dst[1] = recorrelate(src_1[0]),
    /// dst[2] = recorrelate(src_0[1]),
    /// dst[3] = recorrelate(src_1[1])
    /// etc.
    /// ```
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_0`: First source slice of [`Color565`] items to transform
    /// - `src_1`: Second source slice of [`Color565`] items to transform  
    /// - `dst`: Destination slice where interleaved transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - Both input slices have the same length
    /// - The destination slice is at least as large as the combined length of both input slices
    /// - All slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// Debug assertions check slice sizes, but these are not present in release builds.
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var3_slice_split(src_0: &[Self], src_1: &[Self], dst: &mut [Self]) {
        debug_assert_eq!(
            src_0.len(),
            src_1.len(),
            "Both source slices must have the same length"
        );
        debug_assert!(
            dst.len() >= src_0.len() + src_1.len(),
            "Destination slice must be at least as large as the combined source slices"
        );

        // Call the raw pointer implementation
        Self::recorrelate_ycocg_r_var3_ptr_split(
            src_0.as_ptr(),
            src_1.as_ptr(),
            dst.as_mut_ptr(),
            src_0.len() + src_1.len(),
        );
    }

    /// Applies the specified recorrelation variant to two slices of colors and interleaves the results
    ///
    /// Takes two separate input slices, applies the transformation to colors from both sources,
    /// and interleaves the results into a single output slice.
    ///
    /// # Parameters
    ///
    /// - `src_0`: The first input slice of colors to transform
    /// - `src_1`: The second input slice of colors to transform
    /// - `dst`: The output slice where interleaved transformed colors will be stored
    /// - `variant`: The [`YCoCgVariant`] to use
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - Both input slices have the same length
    /// - The destination slice is at least twice as large as each source slice
    /// - All slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// Debug assertions check slice sizes, but these are not present in release builds.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let decorrelated_0 = [Color565::from_rgb(255, 0, 0), Color565::from_rgb(0, 255, 0)];
    /// let decorrelated_1 = [Color565::from_rgb(0, 0, 255), Color565::from_rgb(255, 255, 0)];
    /// // First transform them to YCoCg-R (separately)
    /// let mut transformed_0 = [Color565::from_raw(0); 2];
    /// let mut transformed_1 = [Color565::from_raw(0); 2];
    /// unsafe {
    ///     Color565::decorrelate_ycocg_r_slice(&decorrelated_0, &mut transformed_0, YCoCgVariant::Variant1);
    ///     Color565::decorrelate_ycocg_r_slice(&decorrelated_1, &mut transformed_1, YCoCgVariant::Variant1);
    ///
    ///     // Then transform back to RGB and interleave results
    ///     let mut recorrelated = [Color565::from_raw(0); 4]; // Need room for both input slices
    ///     Color565::recorrelate_ycocg_r_slice_split(&transformed_0, &transformed_1, &mut recorrelated, YCoCgVariant::Variant1);
    /// }
    /// ```
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_slice_split(
        src_0: &[Self],
        src_1: &[Self],
        dst: &mut [Self],
        variant: YCoCgVariant,
    ) {
        debug_assert_eq!(
            src_0.len(),
            src_1.len(),
            "Source slices must have the same length"
        );
        debug_assert!(
            dst.len() >= src_0.len() * 2,
            "Destination slice must be at least twice as large as each source slice"
        );

        // Call the raw pointer implementation
        Self::recorrelate_ycocg_r_ptr_split(
            src_0.as_ptr(),
            src_1.as_ptr(),
            dst.as_mut_ptr(),
            src_0.len() * 2,
            variant,
        );
    }
}
