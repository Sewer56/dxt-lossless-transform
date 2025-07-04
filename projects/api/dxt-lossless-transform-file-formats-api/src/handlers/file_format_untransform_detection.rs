//! Trait for file format detection during untransformation.

use super::FileFormatHandler;

/// Trait for detecting file formats during untransformation.
///
/// This trait extends [`FileFormatHandler`] with the ability to detect the original
/// file format when untransforming back from a transformed file. At this stage,
/// your input is a file where some portion of the original file header has been replaced
/// with this library's metadata, making detection less reliable than during transformation.
///
/// **Important: This trait is only needed in specific scenarios.**
///
/// Usually you should work with files in archive formats, game engines, or other programs
/// where the format is known, making this trait unnecessary. (For example: Archive format
/// would store the format in the archive metadata.)
///
/// This should be used in cases where such data is not available; for example,
/// reading transformed files from disk using a CLI or GUI tool; or an archive format
/// without native support for transformed files.
///
/// **Detection challenges:**
///
/// The [`FileFormatHandler`] implementation determines what part of the file header gets
/// replaced with transform metadata (see its documentation for details).
/// This reduces the identifying information available for format detection.
///
/// **When implementing this trait, be extra careful:**
/// - This trait is typically used in software supporting many formats (false positives possible)
/// - Header replacement reduces detection reliability (less identifying information available)
/// - Add additional safeguards to ensure correct format detection (check if data looks sane)
pub trait FileFormatUntransformDetection: FileFormatHandler {
    /// Check if this handler can process transformed data for untransform.
    ///
    /// This method attempts to detect if a transformed file can be processed by this handler
    /// to recover the original format. Since some portion of the original file header
    /// was replaced with transform metadata (see [`FileFormatHandler`] documentation for
    /// replacement strategy details), detection relies on examining the remaining file structure
    /// and optionally the file extension.
    ///
    /// **Reliability warning**: This detection is less reliable than [`FileFormatDetection::can_handle`]
    /// because critical format identification information has been replaced with transform metadata.
    ///
    /// # Parameters
    ///
    /// - `input`: The transformed file data to analyze
    /// - `file_extension`: *Optional* file extension (lowercase, without leading dot).
    ///   This is [`None`] if it is unknown or file does not have an extension.
    ///
    /// # Returns
    ///
    /// `true` if this handler can likely process the transformed data, `false` otherwise
    ///
    /// # Implementation Guidelines
    ///
    /// Implementations should perform the fastest check first, whether that's a file
    /// extension check or parsing the remaining file structure.
    ///
    /// It is recommended to check both header and extension to avoid false positives.
    ///
    /// If your file format typically has no extension, you should explicitly check that the
    /// value is [`None`] as that is what you're expecting.
    ///
    /// [`FileFormatDetection::can_handle`]: crate::handlers::FileFormatDetection::can_handle
    fn can_handle_untransform(&self, input: &[u8], file_extension: Option<&str>) -> bool;
}
