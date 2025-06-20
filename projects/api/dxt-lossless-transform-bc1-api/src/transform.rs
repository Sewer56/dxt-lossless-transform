//! BC1 transform and untransform operations.

use crate::error::Bc1Error;
use dxt_lossless_transform_bc1::{Bc1DetransformDetails, Bc1TransformDetails};
use dxt_lossless_transform_common::allocate::allocate_align_64;
use safe_allocator_api::RawAlloc;

/// Transform BC1 data in-place using the provided slices.
///
/// # Parameters
///
/// - `input`: The input BC1 data to transform
/// - `output`: The output buffer to write transformed data to  
/// - `options`: The transform options to use
///
/// # Returns
///
/// [`Ok`] on success, or an error if validation fails.
///
/// # Errors
///
/// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1Error::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use dxt_lossless_transform_bc1_api::transform_bc1_slice;
/// # use dxt_lossless_transform_bc1::Bc1TransformDetails;
/// let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
/// let mut output = vec![0u8; bc1_data.len()];
///
/// transform_bc1_slice(&bc1_data, &mut output, Bc1TransformDetails::default())?;
/// # Ok(())
/// # }
/// ```
pub fn transform_bc1_slice(
    input: &[u8],
    output: &mut [u8],
    options: Bc1TransformDetails,
) -> Result<(), Bc1Error> {
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1Error::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc1Error::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated all preconditions
    unsafe {
        dxt_lossless_transform_bc1::transform_bc1(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len(),
            options,
        );
    }

    Ok(())
}

/// Transform BC1 data and return a new allocated buffer.
///
/// This function allocates a new 64-byte aligned buffer using [`RawAlloc`] to avoid
/// zeroing memory and ensure optimal alignment for SIMD operations.
///
/// # Parameters
///
/// - `input`: The input BC1 data to transform
/// - `options`: The transform options to use
///
/// # Returns
///
/// A [`RawAlloc`] containing the transformed data.
///
/// # Errors
///
/// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1Error::AllocationFailed`] if memory allocation fails
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use dxt_lossless_transform_bc1_api::transform_bc1_allocating;
/// # use dxt_lossless_transform_bc1::Bc1TransformDetails;
/// let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
///
/// let transformed = transform_bc1_allocating(&bc1_data, Bc1TransformDetails::default())?;
/// // Use transformed.as_slice() to access the data
/// # Ok(())
/// # }
/// ```
pub fn transform_bc1_allocating(
    input: &[u8],
    options: Bc1TransformDetails,
) -> Result<RawAlloc, Bc1Error> {
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1Error::InvalidLength(input.len()));
    }

    // Allocate aligned output buffer
    let mut output = allocate_align_64(input.len())?;

    // Safety: We've validated all preconditions and allocated sufficient memory
    unsafe {
        dxt_lossless_transform_bc1::transform_bc1(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len(),
            options,
        );
    }

    Ok(output)
}

/// Untransform BC1 data in-place using the provided slices.
///
/// # Parameters
///
/// - `input`: The transformed BC1 data to restore
/// - `output`: The output buffer to write restored data to
/// - `options`: The detransform options (must match the original transform)
///
/// # Returns
///
/// [`Ok`] on success, or an error if validation fails.
///
/// # Errors
///
/// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1Error::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use dxt_lossless_transform_bc1_api::untransform_bc1_slice;
/// # use dxt_lossless_transform_bc1::Bc1DetransformDetails;
/// let transformed_data = vec![0u8; 8 * 100]; // 100 transformed BC1 blocks
/// let mut output = vec![0u8; transformed_data.len()];
/// # let detransform_options = Bc1DetransformDetails::default();
///
/// untransform_bc1_slice(&transformed_data, &mut output, detransform_options)?;
/// # Ok(())
/// # }
/// ```
pub fn untransform_bc1_slice(
    input: &[u8],
    output: &mut [u8],
    options: Bc1DetransformDetails,
) -> Result<(), Bc1Error> {
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1Error::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc1Error::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated all preconditions
    unsafe {
        dxt_lossless_transform_bc1::untransform_bc1(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len(),
            options,
        );
    }

    Ok(())
}

/// Untransform BC1 data and return a new allocated buffer.
///
/// This function allocates a new 64-byte aligned buffer using [`RawAlloc`] to avoid
/// zeroing memory and ensure optimal alignment for SIMD operations.
///
/// # Parameters
///
/// - `input`: The transformed BC1 data to restore  
/// - `options`: The detransform options (must match the original transform)
///
/// # Returns
///
/// A [`RawAlloc`] containing the restored data.
///
/// # Errors
///
/// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1Error::AllocationFailed`] if memory allocation fails
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use dxt_lossless_transform_bc1_api::untransform_bc1_allocating;
/// # use dxt_lossless_transform_bc1::Bc1DetransformDetails;
/// let transformed_data = vec![0u8; 8 * 100]; // 100 transformed BC1 blocks
/// # let detransform_options = Bc1DetransformDetails::default();
///
/// let restored = untransform_bc1_allocating(&transformed_data, detransform_options)?;
/// // Use restored.as_slice() to access the data
/// # Ok(())
/// # }
/// ```
pub fn untransform_bc1_allocating(
    input: &[u8],
    options: Bc1DetransformDetails,
) -> Result<RawAlloc, Bc1Error> {
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1Error::InvalidLength(input.len()));
    }

    // Allocate aligned output buffer
    let mut output = allocate_align_64(input.len())?;

    // Safety: We've validated all preconditions and allocated sufficient memory
    unsafe {
        dxt_lossless_transform_bc1::untransform_bc1(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len(),
            options,
        );
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_length() {
        let input = vec![0u8; 7]; // Not divisible by 8
        let mut output = vec![0u8; 8];

        let result = transform_bc1_slice(&input, &mut output, Bc1TransformDetails::default());
        assert!(matches!(result, Err(Bc1Error::InvalidLength(7))));
    }

    #[test]
    fn output_buffer_too_small() {
        let input = vec![0u8; 16]; // 2 blocks
        let mut output = vec![0u8; 8]; // Only 1 block

        let result = transform_bc1_slice(&input, &mut output, Bc1TransformDetails::default());
        assert!(matches!(
            result,
            Err(Bc1Error::OutputBufferTooSmall {
                needed: 16,
                actual: 8
            })
        ));
    }
}
