pub mod portable;
pub use portable::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use sse2::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_bc3_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    // SSE2 is required by x86-64, so no check needed
    // On i686, this is slower, so skipped.
    #[cfg(target_arch = "x86_64")]
    {
        if len % 64 == 0 {
            sse2::u64_detransform_sse2(input_ptr, output_ptr, len);
        }
    }

    if len % 32 == 0 {
        #[cfg(target_arch = "x86_64")]
        {
            u64_detransform(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86")]
        {
            u32_detransform_v2(input_ptr, output_ptr, len);
            return;
        }
    }

    u32_detransform(input_ptr, output_ptr, len);
}

/// Transform bc3 data from separated alpha/color/index format back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_blocks_bc3(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        untransform_bc3_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32_detransform(input_ptr, output_ptr, len)
    }
}

#[cfg(test)]
mod tests {
    use crate::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    // Helper to generate test data of specified size (in blocks)
    pub(crate) fn generate_bc3_transformed_test_data(num_blocks: usize) -> RawAlloc {
        let mut data = allocate_align_64(num_blocks * 16);
        let mut data_ptr = data.as_mut_ptr();

        let num_alpha_bytes = data.len() * 2 / 16;
        let num_alpha_bits = data.len() * 6 / 16;
        let num_colors = data.len() * 4 / 16;
        let num_indices = data.len() * 4 / 16;
        unsafe {
            // First the alpha bytes.
            for id in 0..num_alpha_bytes {
                data_ptr.write(id as u8);
                data_ptr = data_ptr.add(1);
            }

            // Now the alpha indices.
            for id in 0..num_alpha_bits {
                data_ptr.write((id + 32) as u8);
                data_ptr = data_ptr.add(1);
            }

            // First the colors.
            for id in 0..num_colors {
                data_ptr.write((id + 128) as u8);
                data_ptr = data_ptr.add(1);
            }

            // Now the indices.
            for id in 0..num_indices {
                data_ptr.write((id + 192) as u8);
                data_ptr = data_ptr.add(1);
            }
        }

        data
    }

    #[test]
    fn validate_bc3_test_data_generator() {
        let expected: Vec<u8> = vec![
            // alpha bytes
            0, 1, 2, 3, 4, 5, // block 1 - 3
            // alpha indices
            32, 33, 34, 35, 36, 37, // block 1
            38, 39, 40, 41, 42, 43, // block 2
            44, 45, 46, 47, 48, 49, // block 3
            // colours
            128, 129, 130, 131, // block 1
            132, 133, 134, 135, // block 2
            136, 137, 138, 139, // block 3
            // indices
            0xC0, 0xC1, 0xC2, 0xC3, // block 1
            0xC4, 0xC5, 0xC6, 0xC7, // block 2
            0xC8, 0xC9, 0xCA, 0xCB, // block 3
        ];
        let output = generate_bc3_transformed_test_data(3);
        assert_eq!(output.as_slice(), expected.as_slice());
    }
}
