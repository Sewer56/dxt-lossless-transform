use crate::color_8888::Color8888;
use core::ptr::copy_nonoverlapping;
use derive_enum_all_values::AllValues;
use multiversion::multiversion;

/// Represents a 16-bit RGB565 color (5 bits red, 6 bits green, 5 bits blue)
/// As encountered in many of the BC1 formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color565 {
    /// The underlying 16-bit RGB565 value
    value: u16,
}

impl Color565 {
    /// Creates a new [`Color565`] from the raw 16-bit value
    #[inline]
    pub fn from_raw(value: u16) -> Self {
        Self { value }
    }

    /// Creates a new [`Color565`] from separate RGB components
    ///
    /// # Parameters
    ///
    /// - `r`: The red component (0-255)
    /// - `g`: The green component (0-255)
    /// - `b`: The blue component (0-255)
    #[inline]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        // Implementation matches etcpak's optimized to565 function
        // Source: https://github.com/wolfpld/etcpak/blob/master/ProcessDxtc.cpp
        // This approach calculates the entire value in one expression, potentially allowing
        // better compiler optimizations.
        Self {
            value: ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3),
        }

        // Original implementation (me):
        //   Computes to same thing, but just for compiler's sake, using the above.
        //
        // let r = (r as u16 >> 3) & 0b11111;
        // let g = (g as u16 >> 2) & 0b111111;
        // let b = b as u16 >> 3;
        //
        // Self {
        //     value: (r << 11) | (g << 5) | b,
        // }
        //
    }

    /// Returns the raw 16-bit value
    #[inline]
    pub fn raw_value(&self) -> u16 {
        self.value
    }

    // NOTE: https://fgiesen.wordpress.com/2021/10/04/gpu-bcn-decoding/
    // BC1 as written in the D3D11 functional spec first expands the endpoint values from 5 or 6 bits
    // to 8 bits by replicating the top bits; all three vendors appear to do this or something equivalent,
    // and then convert the result from 8-bit UNorm to float exactly.
    // Thanks ryg!! I know am decoding the colour endpoints right!!

    /// Extracts the expanded 8-bit red component
    #[inline]
    pub fn red(&self) -> u8 {
        let r = (self.value & 0b11111000_00000000) >> 11;
        ((r << 3) | (r >> 2)) as u8
    }

    /// Extracts the expanded 8-bit green component
    #[inline]
    pub fn green(&self) -> u8 {
        let g = (self.value & 0b00000111_11100000) >> 5;
        ((g << 2) | (g >> 4)) as u8
    }

    /// Extracts the expanded 8-bit blue component
    #[inline]
    pub fn blue(&self) -> u8 {
        let b = self.value & 0b00000000_00011111;
        ((b << 3) | (b >> 2)) as u8
    }

    /// Compares two [`Color565`] values
    #[inline]
    pub fn greater_than(&self, other: &Self) -> bool {
        self.value > other.value
    }

    /// Converts this [`Color565`] to a [`Color8888`] with full opacity (alpha=255)
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let rgb565 = Color565::from_rgb(255, 0, 0);
    /// let rgba8888 = rgb565.to_color_8888();
    /// assert_eq!(rgba8888.r, 255);
    /// assert_eq!(rgba8888.g, 0);
    /// assert_eq!(rgba8888.b, 0);
    /// assert_eq!(rgba8888.a, 255);
    /// ```
    pub fn to_color_8888(&self) -> Color8888 {
        Color8888::new(self.red(), self.green(), self.blue(), 255)
    }

    /// Converts this RGB565 color to a RGBA8888 color with the specified alpha value
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let rgb565 = Color565::from_rgb(255, 0, 0);
    /// let rgba8888 = rgb565.to_color_8888_with_alpha(128);
    /// assert_eq!(rgba8888.r, 255);
    /// assert_eq!(rgba8888.g, 0);
    /// assert_eq!(rgba8888.b, 0);
    /// assert_eq!(rgba8888.a, 128);
    /// ```
    pub fn to_color_8888_with_alpha(&self, alpha: u8) -> Color8888 {
        Color8888::new(self.red(), self.green(), self.blue(), alpha)
    }

    /// Transforms RGB color to YCoCg-R (reversible YCoCg) color space.
    ///
    /// YCoCg-R is a lifting-based variation of YCoCg that offers perfect reversibility.
    /// This implementation treats all channels as 5-bit values for consistency.
    ///
    /// The YCoCg-R transformation follows these steps:
    /// 1. Co = R - B
    /// 2. t = B + (Co >> 1)
    /// 3. Cg = G - t
    /// 4. Y = t + (Cg >> 1)
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r_var1();
    /// // Color is now in YCoCg-R form
    /// ```
    #[inline]
    pub fn decorrelate_ycocg_r_var1(&self) -> Self {
        // 0x1F == 0b11111
        // Extract RGB components
        let r = (self.value >> 11) & 0x1F; // 5 bits for red
        let g = (self.value >> 6) & 0x1F; // 5 top bits for green (ignoring bottom 1 bit)
        let g_low = (self.value >> 5) & 0x1; // leftover bit for green
        let b = self.value & 0x1F; // 5 bits for blue

        // Apply YCoCg-R forward transform (all operations in 5-bit space)
        // Step 1: Co = R - B
        let co = (r as i16 - b as i16) & 0x1F;

        // Step 2: t = B + (Co >> 1)
        let t = (b as i16 + (co >> 1)) & 0x1F;

        // Step 3: Cg = G - t
        let cg = (g as i16 - t) & 0x1F;

        // Step 4: Y = t + (Cg >> 1)
        let y = (t + (cg >> 1)) & 0x1F;

        // Pack into Color565 format:
        // - Y (5 bits) in red position
        // - Co (5 bits) in green position (shifted to use upper 5 bits of the 6-bit field)
        // - Cg (5 bits) in blue position
        Color565::from_raw(((y as u16) << 11) | ((co as u16) << 6) | (g_low << 5) | (cg as u16))
    }

    /// [Variant 1: Usually compresses best]
    /// Transforms color from YCoCg-R back to RGB color space.
    ///
    /// This is the inverse of the decorrelation operation, following these steps:
    /// 1. t = Y - (Cg >> 1)
    /// 2. G = Cg + t
    /// 3. B = t - (Co >> 1)
    /// 4. R = B + Co
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let original = Color565::from_rgb(255, 128, 64);
    /// let mut decorrelated = original;
    /// decorrelated.decorrelate_ycocg_r_var1();
    ///
    /// // Now recorrelate it back to original
    /// decorrelated.recorrelate_ycocg_r_var1();
    /// assert_eq!(decorrelated.raw_value(), original.raw_value());
    /// ```
    #[inline]
    pub fn recorrelate_ycocg_r_var1(&self) -> Self {
        // 0x1F == 0b11111
        // Extract YCoCg-R components
        let y = (self.value >> 11) & 0x1F; // 5 bits (Y in red position)
        let co = (self.value >> 6) & 0x1F; // 5 bits (Co in upper 5 bits of green position)
        let g_low = (self.value >> 5) & 0x1; // Extract the preserved low bit of green
        let cg = self.value & 0x1F; // 5 bits (Cg in blue position)

        // Apply YCoCg-R inverse transform (all operations in 5-bit space)
        // Step 1: t = Y - (Cg >> 1)
        let t = (y as i16 - ((cg as i16) >> 1)) & 0x1F;

        // Step 2: G = Cg + t
        let g = (cg as i16 + t) & 0x1F;

        // Step 3: B = t - (Co >> 1)
        let b = (t - ((co as i16) >> 1)) & 0x1F;

        // Step 4: R = B + Co
        let r = (b + co as i16) & 0x1F;

        // Pack back into RGB565 format, preserving the original g_low bit
        Color565::from_raw(((r as u16) << 11) | ((g as u16) << 6) | (g_low << 5) | (b as u16))
    }

    /// [Variant 2: Faster recorrelate (marginally) for compression speed.]
    /// Transforms RGB color to YCoCg-R (reversible YCoCg) color space.
    ///
    /// YCoCg-R is a lifting-based variation of YCoCg that offers perfect reversibility.
    /// This implementation treats all channels as 5-bit values for consistency.
    ///
    /// The YCoCg-R transformation follows these steps:
    /// 1. Co = R - B
    /// 2. t = B + (Co >> 1)
    /// 3. Cg = G - t
    /// 4. Y = t + (Cg >> 1)
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r_var2();
    /// // Color is now in YCoCg-R form
    /// ```
    #[inline]
    pub fn decorrelate_ycocg_r_var2(&self) -> Self {
        // 0x1F == 0b11111
        // Extract RGB components
        let r = (self.value >> 11) & 0x1F; // 5 bits for red
        let g = (self.value >> 6) & 0x1F; // 5 top bits for green (ignoring bottom 1 bit)
        let g_low = (self.value >> 5) & 0x1; // leftover bit for green
        let b = self.value & 0x1F; // 5 bits for blue

        // Apply YCoCg-R forward transform (all operations in 5-bit space)
        // Step 1: Co = R - B
        let co = (r as i16 - b as i16) & 0x1F;

        // Step 2: t = B + (Co >> 1)
        let t = (b as i16 + (co >> 1)) & 0x1F;

        // Step 3: Cg = G - t
        let cg = (g as i16 - t) & 0x1F;

        // Step 4: Y = t + (Cg >> 1)
        let y = (t + (cg >> 1)) & 0x1F;

        // Pack into Color565 format:
        // - Y (5 bits) in red position
        // - Co (5 bits) in green position (shifted to use upper 5 bits of the 6-bit field)
        // - Cg (5 bits) in blue position
        Color565::from_raw((g_low << 15) | ((y as u16) << 10) | ((co as u16) << 5) | (cg as u16))
        // Note: Marginal speed improvement on recorrelate by placing low bit in the top.
    }

    /// [Variant 3: Sometimes better than variant 2.]
    /// Transforms color from YCoCg-R back to RGB color space.
    ///
    /// This is the inverse of the decorrelation operation, following these steps:
    /// 1. t = Y - (Cg >> 1)
    /// 2. G = Cg + t
    /// 3. B = t - (Co >> 1)
    /// 4. R = B + Co
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let original = Color565::from_rgb(255, 128, 64);
    /// let mut decorrelated = original;
    /// decorrelated.decorrelate_ycocg_r_var2();
    ///
    /// // Now recorrelate it back to original
    /// decorrelated.recorrelate_ycocg_r_var2();
    /// assert_eq!(decorrelated.raw_value(), original.raw_value());
    /// ```
    #[inline]
    pub fn recorrelate_ycocg_r_var2(&self) -> Self {
        // 0x1F == 0b11111
        // Extract YCoCg-R components
        let g_low = self.value >> 15; // Extract the preserved low bit of green
        let y = (self.value >> 10) & 0x1F; // 5 bits (Y in red position)
        let co = (self.value >> 5) & 0x1F; // 5 bits (Co in upper 5 bits of green position)
        let cg = self.value & 0x1F; // 5 bits (Cg in blue position)

        // Apply YCoCg-R inverse transform (all operations in 5-bit space)
        // Step 1: t = Y - (Cg >> 1)
        let t = (y as i16 - ((cg as i16) >> 1)) & 0x1F;

        // Step 2: G = Cg + t
        let g = (cg as i16 + t) & 0x1F;

        // Step 3: B = t - (Co >> 1)
        let b = (t - ((co as i16) >> 1)) & 0x1F;

        // Step 4: R = B + Co
        let r = (b + co as i16) & 0x1F;

        // Pack back into RGB565 format, preserving the original g_low bit
        Color565::from_raw(((r as u16) << 11) | ((g as u16) << 6) | (g_low << 5) | (b as u16))
    }

    /// Transforms RGB color to YCoCg-R (reversible YCoCg) color space.
    ///
    /// YCoCg-R is a lifting-based variation of YCoCg that offers perfect reversibility.
    /// This implementation treats all channels as 5-bit values for consistency.
    ///
    /// The YCoCg-R transformation follows these steps:
    /// 1. Co = R - B
    /// 2. t = B + (Co >> 1)
    /// 3. Cg = G - t
    /// 4. Y = t + (Cg >> 1)
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r_var3();
    /// // Color is now in YCoCg-R form
    /// ```
    #[inline]
    pub fn decorrelate_ycocg_r_var3(&self) -> Self {
        // 0x1F == 0b11111
        // Extract RGB components
        let r = (self.value >> 11) & 0x1F; // 5 bits for red
        let g = (self.value >> 6) & 0x1F; // 5 top bits for green (ignoring bottom 1 bit)
        let g_low = (self.value >> 5) & 0x1; // leftover bit for green
        let b = self.value & 0x1F; // 5 bits for blue

        // Apply YCoCg-R forward transform (all operations in 5-bit space)
        // Step 1: Co = R - B
        let co = (r as i16 - b as i16) & 0x1F;

        // Step 2: t = B + (Co >> 1)
        let t = (b as i16 + (co >> 1)) & 0x1F;

        // Step 3: Cg = G - t
        let cg = (g as i16 - t) & 0x1F;

        // Step 4: Y = t + (Cg >> 1)
        let y = (t + (cg >> 1)) & 0x1F;

        // Pack into Color565 format:
        // - Y (5 bits) in red position
        // - Co (5 bits) in green position (shifted to use upper 5 bits of the 6-bit field)
        // - Cg (5 bits) in blue position
        Color565::from_raw(((y as u16) << 11) | ((co as u16) << 6) | ((cg as u16) << 1) | g_low)
    }

    /// Transforms color from YCoCg-R back to RGB color space.
    ///
    /// This is the inverse of the decorrelation operation, following these steps:
    /// 1. t = Y - (Cg >> 1)
    /// 2. G = Cg + t
    /// 3. B = t - (Co >> 1)
    /// 4. R = B + Co
    ///
    /// This variation has the g_low bit placed as the lowermost bit.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let original = Color565::from_rgb(255, 128, 64);
    /// let mut decorrelated = original;
    /// decorrelated.decorrelate_ycocg_r_var3();
    ///
    /// // Now recorrelate it back to original
    /// decorrelated.recorrelate_ycocg_r_var3();
    /// assert_eq!(decorrelated.raw_value(), original.raw_value());
    /// ```
    #[inline]
    pub fn recorrelate_ycocg_r_var3(&self) -> Self {
        // 0x1F == 0b11111
        // Extract YCoCg-R components
        let y = (self.value >> 11) & 0x1F; // 5 bits (Y in red position)
        let co = (self.value >> 6) & 0x1F; // 5 bits (Co in upper 5 bits of green position)
        let cg = (self.value >> 1) & 0x1F; // 5 bits (Cg in bits 1-5)
        let g_low = self.value & 0x1; // Extract the preserved low bit of green (lowermost bit)

        // Apply YCoCg-R inverse transform (all operations in 5-bit space)
        // Step 1: t = Y - (Cg >> 1)
        let t = (y as i16 - ((cg as i16) >> 1)) & 0x1F;

        // Step 2: G = Cg + t
        let g = (cg as i16 + t) & 0x1F;

        // Step 3: B = t - (Co >> 1)
        let b = (t - ((co as i16) >> 1)) & 0x1F;

        // Step 4: R = B + Co
        let r = (b + co as i16) & 0x1F;

        // Pack back into RGB565 format, preserving the original g_low bit
        Color565::from_raw(((r as u16) << 11) | ((g as u16) << 6) | (g_low << 5) | (b as u16))
    }

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
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
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
    /// dst[0] = recorrelate(src_ptr_0[0]),
    /// dst[1] = recorrelate(src_ptr_1[0]),
    /// dst[2] = recorrelate(src_ptr_0[1]),
    /// etc.
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
    #[cfg(not(tarpaulin_include))]
    pub unsafe fn recorrelate_ycocg_r_var1_ptr_split(
        src_ptr_0: *const Self,
        src_ptr_1: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
        debug_assert!(
            num_items % 2 == 0,
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
                    let color_0 = &*src_ptr_0.add(x);
                    let color_1 = &*src_ptr_1.add(x);
                    *dst_ptr.add(x * 2) = color_0.recorrelate_ycocg_r_var1();
                    *dst_ptr.add((x * 2) + 1) = color_1.recorrelate_ycocg_r_var1();
                }
            }
        }

        recorr(src_ptr_0, src_ptr_1, dst_ptr, num_items);
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var1`] to elements from two input slices,
    /// interleaving the recorrelated results into a single output slice.
    ///
    /// dst[0] = recorrelate(src_0[0]),
    /// dst[1] = recorrelate(src_1[0]),
    /// dst[2] = recorrelate(src_0[1]),
    /// dst[3] = recorrelate(src_1[1])
    /// etc.
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
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
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
    /// dst[0] = recorrelate(src_ptr_0[0]),
    /// dst[1] = recorrelate(src_ptr_1[0]),
    /// dst[2] = recorrelate(src_ptr_0[1]),
    /// etc.
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
    #[cfg(not(tarpaulin_include))]
    pub unsafe fn recorrelate_ycocg_r_var2_ptr_split(
        src_ptr_0: *const Self,
        src_ptr_1: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
        debug_assert!(
            num_items % 2 == 0,
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
                    let color_0 = &*src_ptr_0.add(x);
                    let color_1 = &*src_ptr_1.add(x);
                    *dst_ptr.add(x * 2) = color_0.recorrelate_ycocg_r_var2();
                    *dst_ptr.add((x * 2) + 1) = color_1.recorrelate_ycocg_r_var2();
                }
            }
        }

        recorr(src_ptr_0, src_ptr_1, dst_ptr, num_items);
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var2`] to elements from two input slices,
    /// interleaving the recorrelated results into a single output slice.
    ///
    /// dst[0] = recorrelate(src_0[0]),
    /// dst[1] = recorrelate(src_1[0]),
    /// dst[2] = recorrelate(src_0[1]),
    /// dst[3] = recorrelate(src_1[1])
    /// etc.
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
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
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
    /// dst[0] = recorrelate(src_ptr_0[0]),
    /// dst[1] = recorrelate(src_ptr_1[0]),
    /// dst[2] = recorrelate(src_ptr_0[1]),
    /// etc.
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
    #[cfg(not(tarpaulin_include))]
    pub unsafe fn recorrelate_ycocg_r_var3_ptr_split(
        src_ptr_0: *const Self,
        src_ptr_1: *const Self,
        dst_ptr: *mut Self,
        num_items: usize,
    ) {
        debug_assert!(
            num_items % 2 == 0,
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
                    let color_0 = &*src_ptr_0.add(x);
                    let color_1 = &*src_ptr_1.add(x);
                    *dst_ptr.add(x * 2) = color_0.recorrelate_ycocg_r_var3();
                    *dst_ptr.add((x * 2) + 1) = color_1.recorrelate_ycocg_r_var3();
                }
            }
        }

        recorr(src_ptr_0, src_ptr_1, dst_ptr, num_items);
    }

    /// Convenience function that applies [`Self::recorrelate_ycocg_r_var3`] to elements from two input slices,
    /// interleaving the recorrelated results into a single output slice.
    ///
    /// dst[0] = recorrelate(src_0[0]),
    /// dst[1] = recorrelate(src_1[0]),
    /// dst[2] = recorrelate(src_0[1]),
    /// dst[3] = recorrelate(src_1[1])
    /// etc.
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

    /// Applies the specified decorrelation variant to a color
    ///
    /// Wrapper around the variant-specific decorrelate methods.
    ///
    /// # Parameters
    ///
    /// - `variant`: The [`YCoCgVariant`] to use
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r(YCoCgVariant::Variant1);
    /// // Color is now in YCoCg-R form
    /// ```
    #[inline]
    pub fn decorrelate_ycocg_r(&self, variant: YCoCgVariant) -> Self {
        match variant {
            YCoCgVariant::Variant1 => self.decorrelate_ycocg_r_var1(),
            YCoCgVariant::Variant2 => self.decorrelate_ycocg_r_var2(),
            YCoCgVariant::Variant3 => self.decorrelate_ycocg_r_var3(),
            YCoCgVariant::None => *self, // No transformation
        }
    }

    /// Applies the specified recorrelation variant to a color
    ///
    /// Wrapper around the variant-specific recorrelate methods.
    ///
    /// # Parameters
    ///
    /// - `variant`: The [`YCoCgVariant`] to use
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r(YCoCgVariant::Variant1);
    /// color.recorrelate_ycocg_r(YCoCgVariant::Variant1);
    /// ```
    #[inline]
    pub fn recorrelate_ycocg_r(&self, variant: YCoCgVariant) -> Self {
        match variant {
            YCoCgVariant::Variant1 => self.recorrelate_ycocg_r_var1(),
            YCoCgVariant::Variant2 => self.recorrelate_ycocg_r_var2(),
            YCoCgVariant::Variant3 => self.recorrelate_ycocg_r_var3(),
            YCoCgVariant::None => *self, // No transformation
        }
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
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
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

    /// Raw pointer implementation for applying the specified recorrelation variant to a block of
    /// colors with split inputs.
    ///
    /// Takes two separate input raw pointers, applies the transformation to colors from both sources,
    /// and interleaves the results into a single output array.
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
    #[cfg(not(tarpaulin_include))]
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
                        num_items % 2 == 0,
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

/// Represents a function variant for decoration/recorrelation operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, AllValues)]
pub enum YCoCgVariant {
    /// Variant 1: Usually compresses best
    Variant1,
    /// Variant 2: Faster recorrelate (marginally) for compression speed
    Variant2,
    /// Variant 3: Sometimes better than variant 2
    Variant3,
    /// None: No transformation (original RGB values preserved)
    None,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Tests that colors can be properly decorrelated and recorrelated
    /// back to their original values for all three variants.
    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    #[case(YCoCgVariant::None)]
    fn can_decorrelate_recorrelate(#[case] variant: YCoCgVariant) {
        // Create a variety of test colors to ensure coverage across the color space
        let test_colors = [
            Color565::from_rgb(255, 0, 0),     // Red
            Color565::from_rgb(0, 255, 0),     // Green
            Color565::from_rgb(0, 0, 255),     // Blue
            Color565::from_rgb(255, 255, 0),   // Yellow
            Color565::from_rgb(0, 255, 255),   // Cyan
            Color565::from_rgb(255, 0, 255),   // Magenta
            Color565::from_rgb(128, 128, 128), // Gray
            Color565::from_rgb(255, 255, 255), // White
            Color565::from_rgb(0, 0, 0),       // Black
            Color565::from_rgb(255, 128, 64),  // Orange
            Color565::from_rgb(128, 0, 255),   // Purple
            Color565::from_rgb(0, 128, 64),    // Teal
            Color565::from_rgb(31, 79, 83),    // Prime Numbers
        ];

        // Test each color individually in a loop for easier debugging
        for (x, original_color) in test_colors.iter().enumerate() {
            // Step 1: Decorrelate
            let decorr = original_color.decorrelate_ycocg_r(variant);

            // Step 2: Recorrelate
            let recorr = decorr.recorrelate_ycocg_r(variant);

            // Verify the color is restored to its original value
            assert_eq!(
                recorr.raw_value(),
                original_color.raw_value(),
                "{variant:?} - Color at index {x} failed to restore."
            );
        }
    }

    /// Tests individual color transformations for each variant
    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    #[case(YCoCgVariant::None)]
    fn can_individual_color_operations(#[case] variant: YCoCgVariant) {
        // Test individual color transformations
        let original = Color565::from_rgb(255, 128, 64);
        let mut transformed = original;

        // Decorrelate
        transformed = transformed.decorrelate_ycocg_r(variant);
        if variant != YCoCgVariant::None {
            assert_ne!(
                transformed.raw_value(),
                original.raw_value(),
                "{variant:?} - Color should change after decorrelation"
            );
        }
        // Recorrelate
        transformed = transformed.recorrelate_ycocg_r(variant);
        assert_eq!(
            transformed.raw_value(),
            original.raw_value(),
            "{variant:?} - Color should be restored after recorrelation"
        );
    }
}
