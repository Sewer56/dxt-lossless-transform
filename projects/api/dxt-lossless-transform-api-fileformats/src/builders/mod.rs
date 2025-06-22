//! File format aware builders that wrap stable API builders.

pub mod bc1;

pub use bc1::{
    Bc1EstimateBuilderWithEstimator,
    Bc1EstimateFileFormatExt,
    Bc1EstimateOptionsBuilderExt,
    // Extension traits for file format operations
    Bc1TransformFileFormatExt,
    TransformResult,
};
