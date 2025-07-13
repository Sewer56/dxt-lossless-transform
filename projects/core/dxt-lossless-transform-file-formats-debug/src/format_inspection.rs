//! Format inspection trait for debug and analysis operations.
//!
//! This module provides a trait for determining the [`TransformFormat`] of texture files
//! without extracting the actual block data, useful for format detection and validation.

use dxt_lossless_transform_file_formats_api::{embed::TransformFormat, error::TransformResult};

use crate::TransformFormatFilter;

/// Trait for determining the [`TransformFormat`] of a texture file format.
///
/// This trait provides a lightweight way to identify the compression format
/// of a texture file without extracting the actual block data. It's useful
/// for format detection, validation, and filtering operations.
///
/// # Example Implementation
///
/// ```rust
/// use dxt_lossless_transform_file_formats_debug::{
///     FileFormatInspection, TransformFormatFilter,
/// };
/// use dxt_lossless_transform_file_formats_api::{
///     embed::TransformFormat,
///     error::{TransformResult, TransformError, FormatHandlerError},
/// };
///
/// // Example custom handler
/// struct MyFormatHandler;
///
/// impl FileFormatInspection for MyFormatHandler {
///     fn get_transform_format(
///         &self,
///         data: &[u8],
///         filter: TransformFormatFilter,
///     ) -> TransformResult<Option<TransformFormat>> {
///         // Parse your custom format header
///         if data.len() < 4 {
///             return Err(TransformError::FormatHandler(
///                 FormatHandlerError::UnknownFileFormat
///             ));
///         }
///         
///         // Determine format from header
///         let transform_format = TransformFormat::Bc1; // Example
///         
///         if !filter.accepts(transform_format) {
///             return Ok(None);
///         }
///         
///         Ok(Some(transform_format))
///     }
/// }
/// ```
pub trait FileFormatInspection {
    /// Get the [`TransformFormat`] of the texture file format.
    ///
    /// This method parses the file format header to determine the compression
    /// format without extracting the actual block data. If the file format
    /// doesn't match the specified filter, returns `Ok(None)`.
    ///
    /// # Parameters
    ///
    /// - `data`: The complete file data to inspect
    /// - `filter`: Filter specifying which block formats to accept
    ///
    /// # Returns
    ///
    /// - `Ok(Some(format))`: Successfully identified format matching the filter
    /// - `Ok(None)`: File format doesn't match the filter
    /// - `Err(error)`: File format is invalid or inspection failed
    fn get_transform_format(
        &self,
        data: &[u8],
        filter: TransformFormatFilter,
    ) -> TransformResult<Option<TransformFormat>>;
}
