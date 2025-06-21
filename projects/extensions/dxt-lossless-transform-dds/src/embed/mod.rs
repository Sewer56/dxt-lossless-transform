//! Embeds information about the various transforms into the DDS header.
//!
//! This module contains the code to pack information about how a file was transformed into the DDS
//! header of each file; such that the file can be detransformed later.
//!
//! # Explanation
//!
//! Each DDS starts with a 'magic' header, which is 4 bytes long.
//! This header is defined as `DDS_MAGIC` in this library.
//!
//! If we know (from context) that the input file is a DDS file at load time, then said
//! header becomes insignificant. In which case, we can use those 4 bytes to store the information we need.
//!
//! This module is responsible for embedding this information into the DDS header.
//!
//! # Header Overwriting and Restoration
//!
//! **Important**: The embed functions **overwrite** the original DDS magic header with transform details.
//! The unembed functions **restore** the original DDS magic header after extracting the transform details.
//! This ensures that:
//!
//! 1. During transformation: The file contains embedded transform details
//! 2. After detransformation: The file is restored to a valid DDS state
//!
//! The embedding process is designed to be reversible, maintaining file integrity.

pub mod bc1;

// Re-export the functions from BC1 module
pub use bc1::{embed_bc1_details, unembed_bc1_details};

// Re-export types from the API crate
pub use dxt_lossless_transform_api_common::embed::{EmbedError, TransformHeader};
pub use dxt_lossless_transform_bc1_api::embed::EmbeddableBc1Details;

// Re-export transform details from the BC1 transform crate
pub use dxt_lossless_transform_bc1::Bc1DetransformDetails;
