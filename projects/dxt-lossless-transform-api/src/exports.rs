#[repr(C)]
pub enum DdsFormat {
    NotADds,
    Unknown,
    /// a.k.a. DXT1
    BC1,
    /// a.k.a. DXT2/3
    BC2,
    /// a.k.a. DXT4/5
    BC3,
    BC7,
}

#[repr(C)]
pub struct DdsInfo {
    pub format: DdsFormat,
    pub data_offset: u8,
}

/// Determines if the given file represents a DDS texture.
/// This is done by checking the 'MAGIC' header, 'DDS ' at offset 0.
///
/// # Safety
///
/// - `ptr` must be valid for reads of `len` bytes
#[no_mangle]
pub unsafe extern "C" fn is_dds(ptr: *const u8, len: usize) -> bool {
    dxt_lossless_transform_utils::dds::is_dds(ptr, len)
}

/// Attempts to parse the data format of a DDS file from the given pointer and length.
///
/// # Safety
///
/// Any input which passes [`is_dds`] check should be a valid input;
/// but you do not need to explicitly call [`is_dds`], this function will return null
/// if the file is not a DDS.
///
/// - `ptr` must be valid for reads of `len` bytes
/// - `len` must accurately represent the length of the file
///
/// # Return
///
/// A [`DdsInfo`] structure. If the file is not a DDS then [`DdsFormat`] will be [`DdsFormat::NotADds`].
/// If the format is an unsupported one, then [`DdsFormat`] will be [`DdsFormat::Unknown`].
#[no_mangle]
pub unsafe extern "C" fn parse_dds(ptr: *const u8, len: usize) -> DdsInfo {
    if let Some(info) = dxt_lossless_transform_utils::dds::parse_dds(ptr, len) {
        DdsInfo {
            format: match info.format {
                dxt_lossless_transform_utils::dds::DdsFormat::Unknown => DdsFormat::Unknown,
                dxt_lossless_transform_utils::dds::DdsFormat::BC1 => DdsFormat::BC1,
                dxt_lossless_transform_utils::dds::DdsFormat::BC2 => DdsFormat::BC2,
                dxt_lossless_transform_utils::dds::DdsFormat::BC3 => DdsFormat::BC3,
                dxt_lossless_transform_utils::dds::DdsFormat::BC7 => DdsFormat::BC7,
            },
            data_offset: info.data_offset,
        }
    } else {
        DdsInfo {
            format: DdsFormat::NotADds,
            data_offset: 0,
        }
    }
}

/// Transforms data from the standard format to one that is more suitable for compression.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 8 (BC1 block size)
/// - `input_ptr` and `output_ptr` must be 32-byte aligned (for performance and required by some platforms).
/// - `format` must be a valid [`DdsFormat`]
#[no_mangle]
pub unsafe extern "C" fn transform_format(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    format: DdsFormat,
) {
    // Convert from C enum to internal enum
    let internal_format = match format {
        DdsFormat::NotADds | DdsFormat::Unknown => crate::DdsFormat::Unknown,
        DdsFormat::BC1 => crate::DdsFormat::BC1,
        DdsFormat::BC2 => crate::DdsFormat::BC2,
        DdsFormat::BC3 => crate::DdsFormat::BC3,
        DdsFormat::BC7 => crate::DdsFormat::BC7,
    };

    crate::transform_format(input_ptr, output_ptr, len, internal_format)
}

/// Untransforms data from a compression suitable one to the standard format.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 8 (BC1 block size)
/// - `input_ptr` and `output_ptr` must be 32-byte aligned (for performance and required by some platforms).
/// - `format` must be a valid [`DdsFormat`], and the same format passed to [`transform_format`].
#[no_mangle]
pub unsafe extern "C" fn untransform_format(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    format: DdsFormat,
) {
    // Convert from C enum to internal enum
    let internal_format = match format {
        DdsFormat::NotADds | DdsFormat::Unknown => {
            dxt_lossless_transform_utils::dds::DdsFormat::Unknown
        }
        DdsFormat::BC1 => crate::DdsFormat::BC1,
        DdsFormat::BC2 => crate::DdsFormat::BC2,
        DdsFormat::BC3 => crate::DdsFormat::BC3,
        DdsFormat::BC7 => crate::DdsFormat::BC7,
    };

    crate::untransform_format(input_ptr, output_ptr, len, internal_format)
}

/// Transform BC1 data from standard interleaved format to separated color/index format
/// to improve compression ratio.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 8 (BC1 block size)
/// - `input_ptr` and `output_ptr` must be 32-byte aligned (for performance and required by some platforms).
#[no_mangle]
pub unsafe extern "C" fn bc1_transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    crate::transform_bc1(input_ptr, output_ptr, len);
}

/// Transform BC1 data from separated color/index format back to standard interleaved format.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 8 (BC1 block size)
/// - `input_ptr` and `output_ptr` must be 32-byte aligned (for performance and required by some platforms).
#[no_mangle]
pub unsafe extern "C" fn bc1_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    crate::untransform_bc1(input_ptr, output_ptr, len);
}

/// Transform BC1 data from standard interleaved format to separated color/index format
/// to improve compression ratio.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 16 (BC2 block size)
/// - `input_ptr` and `output_ptr` must be 32-byte aligned (for performance and required by some platforms).
#[no_mangle]
pub unsafe extern "C" fn bc2_transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    crate::transform_bc2(input_ptr, output_ptr, len);
}

/// Transform BC1 data from separated color/index format back to standard interleaved format.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 16 (BC2 block size)
/// - `input_ptr` and `output_ptr` must be 32-byte aligned (for performance and required by some platforms).
#[no_mangle]
pub unsafe extern "C" fn bc2_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    crate::untransform_bc2(input_ptr, output_ptr, len);
}
