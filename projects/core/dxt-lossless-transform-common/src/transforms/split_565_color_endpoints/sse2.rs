use crate::transforms::split_565_color_endpoints::portable32::u32_with_separate_endpoints;

/// Alternative implementation using pshuflw and pshufhw instructions from SSE2,
/// processing 64 bytes at once (unroll factor of 2)
///
/// # Arguments
///
/// * `colors` - Pointer to the input array of colors
/// * `colors_out` - Pointer to the output array of colors
/// * `colors_len_bytes` - Number of bytes in the input array.
///
/// # Safety
///
/// - `colors` must be valid for reads of `colors_len_bytes` bytes
/// - `colors_out` must be valid for writes of `colors_len_bytes` bytes
/// - `colors_len_bytes` must be a multiple of 4
/// - Pointers should be 16-byte aligned for best performance
/// - CPU must support SSE2 instructions
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub(crate) unsafe fn sse2_shuf_unroll2_impl_asm(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Calculate end pointer for our main loop (process 64 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);
    let aligned_end_ptr = end_ptr.sub(64);

    if input_ptr < aligned_end_ptr {
        core::arch::asm!(
            // Loop alignment
            ".p2align 4",

            // Main processing loop
            "2:",  // Label for loop start

            // Load 64 bytes (4 XMM registers)
            "movdqu {xmm0}, xmmword ptr [{src_ptr}]",
            "movdqu {xmm1}, xmmword ptr [{src_ptr} + 16]",
            "movdqu {xmm3}, xmmword ptr [{src_ptr} + 48]",
            "movdqu {xmm2}, xmmword ptr [{src_ptr} + 32]",
            "add {src_ptr}, 64",

            // Shuffle operations for first chunk
            "pshuflw {xmm0}, {xmm0}, 216", // 11011000b = 216
            "pshuflw {xmm1}, {xmm1}, 216",
            "pshuflw {xmm3}, {xmm3}, 216",
            "pshufhw {xmm0}, {xmm0}, 39",  // 00100111b = 39
            "pshufhw {xmm1}, {xmm1}, 39",
            "pshufhw {xmm3}, {xmm3}, 39",
            "pshufd {xmm0}, {xmm0}, 108",  // 01101100b = 108
            "pshufd {xmm1}, {xmm1}, 108",
            "pshufd {xmm3}, {xmm3}, 108",
            "pshufhw {xmm4}, {xmm0}, 30",  // 00011110b = 30
            "pshufhw {xmm5}, {xmm1}, 30",
            "pshuflw {xmm0}, {xmm0}, 180", // 10110100b = 180
            "pshuflw {xmm1}, {xmm1}, 180",
            "punpcklqdq {xmm0}, {xmm1}",
            "pshuflw {xmm1}, {xmm2}, 216",
            "movhlps {xmm5}, {xmm4}",
            "pshufhw {xmm4}, {xmm3}, 30",
            "pshuflw {xmm3}, {xmm3}, 180",
            "pshufhw {xmm1}, {xmm1}, 39",

            // Store first result
            "movdqu xmmword ptr [{out_low}], {xmm0}",
            "pshufd {xmm1}, {xmm1}, 108",
            "pshufhw {xmm2}, {xmm1}, 30",
            "pshuflw {xmm1}, {xmm1}, 180",
            "punpcklqdq {xmm1}, {xmm3}",
            "movhlps {xmm4}, {xmm2}",

            // Store remaining results
            "movdqu xmmword ptr [{out_low} + 16], {xmm1}",
            "movups xmmword ptr [{out_high}], {xmm5}",
            "movups xmmword ptr [{out_high} + 16], {xmm4}",
            "add {out_low}, 32",
            "add {out_high}, 32",

            // Loop condition
            "cmp {src_ptr}, {end_ptr}",
            "jb 2b",

            src_ptr = inout(reg) input_ptr,
            end_ptr = in(reg) aligned_end_ptr,
            out_low = inout(reg) output_low,
            out_high = inout(reg) output_high,

            // XMM registers
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,

            options(preserves_flags, nostack)
        );
    }

    // Handle remaining elements
    u32_with_separate_endpoints(
        end_ptr as *const u32,
        input_ptr as *const u32,
        output_low as *mut u16,
        output_high as *mut u16,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::split_565_color_endpoints::tests::{
        test_implementation_aligned, test_implementation_unaligned, TransformFn,
    };
    use rstest::rstest;

    #[rstest]
    #[case(sse2_shuf_unroll2_impl_asm, "sse2_shuf_unroll2_asm")]
    fn test_sse2_aligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
        test_implementation_aligned(implementation, impl_name);
    }

    #[rstest]
    #[case(sse2_shuf_unroll2_impl_asm, "sse2_shuf_unroll2_asm")]
    fn test_sse2_unaligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
        test_implementation_unaligned(implementation, impl_name);
    }
}
