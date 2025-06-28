//! Trait for file format detection during transformation.

use crate::traits::FileFormatHandler;

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
    /// This method examines the file headers (including magic numbers) to determine
    /// if this handler can process the given input file. Since the original headers
    /// are intact, detection is reliable.
    ///
    /// # Parameters
    ///
    /// - `input`: The input file data to analyze
    ///
    /// # Returns
    ///
    /// `true` if this handler can process the input data, `false` otherwise
    fn can_handle(&self, input: &[u8]) -> bool;

    /// Get the list of file extensions supported by this handler.
    ///
    /// Used to filter potential handlers based on file extensions when automatically
    /// detecting file formats, reducing false positives during format detection.
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
