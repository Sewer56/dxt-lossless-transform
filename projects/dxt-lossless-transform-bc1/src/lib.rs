#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(avx512_target_feature))]
#![cfg_attr(feature = "nightly", feature(stdarch_x86_avx512))]

use dxt_lossless_transform_common::color_565::YCoCgVariant;
use normalize_blocks::ColorNormalizationMode;
use split_blocks::unsplit_blocks;
pub mod determine_optimal_transform;
pub mod normalize_blocks;
pub mod split_blocks;
pub mod util;

/// The information about the BC1 transform that was just performed.
/// Each item transformed via [`transform_bc1`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass the same instance to [`untransform_bc1`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bc1TransformDetails {
    /*
        Operations (per readme)
        1. Normalize
        2. Split (Always)
        3. Decorrelate
        4. Split Colours (Optional)
    */
    /// The color normalization mode that was used to normalize the data.
    pub color_normalization_mode: ColorNormalizationMode,

    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    pub split_colours: bool,
}

impl Default for Bc1TransformDetails {
    fn default() -> Self {
        // Best (on average) results, but of course not perfect, as is with brute-force method.
        Self {
            color_normalization_mode: ColorNormalizationMode::Color0Only,
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colours: true,
        }
    }
}

/// Transform BC1 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes
///
/// # Returns
///
/// A struct informing you how the file was transformed. You will need this to call the
/// [`untransform_bc1`] function.
///
/// # Remarks
///
/// The transform is lossless, in the sense that each pixel will produce an identical value upon
/// decode, however, it is not guaranteed that after decode, the file will produce an identical hash.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1(
    _input_ptr: *const u8,
    _output_ptr: *mut u8,
    len: usize,
) -> Bc1TransformDetails {
    debug_assert!(len % 8 == 0);

    /*
    let mut normalized = RawAlloc::new(Layout::from_size_align_unchecked(len, 64)).unwrap();
    normalize_blocks::normalize_blocks(
        input_ptr,
        normalized.as_mut_ptr(),
        len,
        ColorNormalizationMode::Color0Only,
    );
    split_blocks(normalized.as_ptr(), output_ptr, len);
    */
    Bc1TransformDetails::default()
}

/// Untransform BC1 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks).
///   Output from [`transform_bc1`].
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `_details`: A struct containing information about the transform that was performed
///   obtained from the original call to [`transform_bc1`].
///
/// # Remarks
///
/// The transform is lossless, in the sense that each pixel will produce an identical value upon
/// decode, however, it is not guaranteed that after decode, the file will produce an identical hash.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc1(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    _details: &Bc1TransformDetails,
) {
    debug_assert!(len % 8 == 0);
    unsplit_blocks(input_ptr, output_ptr, len);
}
