use core::ptr::{read_unaligned, write_unaligned};
use dxt_lossless_transform_common::color_565::Color565;

#[cfg(not(feature = "no-runtime-cpu-detection"))]
#[cfg(feature = "nightly")]
use dxt_lossless_transform_common::cpu_detect::*;
use multiversion::multiversion;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[inline(always)]
pub(crate) unsafe fn untransform_split_and_decorrelate_variant1(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    untransform_split_and_decorrelate_variant1_x86(colors_ptr, indices_ptr, output_ptr, num_blocks);

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    untransform_split_and_decorrelate_variant1_generic(
        colors_ptr,
        indices_ptr,
        output_ptr,
        num_blocks,
    );
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_split_and_decorrelate_variant1_x86(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() && has_avx512bw() {
            avx512::untransform_split_and_decorrelate_variant1_avx512(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::untransform_split_and_decorrelate_variant1_avx512(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
            );
            return;
        }
    }

    // Fallback to portable implementation
    untransform_split_and_decorrelate_variant1_generic(
        colors_ptr,
        indices_ptr,
        output_ptr,
        num_blocks,
    )
}

#[inline(always)]
pub(crate) unsafe fn untransform_split_and_decorrelate_variant2(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    untransform_split_and_decorrelate_variant2_x86(colors_ptr, indices_ptr, output_ptr, num_blocks);

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    untransform_split_and_decorrelate_variant2_generic(
        colors_ptr,
        indices_ptr,
        output_ptr,
        num_blocks,
    );
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_split_and_decorrelate_variant2_x86(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() && has_avx512bw() {
            avx512::untransform_split_and_decorrelate_variant2_avx512(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::untransform_split_and_decorrelate_variant2_avx512(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
            );
            return;
        }
    }

    // Fallback to portable implementation
    untransform_split_and_decorrelate_variant2_generic(
        colors_ptr,
        indices_ptr,
        output_ptr,
        num_blocks,
    )
}

#[inline(always)]
pub(crate) unsafe fn untransform_split_and_decorrelate_variant3(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    untransform_split_and_decorrelate_variant3_x86(colors_ptr, indices_ptr, output_ptr, num_blocks);

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    untransform_split_and_decorrelate_variant3_generic(
        colors_ptr,
        indices_ptr,
        output_ptr,
        num_blocks,
    );
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_split_and_decorrelate_variant3_x86(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() && has_avx512bw() {
            avx512::untransform_split_and_decorrelate_variant3_avx512(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::untransform_split_and_decorrelate_variant3_avx512(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
            );
            return;
        }
    }

    // Fallback to portable implementation
    untransform_split_and_decorrelate_variant3_generic(
        colors_ptr,
        indices_ptr,
        output_ptr,
        num_blocks,
    )
}

#[multiversion(targets(
    // x86-64-v3 without lahfsahf
    "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
    // x86-64-v2 without lahfsahf
    "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
))]
#[inline(never)] // improve register budget.
unsafe fn untransform_split_and_decorrelate_variant1_generic(
    mut colors_ptr: *const u32,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    unsafe {
        for _ in 0..num_blocks {
            // Read both values first (better instruction scheduling)
            let color_raw = read_unaligned(colors_ptr);
            let index_value = read_unaligned(indices_ptr);

            colors_ptr = colors_ptr.add(1);
            indices_ptr = indices_ptr.add(1);

            // Extract both Color565 values from the u32
            let color0 = Color565::from_raw(color_raw as u16);
            let color1 = Color565::from_raw((color_raw >> 16) as u16);

            // Apply recorrelation to both colors
            let recorr_color0 = color0.recorrelate_ycocg_r_var1();
            let recorr_color1 = color1.recorrelate_ycocg_r_var1();

            // Pack both recorrelated colors back into u32
            let recorrelated_colors =
                (recorr_color0.raw_value() as u32) | ((recorr_color1.raw_value() as u32) << 16);

            // Write both values together
            write_unaligned(output_ptr as *mut u32, recorrelated_colors);
            write_unaligned(output_ptr.add(4) as *mut u32, index_value);

            output_ptr = output_ptr.add(8);
        }
    }
}

#[multiversion(targets(
    // x86-64-v3 without lahfsahf
    "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
    // x86-64-v2 without lahfsahf
    "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
))]
#[inline(never)] // improve register budget.
unsafe fn untransform_split_and_decorrelate_variant2_generic(
    mut colors_ptr: *const u32,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    unsafe {
        for _ in 0..num_blocks {
            // Read color data (4 bytes containing two Color565 values)
            let color_raw = read_unaligned(colors_ptr);
            colors_ptr = colors_ptr.add(1);

            // Extract both Color565 values from the u32
            let color0 = Color565::from_raw(color_raw as u16); // Lower 16 bits
            let color1 = Color565::from_raw((color_raw >> 16) as u16); // Upper 16 bits

            // Apply recorrelation to both colors
            let recorr_color0 = color0.recorrelate_ycocg_r_var2();
            let recorr_color1 = color1.recorrelate_ycocg_r_var2();

            // Pack both recorrelated colors back into u32
            let recorrelated_colors =
                (recorr_color0.raw_value() as u32) | ((recorr_color1.raw_value() as u32) << 16);

            // Read index data (4 bytes)
            let index_value = read_unaligned(indices_ptr);
            indices_ptr = indices_ptr.add(1);

            // Write interleaved BC1 block: colors (4 bytes) + indices (4 bytes)
            write_unaligned(output_ptr as *mut u32, recorrelated_colors);
            write_unaligned(output_ptr.add(4) as *mut u32, index_value);

            // Move to next output block
            output_ptr = output_ptr.add(8);
        }
    }
}

#[multiversion(targets(
    // x86-64-v3 without lahfsahf
    "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
    // x86-64-v2 without lahfsahf
    "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
))]
#[inline(never)] // improve register budget.
unsafe fn untransform_split_and_decorrelate_variant3_generic(
    mut colors_ptr: *const u32,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    unsafe {
        for _ in 0..num_blocks {
            // Read color data (4 bytes containing two Color565 values)
            let color_raw = read_unaligned(colors_ptr);
            colors_ptr = colors_ptr.add(1);

            // Extract both Color565 values from the u32
            let color0 = Color565::from_raw(color_raw as u16); // Lower 16 bits
            let color1 = Color565::from_raw((color_raw >> 16) as u16); // Upper 16 bits

            // Apply recorrelation to both colors
            let recorr_color0 = color0.recorrelate_ycocg_r_var3();
            let recorr_color1 = color1.recorrelate_ycocg_r_var3();

            // Pack both recorrelated colors back into u32
            let recorrelated_colors =
                (recorr_color0.raw_value() as u32) | ((recorr_color1.raw_value() as u32) << 16);

            // Read index data (4 bytes)
            let index_value = read_unaligned(indices_ptr);
            indices_ptr = indices_ptr.add(1);

            // Write interleaved BC1 block: colors (4 bytes) + indices (4 bytes)
            write_unaligned(output_ptr as *mut u32, recorrelated_colors);
            write_unaligned(output_ptr.add(4) as *mut u32, index_value);

            // Move to next output block
            output_ptr = output_ptr.add(8);
        }
    }
}
