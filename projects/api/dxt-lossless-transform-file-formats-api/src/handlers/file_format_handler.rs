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
/// ## Implementation Checklist
///
/// Follow these steps when implementing transform and untransform operations:
///
/// **Reference Implementation**: See the DDS handler implementation in the `dxt-lossless-transform-dds` crate
/// for a complete example of how to implement this trait following all the recommended patterns.
///
/// **Note**: While this checklist provides a general framework, some steps may vary depending on the specific
/// file format. For example, the location of the magic header, header structure, and safety requirements
/// for metadata embedding may differ between formats.
///
/// ### Transform Bundle Implementation Steps
///
/// ✅ **1. Buffer Size Validation**
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::*;
///
/// # fn example(input: &[u8], output: &mut [u8]) -> TransformResult<()> {
/// if output.len() < input.len() {
///     return Err(FormatHandlerError::OutputBufferTooSmall {
///         required: input.len(),
///         actual: output.len()
///     }.into());
/// }
/// Ok(())
/// # }
/// ```
///
/// ✅ **2. Parse File Header**
/// - Parse the original file header to extract format information
/// - Return `FormatHandlerError::InvalidInputFileHeader` if parsing fails
/// - Extract `data_offset` (where texture data starts) and calculate `data_length`
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::*;
///
/// // Example from DDS implementation approach:
/// use dxt_lossless_transform_dds::dds::parse_dds::{DdsInfo, DdsFormat, parse_dds};
///
/// # fn example(input: &[u8]) -> TransformResult<DdsInfo> {
/// // Example from DDS implementation approach:
/// let info = parse_dds(input)
///     .ok_or(FormatHandlerError::InvalidInputFileHeader)?;
///
/// let data_offset = info.data_offset as usize;
/// let data_length = info.data_length as usize;
///
/// Ok(info)
/// # }
/// ```
///
/// ✅ **3. Validate Texture Data Length**
///
/// ⚠️ Don't blindly trust the file headers, validate there are enough bytes left in the file
/// to contain the claimed texture data size.
///
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::*;
///
/// # fn example(input: &[u8], data_offset: usize, data_length: usize) -> TransformResult<()> {
/// let total_required = data_offset + data_length;
/// if input.len() < total_required {
///     return Err(FormatHandlerError::InputTooShortForStatedTextureSize {
///         required: total_required,
///         actual: input.len(),
///     }.into());
/// }
/// Ok(())
/// # }
/// ```
///
/// ✅ **4. Copy File Headers**
/// ```no_run
/// # fn example(input: &[u8], output: &mut [u8], data_offset: usize) {
/// // Copy all header data up to where texture data begins
/// output[..data_offset].copy_from_slice(&input[..data_offset]);
/// # }
/// ```
///
/// ✅ **5. Transform Texture Data**
/// - Convert format to transform format
/// - Call [`crate::dispatch_transform`] with texture data only (not headers)
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::{*, embed::*};
/// use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
/// use core::fmt::Debug;
///
/// // Example from DDS implementation approach:
/// use dxt_lossless_transform_dds::dds::parse_dds::DdsFormat;
///
/// # fn example<T>(
/// #     transform_format: TransformFormat,
/// #     input: &[u8],
/// #     output: &mut [u8],
/// #     bundle: &TransformBundle<T>,
/// #     data_offset: usize,
/// #     data_length: usize
/// # ) -> TransformResult<TransformHeader>
/// # where
/// #     T: SizeEstimationOperations,
/// #     T::Error: Debug
/// # {
/// // Example from DDS implementation:
/// let header = dispatch_transform(
///     transform_format,
///     &input[data_offset..data_offset + data_length],
///     &mut output[data_offset..data_offset + data_length],
///     bundle,
/// )?;
/// Ok(header)
/// # }
/// ```
///
/// **Note**: The exact format conversion logic may vary by file format.
///
/// ✅ **6. Preserve Leftover Data**
///
/// 📓 Even if the file format itself has no extra data after the raw texture data, some
/// 'clever' programmers may try to put extra data anyway, so make sure to copy it.
///
/// ```no_run
/// # fn example(input: &[u8], output: &mut [u8], data_offset: usize, data_length: usize) {
/// // Copy any data that exists after the texture data
/// let leftover_start = data_offset + data_length;
/// if input.len() > leftover_start {
///     output[leftover_start..].copy_from_slice(&input[leftover_start..]);
/// }
/// # }
/// ```
///
/// ✅ **7. Embed Transform Metadata**
/// - Overwrite magic header/signature with transform metadata
/// - Use safe pointer operations with proper bounds checking
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::{*, embed::*};
///
/// # fn example(header: TransformHeader, output: &mut [u8]) {
/// // Example from DDS implementation:
/// // SAFETY: output.as_mut_ptr() is valid for writes of at least TRANSFORM_HEADER_SIZE bytes because:
/// // 1. We validated output.len() >= input.len() above
/// // 2. parse_dds succeeded, guaranteeing input has valid DDS structure (minimum 128 bytes)
/// // 3. Therefore output has at least 128 bytes, which is >= TRANSFORM_HEADER_SIZE bytes required for the header
/// unsafe {
///     header.write_to_ptr(output.as_mut_ptr());
/// }
/// # }
/// ```
///
/// **Note**: Safety requirements and embedding location may vary by file format.
///
/// ### Untransform Implementation Steps
///
/// ✅ **1. Buffer Size Validation**
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::{*, embed::*};
///
/// # fn example(input: &[u8], output: &mut [u8]) -> TransformResult<()> {
/// if input.len() < TRANSFORM_HEADER_SIZE {
///     return Err(FormatHandlerError::InputTooShort {
///         required: TRANSFORM_HEADER_SIZE,
///         actual: input.len()
///     }.into());
/// }
/// if output.len() < input.len() {
///     return Err(FormatHandlerError::OutputBufferTooSmall {
///         required: input.len(),
///         actual: output.len()
///     }.into());
/// }
/// Ok(())
/// # }
/// ```
///
/// ✅ **2. Read Transform Header**
/// - Extract transform metadata from the first 4 bytes
/// - Use safe pointer operations with proper bounds checking
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::{*, embed::*};
///
/// # fn example(input: &[u8]) -> TransformHeader {
/// // Example from DDS implementation:
/// // SAFETY: input.as_ptr() is valid for reads of at least TRANSFORM_HEADER_SIZE bytes because we validated
/// // input.len() >= TRANSFORM_HEADER_SIZE above. The input slice guarantees pointer validity.
/// let header = unsafe { TransformHeader::read_from_ptr(input.as_ptr()) };
/// header
/// # }
/// ```
///
/// ✅ **3. Parse File Header (Ignoring Magic)**
/// - Parse the file header while ignoring the overwritten magic bytes
/// - Return `FormatHandlerError::InvalidRestoredFileHeader` if parsing fails
/// - Extract `data_offset` and calculate `data_length`
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::*;
///
/// // Example from DDS implementation approach:
/// use dxt_lossless_transform_dds::dds::parse_dds::{DdsInfo, DdsFormat, parse_dds_ignore_magic};
///
/// # fn example(input: &[u8]) -> TransformResult<DdsInfo> {
/// // Example from DDS implementation approach:
/// // Parse header while ignoring the first 4 bytes (transform metadata)
/// let info = parse_dds_ignore_magic(input)
///     .ok_or(FormatHandlerError::InvalidRestoredFileHeader)?;
///
/// let data_offset = info.data_offset as usize;
/// let data_length = info.data_length as usize;
///
/// Ok(info)
/// # }
///
/// // Format-specific parsing function that ignores overwritten magic
/// # fn parse_file_format_ignore_magic(data: &[u8]) -> Option<DdsInfo> {
/// // Skip first 4 bytes (transform metadata), validate rest of structure
/// // This implementation is completely format-dependent
/// #     todo!("Implement based on your file format, ignoring first 4 bytes")
/// # }
/// ```
///
/// ✅ **4. Validate Texture Data Length**
///
/// ⚠️ Don't blindly trust the file headers, validate there are enough bytes left in the file
/// to contain the claimed texture data size.
///
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::*;
///
/// # fn example(input: &[u8], data_offset: usize, data_length: usize) -> TransformResult<()> {
///     let total_required = data_offset + data_length;
///     if input.len() < total_required {
///         return Err(FormatHandlerError::InputTooShortForStatedTextureSize {
///             required: total_required,
///             actual: input.len(),
///         }.into());
///     }
///     Ok(())
/// # }
/// ```
///
/// ✅ **5. Restore Original Magic Header**
/// ```no_run
/// # fn example(output: &mut [u8]) {
/// // Example: DDS magic header (format-specific)
/// const ORIGINAL_MAGIC: u32 = 0x44445320; // "DDS " in little-endian
/// output[0..4].copy_from_slice(&ORIGINAL_MAGIC.to_ne_bytes());
/// # }
/// ```
///
/// ✅ **6. Copy Remaining Headers**
/// ```no_run
/// # fn example(input: &[u8], output: &mut [u8], data_offset: usize) {
/// // Copy header data from byte 4 (after magic) to where texture data begins
/// output[4..data_offset].copy_from_slice(&input[4..data_offset]);
/// # }
/// ```
///
/// ✅ **7. Untransform Texture Data**
/// - Call [`crate::dispatch_untransform`] with texture data only (not headers)
/// ```no_run
/// use dxt_lossless_transform_file_formats_api::{*, embed::*};
///
/// # fn example(
/// #     header: TransformHeader,
/// #     input: &[u8],
/// #     output: &mut [u8],
/// #     data_offset: usize,
/// #     data_length: usize
/// # ) -> TransformResult<()> {
/// // Example from DDS implementation:
/// dispatch_untransform(
///     header,
///     &input[data_offset..data_offset + data_length],
///     &mut output[data_offset..data_offset + data_length],
/// )?;
/// Ok(())
/// # }
/// ```
///
/// ✅ **8. Preserve Leftover Data**
///
/// 📓 Even if the file format itself has no extra data after the raw texture data, some
/// 'clever' programmers may try to put extra data anyway, so make sure to copy it, same
/// way you did during the transform operation itself.
///
/// ```no_run
/// # fn example(input: &[u8], output: &mut [u8], data_offset: usize, data_length: usize) {
/// // Copy any data that exists after the texture data
/// let leftover_start = data_offset + data_length;
/// if input.len() > leftover_start {
///     output[leftover_start..].copy_from_slice(&input[leftover_start..]);
/// }
/// # }
/// ```
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
    /// **Follow the complete Transform Bundle Implementation Steps documented in the trait documentation.**
    ///
    /// The handler will:
    /// 1. Validate buffer sizes
    /// 2. Parse the header to obtain necessary information
    /// 3. Validate input buffer contains sufficient data for declared texture dimensions
    /// 4. Copy headers to output
    /// 5. Transform texture data using the appropriate builder from the bundle
    /// 6. Preserve any leftover data after texture data
    /// 7. Embed transform metadata in the output header
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
    /// - Output buffer is smaller than input buffer
    /// - Invalid or corrupted file header
    /// - Input buffer is too short for the texture dimensions declared in the header
    /// - No appropriate builder is provided in the bundle for the detected format
    /// - Transform operation fails (e.g. invalid texture data, etc.)
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
    /// **Follow the complete Untransform Implementation Steps documented in the trait documentation.**
    ///
    /// The handler will:
    /// 1. Validate buffer sizes
    /// 2. Read transform metadata from the header
    /// 3. Parse the file header (ignoring overwritten magic bytes)
    /// 4. Validate input buffer contains sufficient data for declared texture dimensions
    /// 5. Restore the original file format magic header
    /// 6. Copy remaining headers
    /// 7. Untransform texture data
    /// 8. Preserve any leftover data after texture data
    ///
    /// # Parameters
    ///
    /// - `input`: Input buffer containing transformed data
    /// - `output`: Output buffer (must be same size as input)
    ///
    /// # Returns
    ///
    /// Ok(()) on success, or an error if:
    /// - Input buffer is too short to contain transform header
    /// - Output buffer is smaller than input buffer
    /// - The restored file header is invalid or corrupted
    /// - Input buffer is too short for the texture dimensions declared in the header
    /// - Untransform operation fails
    fn untransform(&self, input: &[u8], output: &mut [u8]) -> TransformResult<()>;
}
