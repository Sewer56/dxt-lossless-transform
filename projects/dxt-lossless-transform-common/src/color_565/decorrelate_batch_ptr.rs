use super::*;
use core::ptr::copy_nonoverlapping;
use multiversion::multiversion;

#[cfg(not(tarpaulin_include))] // These are just innocent wrapper functions not worth testing.
impl Color565 {
    /// Raw pointer implementation of the YCoCg-R variant 1 decorrelation for maximum performance.
    ///
    /// Takes input and output raw pointers, applying the transformation while copying `num_items` elements.
    /// It is the raw pointer equivalent of [`Self::decorrelate_ycocg_r_var1_slice`].
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
    pub unsafe fn decorrelate_ycocg_r_var1_ptr(
        src_ptr: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
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
        unsafe fn decorr(src_ptr: *const Color565, dst_ptr: *mut Color565, num_items: usize) {
            // hack around Multiversion
            for x in 0..num_items {
                unsafe {
                    let color = &*src_ptr.add(x);
                    *dst_ptr.add(x) = color.decorrelate_ycocg_r_var1();
                }
            }
        }

        decorr(src_ptr, dst_ptr, num_items);
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var1`] to each element at a pointer.
    ///
    /// Takes input and output raw pointers, applying the transformation while copying `num_items` elements.
    /// It is the raw pointer equivalent of [`Self::recorrelate_ycocg_r_var1_slice`].
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
    pub unsafe fn recorrelate_ycocg_r_var1_ptr(
        src_ptr: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
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
        unsafe fn recorr(src_ptr: *const Color565, dst_ptr: *mut Color565, num_items: usize) {
            // hack around Multiversion
            for x in 0..num_items {
                unsafe {
                    let color = &*src_ptr.add(x);
                    *dst_ptr.add(x) = color.recorrelate_ycocg_r_var1();
                }
            }
        }

        recorr(src_ptr, dst_ptr, num_items);
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var2`] to each element at a pointer.
    ///
    /// Takes input and output raw pointers, applying the transformation while copying `num_items` elements.
    /// It is the raw pointer equivalent of [`Self::recorrelate_ycocg_r_var2_slice`].
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
    pub unsafe fn decorrelate_ycocg_r_var2_ptr(
        src_ptr: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
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
        unsafe fn decorr(src_ptr: *const Color565, dst_ptr: *mut Color565, num_items: usize) {
            // hack around Multiversion
            for x in 0..num_items {
                unsafe {
                    let color = &*src_ptr.add(x);
                    *dst_ptr.add(x) = color.decorrelate_ycocg_r_var2();
                }
            }
        }

        decorr(src_ptr, dst_ptr, num_items);
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var2`] to each element at a pointer.
    ///
    /// Takes an input pointer and an output pointer, applying the transformation while copying `num_items` elements.
    /// It is the raw pointer equivalent of [`Self::recorrelate_ycocg_r_var2_slice`].
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
    pub unsafe fn recorrelate_ycocg_r_var2_ptr(
        src_ptr: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
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
        unsafe fn recorr(src_ptr: *const Color565, dst_ptr: *mut Color565, num_items: usize) {
            // hack around Multiversion
            for x in 0..num_items {
                unsafe {
                    let color = &*src_ptr.add(x);
                    *dst_ptr.add(x) = color.recorrelate_ycocg_r_var2();
                }
            }
        }

        recorr(src_ptr, dst_ptr, num_items);
    }

    /// Convenience function that applies [`Self::decorrelate_ycocg_r_var3`] to each element at a pointer.
    ///
    /// Takes an input pointer and an output pointer, applying the transformation while copying `num_items` elements.
    /// It is the raw pointer equivalent of [`Self::decorrelate_ycocg_r_var3_slice`].
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
    pub unsafe fn decorrelate_ycocg_r_var3_ptr(
        src_ptr: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
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
        unsafe fn decorr(src_ptr: *const Color565, dst_ptr: *mut Color565, num_items: usize) {
            // hack around Multiversion
            for x in 0..num_items {
                unsafe {
                    let color = &*src_ptr.add(x);
                    *dst_ptr.add(x) = color.decorrelate_ycocg_r_var3();
                }
            }
        }

        decorr(src_ptr, dst_ptr, num_items);
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var3`] to each element at a pointer.
    ///
    /// Takes input and output raw pointers, applying the transformation while copying `num_items` elements.
    /// It is the raw pointer equivalent of [`Self::recorrelate_ycocg_r_var3_slice`].
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
    pub unsafe fn recorrelate_ycocg_r_var3_ptr(
        src_ptr: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
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
        unsafe fn recorr(src_ptr: *const Color565, dst_ptr: *mut Color565, num_items: usize) {
            // hack around Multiversion
            for x in 0..num_items {
                unsafe {
                    let color = &*src_ptr.add(x);
                    *dst_ptr.add(x) = color.recorrelate_ycocg_r_var3();
                }
            }
        }

        recorr(src_ptr, dst_ptr, num_items);
    }

