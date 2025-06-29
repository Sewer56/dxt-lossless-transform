//! Core trait for file format transformation.

use crate::bundle::TransformBundle;
use crate::error::TransformResult;
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Core trait for file format transformation and untransformation.
///
/// This trait provides the fundamental operations for transforming file formats
/// when the format is already known. (Do not do unneeded format validation here.)
///
/// It focuses purely on the transformation operations without any format
/// detection responsibilities.
///
/// **Important**: Assume the data is untrusted. Make sure to do safety validation
/// (buffer sizes, bounds checking, etc.) to prevent buffer overflows or out-of-range reads,
/// especially when your code may be used in web applications or other security-sensitive contexts.
/// Such validation may be feature-gated but should be enabled by default.
///
/// ## Header Replacement Strategy
///
/// When implementing this trait, you'll need to decide what part of the file header to replace
/// with transform metadata:
///
/// - **What gets replaced**: Implementation-dependent (your choice)
/// - **Typical replacement**: Magic header/signature (first 4 bytes) with transform metadata
/// - **Example**: DDS handler replaces the 4-byte DDS magic (`"DDS "`) with transform metadata
/// - **Alternative approaches**: Some handlers may choose to preserve magic headers intact
/// - **Recommendation**: Generally recommended to write into magic header space to prevent
///   issues with clever developers writing custom data in unused file format areas
///
/// The transform metadata is 4 bytes, matching what many file formats use for their magic header size.
pub trait FileFormatHandler: Send + Sync {
    /// Transform the input buffer to output buffer using the provided transform bundle.
    ///
    /// The handler will:
    /// 1. Parse the header to obtain necessary information
    /// 2. Copy headers to output
    /// 3. Use the appropriate builder from the bundle based on detected BCx format
    /// 4. Embed transform details in the output header
    ///
    /// # Parameters
    ///
    /// - `input`: Input buffer containing the file data
    /// - `output`: Output buffer (must be same size as input)
    /// - `bundle`: Bundle containing transform builders for different BCx formats
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error if:
    /// - There's an error parsing the header (etc.)
    /// - No appropriate builder is provided in the bundle
    /// - Transform operation fails (e.g. invalid data, etc.)
    fn transform_bundle<T>(
        &self,
        input: &[u8],
        output: &mut [u8],
        bundle: &TransformBundle<T>,
    ) -> TransformResult<()>
    where
        T: SizeEstimationOperations,
        T::Error: Debug;

    /// Untransform the input buffer to output buffer.
    ///
    /// The handler will:
    /// 1. Extract transform details from the header
    /// 2. Restore the original file format header
    /// 3. Dispatch to the appropriate untransform function
    ///
    /// # Parameters
    ///
    /// - `input`: Input buffer containing transformed data
    /// - `output`: Output buffer (must be same size as input)
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error if:
    /// - The header is invalid or corrupted
    /// - Untransform operation fails
    fn untransform(&self, input: &[u8], output: &mut [u8]) -> TransformResult<()>;
}
