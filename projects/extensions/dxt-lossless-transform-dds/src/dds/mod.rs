/// Shared constants between modules.
pub mod constants;

/// Determine if a file is a DDS file.
pub mod likely_dds;

/// Extract the pixel data from a DDS file.  
pub mod parse_dds;

/// C exports for DDS functionality.
#[cfg(feature = "c-exports")]
pub mod exports;

pub use likely_dds::*;
pub use parse_dds::*;
