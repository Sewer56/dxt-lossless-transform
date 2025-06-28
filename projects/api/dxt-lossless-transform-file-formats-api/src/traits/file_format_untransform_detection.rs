//! Trait for file format detection during untransformation.

use crate::traits::file_format_handler::FileFormatHandler;

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
    /// replacement strategy details), detection relies on examining the remaining file structure.
    ///
    /// **Reliability warning**: This detection is less reliable than [`FileFormatDetection::can_handle`]
    /// because critical format identification information has been replaced with transform metadata.
    ///
    /// # Parameters
    ///
    /// - `input`: The transformed file data to analyze
    ///
    /// # Returns
    ///
    /// `true` if this handler can likely process the transformed data, `false` otherwise
    ///
    /// [`FileFormatDetection::can_handle`]: crate::traits::file_format_detection::FileFormatDetection::can_handle
    fn can_handle_untransform(&self, input: &[u8]) -> bool;

    /// Get the list of file extensions supported by this handler.
    ///
    /// Used to filter potential handlers based on file extensions when automatically
    /// detecting file formats during untransformation, reducing false positives.
    ///
    /// # Returns
    ///
    /// A slice of supported file extensions (lowercase, without leading dot)
    /// An empty string in the slice indicates all extensions are supported.
    ///
    /// # Remarks
    ///
    /// This is used to reduce the false positive rate during format detection;
    /// it's better to be safe than sorry.
    fn supported_extensions(&self) -> &[&str];
}
