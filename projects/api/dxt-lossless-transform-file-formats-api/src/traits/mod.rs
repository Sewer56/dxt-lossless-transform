//! Core traits for file format handling.
//!
//! This module defines three complementary traits for implementing file format handlers
//! in the DXT lossless transform system. The traits are designed to separate different
//! responsibilities and use cases in the transformation pipeline.
//!
//! ## Architecture Overview
//!
//! The file format handling system operates in two layers:
//!
//! 1. **Format-specific operations**: Header parsing, metadata extraction, format validation
//! 2. **Transform operations**: Actual DXT compression/decompression using format-agnostic builders
//!
//! Handlers coordinate between these layers, embedding transform metadata in file headers
//! and managing the round-trip process (transform → embed metadata → extract metadata → untransform).
//!
//! ## Three-Trait System
//!
//! ### [`FileFormatHandler`]
//!
//! **Core transformation trait for known file formats.**
//!
//! This trait provides the fundamental transform and untransform operations when the file
//! format is already known. It contains no detection logic, just going from A to B.
//!
//! Use this when:
//!
//! - **Archive formats**: The exact known format is stored in archive metadata
//! - **Single format processing**: You're only working with one specific format
//! - **Performance-critical applications**: Detection overhead is unacceptable
//! - **Integrated systems** (e.g. Games): You control the file format
//!
//! ### [`FileFormatDetection`]
//!
//! **Transform-time format detection.**
//!
//! This trait extends [`FileFormatHandler`] with the ability to detect file formats
//! during transformation. Since original file headers (including magic numbers) are
//! intact, detection is reliable and fast.
//!
//! Use this for:
//! - **CLI tools**: Processing unknown file types provided by users
//! - **Interactive applications**: Where users drag-and-drop arbitrary files
//! - **File conversion utilities**: That handle mixed file types
//! - **Format auto-detection**: When input format is genuinely unknown
//!
//! ### [`FileFormatUntransformDetection`]
//!
//! **Untransform-time format detection (rarely needed).**
//!
//! This trait extends [`FileFormatHandler`] with the ability to detect the original
//! file format from transformed data. **This should rarely be implemented** - usually
//! you should work with archive formats, game engines, or other programs where the
//! format is already known.
//!
//! **Only implement this when absolutely necessary:**
//! - You're building multi-format utilities that handle unknown transformed files
//! - You're working with legacy data where metadata wasn't preserved
//! - You need recovery tools for completely unknown format information
//!
//! **Implementation warning**: Be extra careful - this is typically used in software
//! supporting many formats, creating high risk of false positives. Add extra
//! validation and safeguards to ensure correct format detection.
//!
//! ## Usage Guidelines
//!
//! ### **Known Format (Optimal Performance)**
//! ```ignore
//! // Archive or controlled environment
//! let dds_handler = DdsFormatHandler::new();
//! dds_handler.transform_bundle(&input, &mut output, &bundle)?;
//! ```
//!
//! ### **Unknown Format at Transform Time**
//! ```ignore
//! // CLI tool processing unknown input files
//! let handlers: Vec<Box<dyn FileFormatDetection>> = get_transform_handlers();
//! for handler in handlers {
//!     if handler.can_handle(&input_data) {
//!         handler.transform_bundle(&input, &mut output, &bundle)?;
//!         break;
//!     }
//! }
//! ```
//!
//! ### **Unknown Format at Untransform Time (rarely needed)**
//! ```ignore
//! // CAUTION: Only use when format information is completely unavailable
//! // Usually you should store format info in archives/metadata instead
//! let handlers: Vec<Box<dyn FileFormatUntransformDetection>> = get_untransform_handlers();
//! for handler in handlers {
//!     if handler.can_handle_untransform(&transformed_data) {
//!         handler.untransform(&transformed_data, &mut output)?;
//!         break;
//!     }
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! - **Known formats**: Use [`FileFormatHandler`] directly for optimal performance
//! - **Transform-time detection**: [`FileFormatDetection`] adds minimal overhead
//! - **Untransform-time detection**: Avoid [`FileFormatUntransformDetection`] - store format info instead
//! - **Archive formats**: Always store format information in metadata rather than relying on detection
//! - **Game engines/applications**: Control your formats - detection should be unnecessary

pub(crate) mod file_format_detection;
pub(crate) mod file_format_handler;
pub(crate) mod file_format_untransform_detection;

// Re-export the main traits for convenience
pub use file_format_detection::*;
pub use file_format_handler::*;
pub use file_format_untransform_detection::*;
