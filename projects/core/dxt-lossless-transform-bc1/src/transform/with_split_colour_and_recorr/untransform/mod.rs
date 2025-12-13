use dxt_lossless_transform_common::color_565::YCoCgVariant;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx512bw;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

mod generic;

pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        untransform_with_split_colour_and_recorr_x86(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
            decorrelation_mode,
        );
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        generic::untransform_with_split_colour_and_recorr_generic(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
            decorrelation_mode,
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn untransform_with_split_colour_and_recorr_x86(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if has_avx512bw() {
            avx512bw::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if has_avx2() {
            avx2::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if has_sse2() {
            sse2::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512bw") {
            avx512bw::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }
    }

    generic::untransform_with_split_colour_and_recorr_generic(
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
        decorrelation_mode,
    );
}
