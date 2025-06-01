#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::ptr::{read_unaligned, write_unaligned};
use dxt_lossless_transform_common::color_565::Color565;

#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline(never)] // improve register budget.
pub(crate) unsafe fn untransform_split_and_decorrelate_variant1_avx512(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    // Load constants from assembly (.LCPI42_12 through .LCPI42_17)
    // vpbroadcastw zmm0, word ptr [rip + .LCPI42_12] ; load 15
    let zmm0 = _mm512_set1_epi16(15);

    // vpbroadcastw zmm1, word ptr [rip + .LCPI42_13] ; load 1984
    let zmm1 = _mm512_set1_epi16(1984);

    // vpbroadcastw zmm2, word ptr [rip + .LCPI42_14] ; load 32
    let zmm2 = _mm512_set1_epi16(32);

    // vpbroadcastw zmm3, word ptr [rip + .LCPI42_15] ; load 31
    let zmm3 = _mm512_set1_epi16(31);

    // vpmovsxbw zmm4, ymmword ptr [rip + .LCPI42_16] ; load shuffle pattern 1
    // Note: vpmovsxbw extends bytes to words, so we need to create the pattern as 16-bit values
    let zmm4 = _mm512_set_epi16(
        47, 46, 23, 7, 45, 44, 22, 6, 43, 42, 21, 5, 41, 40, 20, 4, 39, 38, 19, 3, 37, 36, 18, 2,
        35, 34, 17, 1, 33, 32, 16, 0,
    );

    // vpmovsxbw zmm5, ymmword ptr [rip + .LCPI42_17] ; load shuffle pattern 2
    let zmm5 = _mm512_set_epi16(
        63, 62, 31, 15, 61, 60, 30, 14, 59, 58, 29, 13, 57, 56, 28, 12, 55, 54, 27, 11, 53, 52, 26,
        10, 51, 50, 25, 9, 49, 48, 24, 8,
    );

    // mov r10, rcx ; and r10, -16 ; vectorized_blocks = num_blocks & !15
    let r10 = num_blocks & !15;

    // xor eax, eax ; rax = 0
    let mut rax = 0;

    // .LBB42_12: loop start
    while rax < r10 {
        // vmovdqu64 zmm6, zmmword ptr [rdi + 4*rax] ; load 16 colors
        let zmm6 = _mm512_loadu_si512(colors_ptr.add(rax) as *const __m512i);

        // vmovdqu64 zmm7, zmmword ptr [rsi + 4*rax] ; load 16 indices
        let zmm7 = _mm512_loadu_si512(indices_ptr.add(rax) as *const __m512i);

        // vpmovdw ymm8, zmm6 ; convert 32->16 bit (low parts)
        let ymm8 = _mm512_cvtepi32_epi16(zmm6);

        // vpsrld zmm16, zmm6, 17 ; shift right by 17
        let zmm16 = _mm512_srli_epi32(zmm6, 17);

        // vpsrld zmm9, zmm6, 16 ; shift right by 16
        let zmm9 = _mm512_srli_epi32(zmm6, 16);

        // vpsrld zmm14, zmm6, 27 ; shift right by 27
        let zmm14 = _mm512_srli_epi32(zmm6, 27);

        // vpsrld zmm15, zmm6, 22 ; shift right by 22
        let zmm15 = _mm512_srli_epi32(zmm6, 22);

        // vpsrld zmm6, zmm6, 23 ; shift right by 23
        let zmm6 = _mm512_srli_epi32(zmm6, 23);

        // vpmovdw ymm16, zmm16 ; convert shifted values to 16-bit
        let ymm16 = _mm512_cvtepi32_epi16(zmm16);

        // vpmovdw ymm14, zmm14
        let ymm14 = _mm512_cvtepi32_epi16(zmm14);

        // vpmovdw ymm6, zmm6
        let ymm6 = _mm512_cvtepi32_epi16(zmm6);

        // vpmovdw ymm9, zmm9
        let ymm9 = _mm512_cvtepi32_epi16(zmm9);

        // vpmovdw ymm15, zmm15
        let ymm15 = _mm512_cvtepi32_epi16(zmm15);

        // vpsrlw ymm12, ymm8, 1 ; shift low parts right by 1
        let ymm12 = _mm256_srli_epi16(ymm8, 1);

        // vpsrlw ymm10, ymm8, 11 ; shift low parts right by 11
        let ymm10 = _mm256_srli_epi16(ymm8, 11);

        // vpsrlw ymm13, ymm8, 7 ; shift low parts right by 7
        let ymm13 = _mm256_srli_epi16(ymm8, 7);

        // vpsrlw ymm11, ymm8, 6 ; shift low parts right by 6
        let ymm11 = _mm256_srli_epi16(ymm8, 6);

        // vinserti64x4 zmm12, zmm12, ymm16, 1 ; combine low and high parts
        let zmm12 = _mm512_inserti64x4(_mm512_castsi256_si512(ymm12), ymm16, 1);

        // vinserti64x4 zmm6, zmm13, ymm6, 1
        let zmm6 = _mm512_inserti64x4(_mm512_castsi256_si512(ymm13), ymm6, 1);

        // vinserti64x4 zmm10, zmm10, ymm14, 1
        let zmm10 = _mm512_inserti64x4(_mm512_castsi256_si512(ymm10), ymm14, 1);

        // vinserti64x4 zmm8, zmm8, ymm9, 1
        let zmm8 = _mm512_inserti64x4(_mm512_castsi256_si512(ymm8), ymm9, 1);

        // vinserti64x4 zmm11, zmm11, ymm15, 1
        let zmm11 = _mm512_inserti64x4(_mm512_castsi256_si512(ymm11), ymm15, 1);

        // vpandq zmm12, zmm12, zmm0 ; mask with 15
        let zmm12 = _mm512_and_si512(zmm12, zmm0);

        // vpandq zmm6, zmm6, zmm0 ; mask with 15
        let zmm6 = _mm512_and_si512(zmm6, zmm0);

        // vpsubw zmm10, zmm10, zmm12 ; subtract
        let zmm10 = _mm512_sub_epi16(zmm10, zmm12);

        // vpsubw zmm6, zmm10, zmm6 ; subtract
        let zmm6 = _mm512_sub_epi16(zmm10, zmm6);

        // vpaddw zmm9, zmm10, zmm8 ; add
        let zmm9 = _mm512_add_epi16(zmm10, zmm8);

        // vpaddw zmm10, zmm6, zmm11 ; add
        let zmm10 = _mm512_add_epi16(zmm6, zmm11);

        // vpsllw zmm9, zmm9, 6 ; shift left by 6
        let zmm9 = _mm512_slli_epi16(zmm9, 6);

        // vpsllw zmm10, zmm10, 11 ; shift left by 11
        let zmm10 = _mm512_slli_epi16(zmm10, 11);

        // vpternlogq zmm10, zmm9, zmm1, 248 ; ternary logic OR
        let zmm10 = _mm512_ternarylogic_epi32(zmm10, zmm9, zmm1, 248);

        // vpternlogq zmm10, zmm8, zmm2, 248 ; ternary logic OR
        let zmm10 = _mm512_ternarylogic_epi32(zmm10, zmm8, zmm2, 248);

        // vpternlogq zmm10, zmm6, zmm3, 248 ; ternary logic OR
        let zmm10 = _mm512_ternarylogic_epi32(zmm10, zmm6, zmm3, 248);

        // vmovdqa64 zmm6, zmm10 ; copy for permute operations
        let zmm6 = zmm10;

        // vpermt2w zmm6, zmm4, zmm7 ; permute with pattern 1
        let zmm6 = _mm512_permutex2var_epi16(zmm6, zmm4, zmm7);

        // vpermt2w zmm10, zmm5, zmm7 ; permute with pattern 2
        let zmm10 = _mm512_permutex2var_epi16(zmm10, zmm5, zmm7);

        // vmovdqu64 zmmword ptr [rdx + 8*rax + 64], zmm10 ; store high part
        _mm512_storeu_si512(output_ptr.add(rax * 8 + 64) as *mut __m512i, zmm10);

        // vmovdqu64 zmmword ptr [rdx + 8*rax], zmm6 ; store low part
        _mm512_storeu_si512(output_ptr.add(rax * 8) as *mut __m512i, zmm6);

        // add rax, 16 ; increment counter
        rax += 16;

        // cmp r10, rax ; jne .LBB42_12 ; loop condition checked by while
    }

    // Handle remaining blocks using scalar fallback
    for block_idx in r10..num_blocks {
        // Read both values first (better instruction scheduling)
        let color_raw = read_unaligned(colors_ptr.add(block_idx));
        let index_value = read_unaligned(indices_ptr.add(block_idx));

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
        write_unaligned(
            output_ptr.add(block_idx * 8) as *mut u32,
            recorrelated_colors,
        );
        write_unaligned(output_ptr.add(block_idx * 8 + 4) as *mut u32, index_value);
    }
}
