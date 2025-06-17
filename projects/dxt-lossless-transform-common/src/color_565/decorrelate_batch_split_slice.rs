use super::*;

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
    /// The output slice must be at least as large as the combined length of both input slices.
    /// Both input slices must have the same length.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_0`: First source slice of [`Color565`] items to transform
    /// - `src_1`: Second source slice of [`Color565`] items to transform  
    /// - `dst`: Destination slice where interleaved transformed items will be stored
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The input slices have different lengths
    /// - The destination slice is smaller than the combined length of both input slices
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn recorrelate_ycocg_r_var1_slice_split(src_0: &[Self], src_1: &[Self], dst: &mut [Self]) {
        assert_eq!(
            src_0.len(),
            src_1.len(),
            "Both source slices must have the same length"
        );
        debug_assert!(
            dst.len() >= src_0.len() + src_1.len(),
            "Destination slice must be at least as large as the combined source slices"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::recorrelate_ycocg_r_var1_ptr_split(
                src_0.as_ptr(),
                src_1.as_ptr(),
                dst.as_mut_ptr(),
                src_0.len() + src_1.len(),
            );
        }
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var2`] to elements from two input slices,
    /// interleaving the recorrelated results into a single output slice.
    ///
    /// ``` ignore
    /// dst[0] = recorrelate(src_0[0]),
    /// dst[1] = recorrelate(src_1[0]),
    /// dst[2] = recorrelate(src_0[1]),
    /// dst[3] = recorrelate(src_1[1])
    /// etc.
    /// ```
    ///
    /// The output slice must be at least as large as the combined length of both input slices.
    /// Both input slices must have the same length.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_0`: First source slice of [`Color565`] items to transform
    /// - `src_1`: Second source slice of [`Color565`] items to transform  
    /// - `dst`: Destination slice where interleaved transformed items will be stored
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The input slices have different lengths
    /// - The destination slice is smaller than the combined length of both input slices
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn recorrelate_ycocg_r_var2_slice_split(src_0: &[Self], src_1: &[Self], dst: &mut [Self]) {
        assert_eq!(
            src_0.len(),
            src_1.len(),
            "Both source slices must have the same length"
        );
        debug_assert!(
            dst.len() >= src_0.len() + src_1.len(),
            "Destination slice must be at least as large as the combined source slices"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::recorrelate_ycocg_r_var2_ptr_split(
                src_0.as_ptr(),
                src_1.as_ptr(),
                dst.as_mut_ptr(),
                src_0.len() + src_1.len(),
            );
        }
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
    /// The output slice must be at least as large as the combined length of both input slices.
    /// Both input slices must have the same length.
    ///
    /// May introduce unrolling optimizations. Refer to the original function for details.
    ///
    /// # Parameters
    ///
    /// - `src_0`: First source slice of [`Color565`] items to transform
    /// - `src_1`: Second source slice of [`Color565`] items to transform  
    /// - `dst`: Destination slice where interleaved transformed items will be stored
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The input slices have different lengths
    /// - The destination slice is smaller than the combined length of both input slices
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn recorrelate_ycocg_r_var3_slice_split(src_0: &[Self], src_1: &[Self], dst: &mut [Self]) {
        assert_eq!(
            src_0.len(),
            src_1.len(),
            "Both source slices must have the same length"
        );
        debug_assert!(
            dst.len() >= src_0.len() + src_1.len(),
            "Destination slice must be at least as large as the combined source slices"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::recorrelate_ycocg_r_var3_ptr_split(
                src_0.as_ptr(),
                src_1.as_ptr(),
                dst.as_mut_ptr(),
                src_0.len() + src_1.len(),
            );
        }
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
    /// Color565::decorrelate_ycocg_r_slice(&decorrelated_0, &mut transformed_0, YCoCgVariant::Variant1);
    /// Color565::decorrelate_ycocg_r_slice(&decorrelated_1, &mut transformed_1, YCoCgVariant::Variant1);
    ///
    /// // Then transform back to RGB and interleave results
    /// let mut recorrelated = [Color565::from_raw(0); 4]; // Need room for both input slices
    /// Color565::recorrelate_ycocg_r_slice_split(&transformed_0, &transformed_1, &mut recorrelated, YCoCgVariant::Variant1);
    /// ```
    #[inline]
    #[cfg(not(tarpaulin_include))]
    pub fn recorrelate_ycocg_r_slice_split(
        src_0: &[Self],
        src_1: &[Self],
        dst: &mut [Self],
        variant: YCoCgVariant,
    ) {
        debug_assert!(
            src_0.len() == src_1.len(),
            "Source slices must have the same length"
        );
        debug_assert!(
            dst.len() >= src_0.len() * 2,
            "Destination slice must be at least twice as large as each source slice"
        );

        // Call the raw pointer implementation
        unsafe {
            Self::recorrelate_ycocg_r_ptr_split(
                src_0.as_ptr(),
                src_1.as_ptr(),
                dst.as_mut_ptr(),
                src_0.len() * 2,
                variant,
            );
        }
    }
}
