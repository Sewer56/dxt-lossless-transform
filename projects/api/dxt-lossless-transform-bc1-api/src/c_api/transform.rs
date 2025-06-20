//! C API for BC1 transform and untransform operations.

use crate::c_api::transform_context::{
    Dltbc1TransformContext, get_detransform_details, get_transform_details,
};
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::transform::{transform_bc1_slice, untransform_bc1_slice};
use core::slice;

/// Transform BC1 data using a pre-allocated buffer.
///
/// # Parameters
/// - `input`: Pointer to input BC1 data
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer
/// - `output_len`: Length of output buffer (must be at least `input_len`)
/// - `context`: The BC1 context containing transform options
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `context` must be a valid pointer to a Dltbc1TransformContext
/// - Pointers must remain valid for the duration of the call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    context: *const Dltbc1TransformContext,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() || output.is_null() || context.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidLength);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get transform options from context
    let options = unsafe { get_transform_details(context) };

    // Perform transform
    let result = transform_bc1_slice(input_slice, output_slice, options);

    result.into()
}

/// Untransform BC1 data using a pre-allocated buffer.
///
/// # Parameters
/// - `input`: Pointer to transformed BC1 data
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer
/// - `output_len`: Length of output buffer (must be at least `input_len`)
/// - `context`: The BC1 context containing detransform options (must match original transform)
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
/// - Pointers must remain valid for the duration of the call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    context: *const Dltbc1TransformContext,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() || output.is_null() || context.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidLength);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get detransform options from context
    let options = unsafe { get_detransform_details(context) };

    // Perform untransform
    let result = untransform_bc1_slice(input_slice, output_slice, options);

    result.into()
}
