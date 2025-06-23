#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx512;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

mod generic;

pub(crate) unsafe fn untransform_with_split_colour(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        untransform_with_split_colour_x86(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        );
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        generic::untransform_with_split_colour(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        );
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
unsafe fn untransform_with_split_colour_x86(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() && has_avx512bw() {
            avx512::untransform_with_split_colour(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }

        if has_avx2() {
            avx2::untransform_with_split_colour(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }

        if has_sse2() {
            sse2::untransform_with_split_colour(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::untransform_with_split_colour(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::untransform_with_split_colour(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::untransform_with_split_colour(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }
    }

    generic::untransform_with_split_colour(
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    );
}
