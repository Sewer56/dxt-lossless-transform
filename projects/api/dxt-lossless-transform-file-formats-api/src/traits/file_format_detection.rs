//! Trait for file format detection during transformation.

use crate::traits::file_format_handler::FileFormatHandler;

/// Trait for detecting file formats during transformation.
///
/// This trait extends [`FileFormatHandler`] with the ability to detect file formats
/// when transforming from the original format. At this stage, the original file headers
/// (including magic numbers) are intact, and the file is passed in unmodified.
///
/// Use this trait for:
/// - **CLI tools**: Processing unknown file types provided by users
/// - **Interactive applications**: Where users drag-and-drop arbitrary files
/// - **File conversion utilities**: That need to handle mixed file types
/// - **Automatic format detection**: When you don't know the input format ahead of time
///
/// ***Important***: Remember to be careful and perform the necessary safety validation
/// (buffer sizes, bounds checking, etc.) to prevent buffer overflows or out-of-range reads.
/// This code may be used in web applications or other security-sensitive contexts.
pub trait FileFormatDetection: FileFormatHandler {
    /// Check if this handler can process the input data.
    ///
    /// This method examines the file headers (including magic numbers) and optionally
    /// the file extension to determine if this handler can process the given input file.
    /// Since the original headers are intact, detection is reliable.
    ///
    /// # Parameters
    ///
    /// - `input`: The input file data to analyze
    /// - `file_extension`: *Optional* file extension (lowercase, without leading dot).
    ///   This is [`None`] if it is unknown or file does not have an extension.
    ///
    /// # Returns
    ///
    /// `true` if this handler can process the input data, `false` otherwise
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
    fn can_handle(&self, input: &[u8], file_extension: Option<&str>) -> bool;
}