    /// Raw pointer implementation for applying the specified decorrelation variant to a block of colors
    ///
    /// # Parameters
    ///
    /// - `src_ptr`: Pointer to the source array of [`Color565`] items to transform
    /// - `dst_ptr`: Pointer to the destination array where transformed items will be stored
    /// - `num_items`: Number of [`Color565`] items to process (not bytes)
    /// - `variant`: The [`YCoCgVariant`] to use for the transformation
    ///
    /// # Safety
    ///
    /// This function is unsafe because it takes raw pointers and doesn't check bounds.
    /// Caller must ensure that:
    /// - Both pointers are properly aligned and valid for reads/writes for at least `num_items` elements
    /// - The memory regions don't overlap
    /// - `src_ptr` points to initialized data
    #[inline]
    pub unsafe fn decorrelate_ycocg_r_ptr(
        src_ptr: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
        variant: YCoCgVariant,
    ) {
        match variant {
            YCoCgVariant::Variant1 => {
                Self::decorrelate_ycocg_r_var1_ptr(src_ptr, dst_ptr, num_items)
            }
            YCoCgVariant::Variant2 => {
                Self::decorrelate_ycocg_r_var2_ptr(src_ptr, dst_ptr, num_items)
            }
            YCoCgVariant::Variant3 => {
                Self::decorrelate_ycocg_r_var3_ptr(src_ptr, dst_ptr, num_items)
            }
            YCoCgVariant::None => {
                // Just copy without transformation if len > 0
                if num_items > 0 && src_ptr != dst_ptr {
                    copy_nonoverlapping(src_ptr, dst_ptr, num_items);
                }
            }
        }
    }

    /// Raw pointer implementation for applying the specified recorrelation variant to a block of colors
    ///
    /// # Parameters
    ///
    /// - `src_ptr`: Pointer to the source array of [`Color565`] items to transform
    /// - `dst_ptr`: Pointer to the destination array where transformed items will be stored
    /// - `num_items`: Number of [`Color565`] items to process (not bytes)
    /// - `variant`: The [`YCoCgVariant`] to use for the transformation
    ///
    /// # Safety
    ///
    /// This function is unsafe because it takes raw pointers and doesn't check bounds.
    /// Caller must ensure that:
    /// - Both pointers are properly aligned and valid for reads/writes for at least `num_items` elements
    /// - The memory regions don't overlap
    /// - `src_ptr` points to initialized data
    #[inline]
    pub unsafe fn recorrelate_ycocg_r_ptr(
        src_ptr: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
        variant: YCoCgVariant,
    ) {
        match variant {
            YCoCgVariant::Variant1 => {
                Self::recorrelate_ycocg_r_var1_ptr(src_ptr, dst_ptr, num_items)
            }
            YCoCgVariant::Variant2 => {
                Self::recorrelate_ycocg_r_var2_ptr(src_ptr, dst_ptr, num_items)
            }
            YCoCgVariant::Variant3 => {
                Self::recorrelate_ycocg_r_var3_ptr(src_ptr, dst_ptr, num_items)
            }
            YCoCgVariant::None => {
                // Just copy without transformation if len > 0
                if num_items > 0 && src_ptr != dst_ptr {
                    copy_nonoverlapping(src_ptr, dst_ptr, num_items);
                }
            }
        }
    }
}
