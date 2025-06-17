use super::*;

impl Color565 {
    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var1`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    /// The output slice must be at least as large as the input slice.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_ptr`: Pointer to the source array of [`Color565`] items to transform
    /// - `dst_ptr`: Pointer to the destination array where transformed items will be stored
    /// - `num_items`: Number of [`Color565`] items to process (not bytes)
    ///
    /// # Safety
    ///
    /// This function is unsafe because it takes raw pointers and doesn't check bounds.
    /// Caller must ensure that:
    /// - Both pointers are properly aligned and valid for reads/writes for at least `num_items` elements
    /// - The memory regions don't overlap
    /// - `src_ptr` points to initialized data
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn decorrelate_ycocg_r_var1_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::decorrelate_ycocg_r_var1_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }
    }

    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var1`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    /// The output slice must be at least as large as the input slice.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn recorrelate_ycocg_r_var1_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::recorrelate_ycocg_r_var1_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }
    }

    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var2`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    /// The output slice must be at least as large as the input slice.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn decorrelate_ycocg_r_var2_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::decorrelate_ycocg_r_var2_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }
    }

    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var2`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    /// The output slice must be at least as large as the input slice.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn recorrelate_ycocg_r_var2_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::recorrelate_ycocg_r_var2_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }
    }

    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var3`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    /// The output slice must be at least as large as the input slice.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn decorrelate_ycocg_r_var3_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::decorrelate_ycocg_r_var3_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }
    }

    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var3`] to each element in a slice.
    ///
    /// Takes an input slice and an output slice, applying the transformation while copying.
    /// The output slice must be at least as large as the input slice.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn recorrelate_ycocg_r_var3_slice(src: &[Self], dst: &mut [Self]) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::recorrelate_ycocg_r_var3_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len());
        }
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
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let colors = [Color565::from_rgb(255, 0, 0), Color565::from_rgb(0, 255, 0)];
    /// let mut transformed = [Color565::from_raw(0); 2];
    /// Color565::decorrelate_ycocg_r_slice(&colors, &mut transformed, YCoCgVariant::Variant1);
    /// ```
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn decorrelate_ycocg_r_slice(src: &[Self], dst: &mut [Self], variant: YCoCgVariant) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::decorrelate_ycocg_r_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len(), variant);
        }
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
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let decorrelated = [Color565::from_rgb(255, 0, 0), Color565::from_rgb(0, 255, 0)];
    /// // First transform them to YCoCg-R
    /// let mut transformed = [Color565::from_raw(0); 2];
    /// Color565::decorrelate_ycocg_r_slice(&decorrelated, &mut transformed, YCoCgVariant::Variant1);
    ///
    /// // Then transform back to RGB
    /// let mut recorrelated = [Color565::from_raw(0); 2];
    /// Color565::recorrelate_ycocg_r_slice(&transformed, &mut recorrelated, YCoCgVariant::Variant1);
    /// ```
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn recorrelate_ycocg_r_slice(src: &[Self], dst: &mut [Self], variant: YCoCgVariant) {
        debug_assert!(
            dst.len() >= src.len(),
            "Destination slice must be at least as large as source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::recorrelate_ycocg_r_ptr(src.as_ptr(), dst.as_mut_ptr(), src.len(), variant);
        }
    }
}
