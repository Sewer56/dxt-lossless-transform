//! Core trait for file format handlers.

use crate::bundle::TransformBundle;
use crate::error::FileFormatResult;

/// Trait for handling specific file formats.
///
/// File format handlers are responsible for:
/// - Detecting if they can handle a given input (for known formats)
/// - Managing header embedding/restoration
/// - Coordinating transform operations with the appropriate builders
///
/// This trait focuses on performance when the file format is known.
/// For automatic format detection, see [`FileFormatDetection`].
pub trait FileFormatHandler: Send + Sync {
    /// Check if this handler can process the input data
    fn can_handle(&self, input: &[u8]) -> bool;

    /// Transform the input buffer to output buffer using the provided transform bundle.
    ///
    /// The handler will:
    /// 1. Validate the input format
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
    /// - The format is not supported
    /// - No appropriate builder is provided in the bundle
    /// - Transform operation fails
    fn transform_bundle(
        &self,
        input: &[u8],
        output: &mut [u8],
        bundle: &TransformBundle,
    ) -> FileFormatResult<()>;

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
    fn untransform(&self, input: &[u8], output: &mut [u8]) -> FileFormatResult<()>;
}

/// Trait for automatic file format detection.
///
/// **Intended for CLI tools/single files where file format is unknown.**
///
/// This trait provides methods for automatically detecting file formats when you don't
/// have prior knowledge of the format. This adds overhead and should be avoided in
/// performance-critical scenarios.
///
/// (Also may be a bit less reliable depending on implementation details, but in the first party
/// packages we strive for accuracy.)
///
/// ## When NOT to use this trait
///
/// - **Archive formats**: Store the exact format in archive metadata instead
/// - **Performance-critical applications**: Use direct format handlers when format is known
/// - **Embedded systems**: Where you control the file format
///
/// ## When to use this trait
///
/// - **CLI tools**: That need to process unknown file formats
/// - **File conversion utilities**: That operate on mixed file types
/// - **Interactive applications**: Where users provide arbitrary files
pub trait FileFormatDetection: FileFormatHandler {
    /// Check if this handler can process transformed data for untransform.
    ///
    /// This method is used to automatically detect which handler can process
    /// a transformed file when you don't have prior knowledge of the original format.
    /// It validates that restoring the original format results in a valid file.
    ///
    /// **Performance Note**: This method involves file parsing and validation,
    /// which adds overhead. Only use when format auto-detection is required.
    ///
    /// # Parameters
    ///
    /// - `input`: The transformed file data to analyze
    ///
    /// # Returns
    ///
    /// `true` if this handler can process the transformed data, `false` otherwise
    fn can_handle_untransform(&self, input: &[u8]) -> bool;

    /// Get the list of file extensions supported by this handler.
    ///
    /// Used to filter potential handlers based on file extensions when automatically
    /// detecting file formats, reducing false positives during format detection.
    ///
    /// # Returns
    ///
    /// A slice of supported file extensions (lowercase, without leading dot)
    /// An empty string in the slice indicates all extensions are supported.
    fn supported_extensions(&self) -> &[&str];
}
