//! Slice-based wrappers for YCoCg-R decorrelation/recorrelation transformations.
//!
//! This module provides convenient slice-based wrapper functions that offer a more
//! ergonomic interface compared to the raw pointer equivalents in
//! [`decorrelate_batch_ptr`]. These functions just call to their raw pointer
//! counterparts, but may in some contexts allow for more convenience.
//!
//! ## Safety
//!
//! ***These functions are still unsafe*** and require careful usage. While they accept
//! slice parameters instead of raw pointers, they only perform debug assertions on
//! slice sizes - no runtime bounds checking is performed in release builds.
//!
//! For maximum performance and lower-level control, see the raw pointer implementations
//! in [`decorrelate_batch_ptr`].
//!
//! [`decorrelate_batch_ptr`]: super::decorrelate_batch_ptr

use super::*;

#[cfg(not(tarpaulin_include))] // These are just innocent wrapper functions not worth testing.
impl Color565 {
    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var1`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src`: Source slice of [`Color565`] items to transform
    /// - `dst`: Destination slice where transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - The destination slice is at least as large as the source slice
    /// - Both slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// A debug assertion checks slice sizes, but this is not present in release builds.
    #[inline]
    pub unsafe fn decorrelate_ycocg_r_var1_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        Self::decorrelate_ycocg_r_var1_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var1`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src`: Source slice of [`Color565`] items to transform
    /// - `dst`: Destination slice where transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - The destination slice is at least as large as the source slice
    /// - Both slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// A debug assertion checks slice sizes, but this is not present in release builds.
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var1_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        Self::recorrelate_ycocg_r_var1_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }

    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var2`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src`: Source slice of [`Color565`] items to transform
    /// - `dst`: Destination slice where transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - The destination slice is at least as large as the source slice
    /// - Both slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// A debug assertion checks slice sizes, but this is not present in release builds.
    #[inline]
    pub unsafe fn decorrelate_ycocg_r_var2_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        Self::decorrelate_ycocg_r_var2_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var2`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src`: Source slice of [`Color565`] items to transform
    /// - `dst`: Destination slice where transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - The destination slice is at least as large as the source slice
    /// - Both slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// A debug assertion checks slice sizes, but this is not present in release builds.
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var2_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        Self::recorrelate_ycocg_r_var2_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }

    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var3`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src`: Source slice of [`Color565`] items to transform
    /// - `dst`: Destination slice where transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - The destination slice is at least as large as the source slice
    /// - Both slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// A debug assertion checks slice sizes, but this is not present in release builds.
    #[inline]
    pub unsafe fn decorrelate_ycocg_r_var3_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        Self::decorrelate_ycocg_r_var3_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var3`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src`: Source slice of [`Color565`] items to transform
    /// - `dst`: Destination slice where transformed items will be stored
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - The destination slice is at least as large as the source slice
    /// - Both slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// A debug assertion checks slice sizes, but this is not present in release builds.
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_var3_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        Self::recorrelate_ycocg_r_var3_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
    }

    /// Applies the specified decorrelation variant to a slice of colors
    ///
    /// Wrapper around the variant-specific decorrelate_slice methods.
    ///
    /// # Parameters
    ///
    /// - `src`: The input slice of colors to transform
    /// - `dst`: The output slice where transformed colors will be stored
    /// - `variant`: The [`YCoCgVariant`] to use
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - The destination slice is at least as large as the source slice
    /// - Both slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// A debug assertion checks slice sizes, but this is not present in release builds.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let colors = [Color565::from_rgb(255, 0, 0), Color565::from_rgb(0, 255, 0)];
    /// let mut transformed = [Color565::from_raw(0); 2];
    /// unsafe {
    ///     Color565::decorrelate_ycocg_r_slice(&colors, &mut transformed, YCoCgVariant::Variant1);
    /// }
    /// ```
    #[inline]
    pub unsafe fn decorrelate_ycocg_r_slice(src: &[Self], dst: &mut [Self], variant: YCoCgVariant) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        Self::decorrelate_ycocg_r_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len(), variant);
    }

    /// Applies the specified recorrelation variant to a slice of colors
    ///
    /// Wrapper around the variant-specific recorrelate_slice methods.
    ///
    /// # Parameters
    ///
    /// - `src`: The input slice of colors to transform
    /// - `dst`: The output slice where transformed colors will be stored
    /// - `variant`: The [`YCoCgVariant`] to use
    ///
    /// # Safety
    ///
    /// This function is unsafe because it doesn't perform runtime bounds checking.
    /// Caller must ensure that:
    /// - The destination slice is at least as large as the source slice
    /// - Both slices contain valid, initialized data
    /// - The memory regions don't overlap
    ///
    /// A debug assertion checks slice sizes, but this is not present in release builds.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let decorrelated = [Color565::from_rgb(255, 0, 0), Color565::from_rgb(0, 255, 0)];
    /// // First transform them to YCoCg-R
    /// let mut transformed = [Color565::from_raw(0); 2];
    /// unsafe {
    ///     Color565::decorrelate_ycocg_r_slice(&decorrelated, &mut transformed, YCoCgVariant::Variant1);
    /// }
    ///
    /// // Then transform back to RGB
    /// let mut recorrelated = [Color565::from_raw(0); 2];
    /// unsafe {
    ///     Color565::recorrelate_ycocg_r_slice(&transformed, &mut recorrelated, YCoCgVariant::Variant1);
    /// }
    /// ```
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_slice(src: &[Self], dst: &mut [Self], variant: YCoCgVariant) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        Self::recorrelate_ycocg_r_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len(), variant);
    }
}
