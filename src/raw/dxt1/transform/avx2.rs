use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 64 (for two AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
pub unsafe fn permute(mut input_ptr: *const u8, mut output_ptr: *mut u8, mut len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        asm!(
            // Calculate end address
            "add {end}, {src_ptr}",  // end = src + len

            // Calculate second destination pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",

            // Load permutation indices into low register
            "vmovdqu ymm0, [{perm}]",

            ".p2align 5",
            "2:",

            // Load 64 bytes
            "vmovdqa ymm1, [{src_ptr}]",
            "vmovdqa ymm2, [{src_ptr} + 32]",
            "add {src_ptr}, 64",  // src += 64

            // Use vpermd to group colors/indices
            "vpermd ymm3, ymm0, ymm1",
            "vpermd ymm4, ymm0, ymm2",

            // Use vperm2i128 to get all colors in one register and all indices in another
            "vperm2i128 ymm5, ymm3, ymm4, 0x20",  // get all colors
            "vperm2i128 ymm6, ymm3, ymm4, 0x31",  // get all indices

            // Store results
            "vmovdqa [{colors_ptr}], ymm5",
            "vmovdqa [{indices_ptr}], ymm6",

            // Update pointers
            "add {colors_ptr}, 32",
            "add {indices_ptr}, 32",

            // Compare against end address and loop if not done
            "cmp {src_ptr}, {end}",
            "jb 2b",

            // Clear YMM registers
            "vzeroupper",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = inout(reg) len,
            indices_ptr = out(reg) _,
            perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (for four AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
pub unsafe fn permute_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, mut len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        asm!(
            // Calculate end address
            "add {end}, {src_ptr}",  // end = src + len

            // Calculate second destination pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",

            // Load permutation indices into low register
            "vmovdqu ymm0, [{perm}]",

            ".p2align 5",
            "2:",

            // Load all inputs first to utilize memory pipeline
            "vmovdqa ymm1, [{src_ptr}]",
            "vmovdqa ymm2, [{src_ptr} + 32]",
            "vmovdqa ymm3, [{src_ptr} + 64]",
            "vmovdqa ymm4, [{src_ptr} + 96]",
            "add {src_ptr}, 128",  // src += 128

            // Do all vpermd operations
            "vpermd ymm5, ymm0, ymm1",  // first block
            "vpermd ymm6, ymm0, ymm2",
            "vpermd ymm1, ymm0, ymm3",  // second block (reuse ymm1/2)
            "vpermd ymm2, ymm0, ymm4",

            // Do all vperm2i128 operations
            "vperm2i128 ymm3, ymm5, ymm6, 0x20",  // all colors first block
            "vperm2i128 ymm4, ymm5, ymm6, 0x31",  // all indices first block
            "vperm2i128 ymm5, ymm1, ymm2, 0x20",  // all colors second block
            "vperm2i128 ymm6, ymm1, ymm2, 0x31",  // all indices second block

            // Store all results
            "vmovdqa [{colors_ptr}], ymm3",      // Store all colors
            "vmovdqa [{colors_ptr} + 32], ymm5",
            "add {colors_ptr}, 64",
            "vmovdqa [{indices_ptr}], ymm4",      // Store all indices
            "vmovdqa [{indices_ptr} + 32], ymm6",
            "add {indices_ptr}, 64",

            // Compare against end address and loop if not done
            "cmp {src_ptr}, {end}",
            "jb 2b",

            // Clear YMM registers
            "vzeroupper",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = inout(reg) len,
            indices_ptr = out(reg) _,
            perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 256 (for eight AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn permute_unroll_4(mut input_ptr: *const u8, mut output_ptr: *mut u8, mut len: usize) {
    debug_assert!(len % 256 == 0);

    unsafe {
        asm!(
            // Calculate end address
            "add {end}, {src_ptr}",  // end = src + len

            // Calculate second destination pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",

            // Load permutation indices for vpermd
            "vmovdqu ymm15, [{perm}]",

            // Align the loop's instruction address to 32 bytes
            ".p2align 5",
            "2:",

            // Load 256 bytes (32 blocks)
            "vmovdqa ymm0, [{src_ptr}]",
            "vmovdqa ymm1, [{src_ptr} + 32]",
            "vmovdqa ymm4, [{src_ptr} + 64]",
            "vmovdqa ymm5, [{src_ptr} + 96]",
            "vmovdqa ymm8, [{src_ptr} + 128]",
            "vmovdqa ymm9, [{src_ptr} + 160]",
            "vmovdqa ymm12, [{src_ptr} + 192]",
            "vmovdqa ymm13, [{src_ptr} + 224]",
            "add {src_ptr}, 256",  // src += 256

            // Do all vpermd operations
            "vpermd ymm2, ymm15, ymm0",
            "vpermd ymm3, ymm15, ymm1",
            "vpermd ymm6, ymm15, ymm4",
            "vpermd ymm7, ymm15, ymm5",
            "vpermd ymm10, ymm15, ymm8",
            "vpermd ymm11, ymm15, ymm9",
            "vpermd ymm14, ymm15, ymm12",
            "vpermd ymm0, ymm15, ymm13",

            // Do all vperm2i128 operations
            "vperm2i128 ymm1, ymm2, ymm3, 0x20",
            "vperm2i128 ymm2, ymm2, ymm3, 0x31",
            "vperm2i128 ymm3, ymm6, ymm7, 0x20",
            "vperm2i128 ymm4, ymm6, ymm7, 0x31",
            "vperm2i128 ymm5, ymm10, ymm11, 0x20",
            "vperm2i128 ymm6, ymm10, ymm11, 0x31",
            "vperm2i128 ymm7, ymm14, ymm0, 0x20",
            "vperm2i128 ymm8, ymm14, ymm0, 0x31",

            // Store all results
            "vmovdqa [{colors_ptr}], ymm1",
            "vmovdqa [{colors_ptr} + 32], ymm3",
            "vmovdqa [{colors_ptr} + 64], ymm5",
            "vmovdqa [{colors_ptr} + 96], ymm7",
            "add {colors_ptr}, 128",

            "vmovdqa [{indices_ptr}], ymm2",
            "vmovdqa [{indices_ptr} + 32], ymm4",
            "vmovdqa [{indices_ptr} + 64], ymm6",
            "vmovdqa [{indices_ptr} + 96], ymm8",
            "add {indices_ptr}, 128",

            // Compare against end address and loop if not done
            "cmp {src_ptr}, {end}",
            "jb 2b",

            // Clear YMM registers
            "vzeroupper",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = inout(reg) len,
            indices_ptr = out(reg) _,
            perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (for four AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
pub unsafe fn gather(mut input_ptr: *const u8, mut output_ptr: *mut u8, mut len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        asm!(
            // Calculate end address
            "add {end}, {src_ptr}",  // end = src + len

            // Calculate second destination pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",

            // Load gather indices into lower ymm registers
            "vmovdqu ymm0, [{color_idx}]",  // for colors (0,2,4,6,8,10,12,14)
            "vmovdqu ymm1, [{index_idx}]",  // for indices (1,3,5,7,9,11,13,15)

            ".p2align 5",
            "2:",

            // Gather colors using even indices
            "vpcmpeqd ymm2, ymm2, ymm2",
            "vpgatherdd ymm3, [{src_ptr} + ymm0 * 4], ymm2",  // scale = 4 for 32-bit elements

            // Gather indices using odd indices
            "vpcmpeqd ymm2, ymm2, ymm2",
            "vpgatherdd ymm4, [{src_ptr} + ymm1 * 4], ymm2",  // scale = 4 for 32-bit elements
            "add {src_ptr}, 64",   // src += 64

            // Store results
            "vmovdqa [{colors_ptr}], ymm3",
            "add {colors_ptr}, 32",
            "vmovdqa [{indices_ptr}], ymm4",
            "add {indices_ptr}, 32",

            // Compare against end address and loop if not done
            "cmp {src_ptr}, {end}",
            "jb 2b",

            // Clear YMM registers
            "vzeroupper",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = inout(reg) len,
            indices_ptr = out(reg) _,
            color_idx = in(reg) &[0, 2, 4, 6, 8, 10, 12, 14u32],
            index_idx = in(reg) &[1, 3, 5, 7, 9, 11, 13, 15u32],
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 256 (for eight AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[cfg(target_arch = "x86_64")]
pub unsafe fn gather_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 256 == 0);

    unsafe {
        asm!(
            // Preserve non-volatile registers we'll use
            "push rbx",
            "push r12",
            "push r13",

            // Calculate end address
            "mov rbx, {src}",
            "add rbx, {len}",  // end = src + len

            // Store pointers in preserved registers
            "mov r12, {src}",      // src
            "mov r13, {dst}",      // dst

            // Load gather indices for colors (every even index)
            "vmovdqu ymm15, [{color_idx}]",
            // Load gather indices for block indices (every odd index)
            "vmovdqu ymm14, [{index_idx}]",

            ".p2align 5",
            "2:",

            // Gather all colors
            "vpcmpeqd ymm13, ymm13, ymm13",
            "vpgatherdd ymm0, [r12 + ymm15 * 4], ymm13",
            "vpcmpeqd ymm13, ymm13, ymm13",
            "vpgatherdd ymm2, [r12 + 64 + ymm15 * 4], ymm13",
            "vpcmpeqd ymm13, ymm13, ymm13",
            "vpgatherdd ymm4, [r12 + 128 + ymm15 * 4], ymm13",
            "vpcmpeqd ymm13, ymm13, ymm13",
            "vpgatherdd ymm6, [r12 + 192 + ymm15 * 4], ymm13",

            // Gather all indices
            "vpcmpeqd ymm13, ymm13, ymm13",
            "vpgatherdd ymm1, [r12 + ymm14 * 4], ymm13",
            "vpcmpeqd ymm13, ymm13, ymm13",
            "vpgatherdd ymm3, [r12 + 64 + ymm14 * 4], ymm13",
            "vpcmpeqd ymm13, ymm13, ymm13",
            "vpgatherdd ymm5, [r12 + 128 + ymm14 * 4], ymm13",
            "vpcmpeqd ymm13, ymm13, ymm13",
            "vpgatherdd ymm7, [r12 + 192 + ymm14 * 4], ymm13",
            "add r12, 256",   // src += 256 (4 iterations * 64 bytes)

            // Store all colors
            "vmovdqa [r13], ymm0",
            "vmovdqa [r13 + 32], ymm2",
            "vmovdqa [r13 + 64], ymm4",
            "vmovdqa [r13 + 96], ymm6",

            // Store all indices
            "vmovdqa [r13 + {len_half}], ymm1",
            "vmovdqa [r13 + {len_half} + 32], ymm3",
            "vmovdqa [r13 + {len_half} + 64], ymm5",
            "vmovdqa [r13 + {len_half} + 96], ymm7",

            // Update pointers
            "add r13, 128",   // dst += 128 (4 iterations * 32 bytes)

            // Compare against end address and loop if not done
            "cmp r12, rbx",
            "jb 2b",

            // Restore preserved registers
            "pop r13",
            "pop r12",
            "pop rbx",

            // Clear YMM registers
            "vzeroupper",

            src = in(reg) input_ptr,
            dst = in(reg) output_ptr,
            len = in(reg) len,
            len_half = in(reg) len / 2,
            // Indices for gathering colors (0, 2, 4, 6, 8, 10, 12, 14)
            color_idx = in(reg) &[0, 2, 4, 6, 8, 10, 12, 14u32],
            // Indices for gathering block indices (1, 3, 5, 7, 9, 11, 13, 15)
            index_idx = in(reg) &[1, 3, 5, 7, 9, 11, 13, 15u32],
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 64 (for two AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
pub unsafe fn shuffle_permute(mut input_ptr: *const u8, mut output_ptr: *mut u8, mut len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        asm!(
            // Calculate end address
            "add {end}, {src_ptr}", // end = src + len

            // Calculate second destination pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",

            // Align the loop's instruction address to 32 bytes.
            // This isn't strictly needed, but more modern processors fetch + execute instructions in
            // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
            // assume a processor with AVX2 will be one of those.
            ".p2align 5",
            "2:",

            // Load 64 bytes
            "vmovdqa ymm0, [{src_ptr}]",
            "vmovdqa ymm1, [{src_ptr} + 32]",

            // Update source pointer early
            "add {src_ptr}, 64",  // src += 64

            // Do shuffles
            "vshufps ymm2, ymm0, ymm1, 136",  // colors (0b10001000)
            "vshufps ymm3, ymm0, ymm1, 221",  // indices (0b11011101)

            // Do permutes
            "vpermpd ymm2, ymm2, 216",  // arrange colors (0b11011000)
            "vpermpd ymm3, ymm3, 216",  // arrange indices (0b11011000)

            // Store results
            "vmovdqa [{colors_ptr}], ymm2",  // Store colors
            "add {colors_ptr}, 32",  // colors_ptr += 32
            "vmovdqa [{indices_ptr}], ymm3",      // Store indices
            "add {indices_ptr}, 32",      // indices_ptr += 32

            // Compare against end address and loop if not done
            "cmp {src_ptr}, {end}",
            "jb 2b",

            // Clear YMM registers
            "vzeroupper",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = inout(reg) len,
            indices_ptr = out(reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (for four AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
pub unsafe fn shuffle_permute_unroll_2(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    mut len: usize,
) {
    debug_assert!(len % 128 == 0);

    unsafe {
        asm!(
            // Calculate end address
            "add {end}, {src_ptr}", // end = src + len

            // Calculate second destination pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",

            // Align the loop's instruction address to 32 bytes.
            // This isn't strictly needed, but more modern processors fetch + execute instructions in
            // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
            // assume a processor with AVX2 will be one of those.
            ".p2align 5",
            "2:",

            // Load all 128 bytes first to utilize memory pipeline
            "vmovdqa ymm0, [{src_ptr}]",
            "vmovdqa ymm1, [{src_ptr} + 32]",
            "vmovdqa ymm4, [{src_ptr} + 64]",
            "vmovdqa ymm5, [{src_ptr} + 96]",
            "add {src_ptr}, 128",  // src += 128

            // Do all shuffles together to utilize shuffle units
            "vshufps ymm2, ymm0, ymm1, 136",  // colors (0b10001000)
            "vshufps ymm3, ymm0, ymm1, 221",  // indices (0b11011101)
            "vshufps ymm6, ymm4, ymm5, 136",  // colors (0b10001000)
            "vshufps ymm7, ymm4, ymm5, 221",  // indices (0b11011101)

            // Group permutes together
            "vpermpd ymm2, ymm2, 216",  // arrange colors (0b11011000)
            "vpermpd ymm3, ymm3, 216",  // arrange indices (0b11011000)
            "vpermpd ymm6, ymm6, 216",  // arrange colors (0b11011000)
            "vpermpd ymm7, ymm7, 216",  // arrange indices (0b11011000)

            // Store all results together to utilize store pipeline
            "vmovdqa [{colors_ptr}], ymm2",      // Store colors
            "vmovdqa [{colors_ptr} + 32], ymm6",  // Store colors
            "add {colors_ptr}, 64",   // colors_ptr += 64

            "vmovdqa [{indices_ptr}], ymm3",      // Store indices
            "vmovdqa [{indices_ptr} + 32], ymm7",  // Store indices
            "add {indices_ptr}, 64",   // indices_ptr += 64

            // Compare against end address and loop if not done
            "cmp {src_ptr}, {end}",
            "jb 2b",

            // Clear YMM registers
            "vzeroupper",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = inout(reg) len,
            indices_ptr = out(reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 256 (for eight AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn shuffle_permute_unroll_4(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    mut len: usize,
) {
    debug_assert!(len % 256 == 0);

    unsafe {
        asm!(
            // Calculate end address
            "add {end}, {src_ptr}", // end = src + len

            // Calculate second destination pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",

            // Align the loop's instruction address to 32 bytes.
            // This isn't strictly needed, but more modern processors fetch + execute instructions in
            // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
            // assume a processor with AVX2 will be one of those.
            ".p2align 5",
            "2:",

            // Load all 256 bytes in sequence to maximize memory throughput
            "vmovdqa ymm0, [{src_ptr}]",
            "vmovdqa ymm1, [{src_ptr} + 32]",
            "vmovdqa ymm4, [{src_ptr} + 64]",
            "vmovdqa ymm5, [{src_ptr} + 96]",
            "vmovdqa ymm8, [{src_ptr} + 128]",
            "vmovdqa ymm9, [{src_ptr} + 160]",
            "vmovdqa ymm12, [{src_ptr} + 192]",
            "vmovdqa ymm13, [{src_ptr} + 224]",
            "add {src_ptr}, 256",  // src += 256

            // Group all shuffles together to better utilize shuffle units
            "vshufps ymm2, ymm0, ymm1, 136",   // colors block 1
            "vshufps ymm3, ymm0, ymm1, 221",   // indices block 1
            "vshufps ymm6, ymm4, ymm5, 136",   // colors block 2
            "vshufps ymm7, ymm4, ymm5, 221",   // indices block 2
            "vshufps ymm10, ymm8, ymm9, 136",  // colors block 3
            "vshufps ymm11, ymm8, ymm9, 221",  // indices block 3
            "vshufps ymm14, ymm12, ymm13, 136", // colors block 4
            "vshufps ymm15, ymm12, ymm13, 221", // indices block 4

            // Group permutes together
            "vpermpd ymm2, ymm2, 216",   // arrange colors block 1
            "vpermpd ymm3, ymm3, 216",   // arrange indices block 1
            "vpermpd ymm6, ymm6, 216",   // arrange colors block 2
            "vpermpd ymm7, ymm7, 216",   // arrange indices block 2
            "vpermpd ymm10, ymm10, 216", // arrange colors block 3
            "vpermpd ymm11, ymm11, 216", // arrange indices block 3
            "vpermpd ymm14, ymm14, 216", // arrange colors block 4
            "vpermpd ymm15, ymm15, 216", // arrange indices block 4

            // Group all stores together to maximize store throughput
            // Store colors
            "vmovdqa [{colors_ptr}], ymm2",
            "vmovdqa [{colors_ptr} + 32], ymm6",
            "vmovdqa [{colors_ptr} + 64], ymm10",
            "vmovdqa [{colors_ptr} + 96], ymm14",
            "add {colors_ptr}, 128",  // colors_ptr += 128

            // Store indices
            "vmovdqa [{indices_ptr}], ymm3",
            "vmovdqa [{indices_ptr} + 32], ymm7",
            "vmovdqa [{indices_ptr} + 64], ymm11",
            "vmovdqa [{indices_ptr} + 96], ymm15",
            "add {indices_ptr}, 128",  // indices_ptr += 128

            // Compare against end address and loop if not done
            "cmp {src_ptr}, {end}",
            "jb 2b",

            // Clear YMM registers
            "vzeroupper",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = inout(reg) len,
            indices_ptr = out(reg) _,
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::dxt1::testutils::allocate_align_64;
    use crate::raw::transform::tests::*;
    use rstest::rstest;

    type PermuteFn = unsafe fn(*const u8, *mut u8, usize);

    #[cfg(target_arch = "x86_64")]
    fn get_implementations() -> [(PermuteFn, &'static str); 8] {
        [
            (shuffle_permute as PermuteFn, "shuffle_permute"),
            (
                shuffle_permute_unroll_2 as PermuteFn,
                "shuffle_permute unroll 2",
            ),
            (
                shuffle_permute_unroll_4 as PermuteFn,
                "shuffle_permute unroll 4",
            ),
            (permute as PermuteFn, "permute"),
            (permute_unroll_2 as PermuteFn, "permute unroll 2"),
            (permute_unroll_4 as PermuteFn, "permute unroll 4"),
            (gather as PermuteFn, "gather"),
            (gather_unroll_4 as PermuteFn, "gather unroll 4"),
        ]
    }

    #[cfg(target_arch = "x86")]
    fn get_implementations() -> [(PermuteFn, &'static str); 5] {
        [
            (shuffle_permute as PermuteFn, "shuffle_permute"),
            (
                shuffle_permute_unroll_2 as PermuteFn,
                "shuffle_permute unroll 2",
            ),
            (permute as PermuteFn, "permute"),
            (permute_unroll_2 as PermuteFn, "permute unroll 2"),
            (gather as PermuteFn, "gather"),
        ]
    }

    #[rstest]
    #[case::min_size(32)] // 256 bytes - minimum size for unroll-4
    #[case::many_unrolls(512)] // 4KB - tests multiple unroll iterations
    #[case::large(2048)] // 16KB - large dataset
    fn test_avx2_implementations(#[case] num_blocks: usize) {
        let input = generate_dxt1_test_data(num_blocks);
        let mut output_expected = allocate_align_64(input.len());
        let mut output_test = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), output_expected.as_mut_slice());

        // Test each implementation
        for (permute_fn, impl_name) in get_implementations() {
            output_test.as_mut_slice().fill(0);
            unsafe {
                permute_fn(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }
            assert_eq!(
                output_expected.as_slice(),
                output_test.as_slice(),
                "{} implementation produced different results than reference for {} blocks.\n\
                First differing block will have predictable values:\n\
                Colors: Sequential 1-4 + (block_num * 4)\n\
                Indices: Sequential 128-131 + (block_num * 4)",
                impl_name,
                num_blocks
            );
        }
    }
}
