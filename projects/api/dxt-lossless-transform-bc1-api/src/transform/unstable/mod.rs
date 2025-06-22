//! ABI-unstable BC1 transform functions.
//!
//! **⚠️ ABI Instability Warning**: All functions in this module may have breaking changes
//! between library versions without major version bumps. The structures and function
//! signatures are subject to change as the library evolves.
//!
//! ## Why Use These Functions?
//!
//! These functions provide maximum performance by avoiding builder pattern overhead.
//! They accept settings structs directly and can be useful in performance-critical
//! inner loops where every allocation matters.
//!
//! ## Why Are They Unstable?
//!
//! The settings structs and function signatures may evolve as new transform options
//! are added or existing ones are modified. This allows the library to improve
//! without being constrained by backwards compatibility.
//!
//! ## Recommended Alternative
//!
//! For production code, use the ABI-stable builder patterns instead:
//! - [`crate::Bc1EstimateSettingsBuilder`] for automatic optimization
//! - [`crate::Bc1TransformSettingsBuilder`] for manual configuration
//!
//! ## Migration Path
//!
//! If you're using these functions and experience breaking changes:
//! 1. Update your code to use the new signatures, or
//! 2. Switch to the stable builder patterns for long-term compatibility

// Individual modules for different functionality
pub mod transform_auto;
pub mod transform_with_settings;

// Re-export the main functions
pub use transform_auto::transform_bc1_auto;
pub use transform_with_settings::{transform_bc1_with_settings, untransform_bc1_with_settings};
