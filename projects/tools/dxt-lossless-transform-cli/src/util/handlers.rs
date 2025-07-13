use dxt_lossless_transform_dds::DdsHandler;

/// Returns an array of all supported file format handlers.
///
/// This function provides a centralized way to access all available
/// file format handlers, avoiding the need to hardcode handler arrays
/// throughout the codebase.
pub fn all_handlers() -> [DdsHandler; 1] {
    [DdsHandler]
}
