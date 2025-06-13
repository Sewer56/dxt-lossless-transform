#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")),
    feature(stdarch_x86_avx512)
)]

use crate::transforms::{
    standard::{transform, transform_with_separate_pointers, untransform},
    with_recorrelate, with_split_colour, with_split_colour_and_recorr,
};
use dxt_lossless_transform_common::{
    color_565::{Color565, YCoCgVariant},
    transforms::split_565_color_endpoints::split_color_endpoints,
};
use experimental::normalize_blocks::ColorNormalizationMode;

pub mod determine_optimal_transform;
pub mod experimental;
pub mod transforms;
pub mod util;

/// The information about the BC1 transform that was just performed.
/// Each item transformed via [`transform_bc1`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass [`Bc1DetransformDetails`] to [`untransform_bc1`],
/// which can be obtained from this struct using the `into` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1TransformDetails {
    /// The color normalization mode that was used to normalize the data.
    pub color_normalization_mode: ColorNormalizationMode,

    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

/// Details required to detransform BC1 data.
///
/// This struct contains only the information needed to reverse the transform operation.
/// Note that color normalization is a preprocessing step that doesn't need to be reversed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1DetransformDetails {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

impl From<Bc1TransformDetails> for Bc1DetransformDetails {
    fn from(transform_details: Bc1TransformDetails) -> Self {
        Self {
            decorrelation_mode: transform_details.decorrelation_mode,
            split_colour_endpoints: transform_details.split_colour_endpoints,
        }
    }
}

impl Default for Bc1DetransformDetails {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl Default for Bc1TransformDetails {
    fn default() -> Self {
        // Best (on average) results, but of course not perfect, as is with brute-force method.
        Self {
            color_normalization_mode: ColorNormalizationMode::None,
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl Bc1TransformDetails {
    /// Returns an iterator over all possible combinations of [`Bc1TransformDetails`] values.
    ///
    /// This function generates all possible combinations by iterating through:
    /// - All [`ColorNormalizationMode`] variants
    /// - All [`YCoCgVariant`] variants  
    /// - Both `true` and `false` values for `split_colour_endpoints`
    ///
    /// The total number of combinations is:
    /// [`ColorNormalizationMode`] variants × [`YCoCgVariant`] variants × 2 bool values
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_bc1::Bc1TransformDetails;
    ///
    /// let all_combinations: Vec<_> = Bc1TransformDetails::all_combinations().collect();
    /// println!("Total combinations: {}", all_combinations.len());
    ///
    /// for details in Bc1TransformDetails::all_combinations() {
    ///     println!("{:?}", details);
    /// }
    /// ```
    #[cfg(not(tarpaulin_include))]
    pub fn all_combinations() -> impl Iterator<Item = Bc1TransformDetails> {
        ColorNormalizationMode::all_values()
            .iter()
            .flat_map(|color_mode| {
                YCoCgVariant::all_values()
                    .iter()
                    .flat_map(move |decorr_mode| {
                        [true, false]
                            .into_iter()
                            .map(move |split_endpoints| Bc1TransformDetails {
                                color_normalization_mode: *color_mode,
                                decorrelation_mode: *decorr_mode,
                                split_colour_endpoints: split_endpoints,
                            })
                    })
            })
    }
}

/// Transform BC1 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `work_ptr`: A pointer to a work buffer (used by function)
/// - `len`: The length of the input data in bytes (size of `input_ptr`, `output_ptr` and half size of `work_ptr`)
/// - `transform_options`: The transform options to use.
///   Obtained from [`determine_optimal_transform::determine_best_transform_details`] or
///   [`Bc1TransformDetails::default`] for less optimal result(s).
///
/// # Remarks
///
/// The transform is lossless, in the sense that each pixel will produce an identical value upon
/// decode, however, it is not guaranteed that after decode, the file will produce an identical hash.
///
/// `output_ptr` will be written to twice if normalization is used (it normally is).
/// This may have performance implications if `output_ptr` is a pointer to a memory mapped file
/// and amount of available memory is scarce. Outside of that, memory should be fairly unaffected.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - work_ptr must be valid for writes of len/2 bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    work_ptr: *mut u8,
    len: usize,
    transform_options: Bc1TransformDetails,
) {
    debug_assert!(len % 8 == 0);

    let has_split_colours = transform_options.split_colour_endpoints;

    if has_split_colours {
        // Split the blocks, colours to work area, indices to final destination.
        transform_with_separate_pointers(
            input_ptr,                           // from our input
            work_ptr as *mut u32,                // colours to go our work area
            output_ptr.add(len / 2) as *mut u32, // but the indices go to their final destination
            len,
        );

        // Split the colour endpoints, writing them to the final output buffer.
        split_color_endpoints(
            work_ptr as *const Color565,
            output_ptr as *mut Color565,
            len / 2,
        );

        // Decorrelate the colours in output buffer in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    } else if transform_options.decorrelation_mode == YCoCgVariant::None {
        // Only split blocks
        transform(input_ptr, output_ptr, len);
    } else {
        // Split the blocks directly into expected output.
        transform(input_ptr, output_ptr, len);

        // And if there's colour decorrelation, do it right now (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
}

/// Untransform BC1 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks).
///   Output from [`transform_bc1`].
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `work_ptr`: A pointer to a work buffer (used by function).
/// - `len`: The length of the input data in bytes
/// - `detransform_options`: A struct containing information about the transform that was originally performed.
///   Must match the settings used in [`transform_bc1`] function (excluding color normalization).
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
/// - work_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc1(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    detransform_options: Bc1DetransformDetails,
) {
    debug_assert!(len % 8 == 0);

    let has_split_colours = detransform_options.split_colour_endpoints;

    if has_split_colours {
        if detransform_options.decorrelation_mode == YCoCgVariant::None {
            // Optimized single-pass operation: unsplit split colors and combine with indices
            // directly into BC1 blocks, avoiding intermediate memory copies
            with_split_colour::untransform_with_split_colour(
                input_ptr as *const u16,              // color0 values
                input_ptr.add(len / 4) as *const u16, // color1 values
                input_ptr.add(len / 2) as *const u32, // indices
                output_ptr,                           // output BC1 blocks
                len / 8,                              // number of blocks (8 bytes per block)
            );
        } else {
            with_split_colour_and_recorr::untransform_with_split_colour_and_recorr(
                input_ptr as *const u16,              // color0 values
                input_ptr.add(len / 4) as *const u16, // color1 values
                input_ptr.add(len / 2) as *const u32, // indices
                output_ptr,                           // output BC1 blocks
                len / 8,                              // number of blocks (8 bytes per block)
                detransform_options.decorrelation_mode,
            );
        }
    } else if detransform_options.decorrelation_mode == YCoCgVariant::None {
        // Only split blocks.
        untransform(input_ptr, output_ptr, len);
    } else {
        // Unsplit blocks + decorrelate.
        with_recorrelate::untransform_with_recorrelate(
            input_ptr,
            output_ptr,
            len,
            detransform_options.decorrelation_mode,
        );
    }
}

/// Common test prelude for avoiding duplicate imports in test modules
#[cfg(test)]
pub mod test_prelude;
