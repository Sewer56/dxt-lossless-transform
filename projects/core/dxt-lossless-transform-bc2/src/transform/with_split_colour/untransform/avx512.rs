#![allow(unused_imports)]

use crate::transform::with_split_colour::untransform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// AVX512 implementation for split‐colour untransform for BC2.
///
/// This implementation processes BC2 blocks in chunks of 8 blocks (128 bytes)
/// and combines separate alpha, color0, color1, and indices arrays back into
/// standard BC2 block format.
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[allow(unused_variables)]
#[allow(unused_mut)]
pub(crate) unsafe fn untransform_with_split_colour(
    mut alpha_ptr: *const u64,
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // TODO: AVX512 once I get back home.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    fn avx512_untransform_roundtrip() {
        if !has_avx512bw() || !has_avx512f() {
            return;
        }

        // 128 bytes processed per main loop iteration (* 2 / 16 == 16)
        run_split_colour_untransform_roundtrip_test(untransform_with_split_colour, 16, "AVX512");
    }
}
