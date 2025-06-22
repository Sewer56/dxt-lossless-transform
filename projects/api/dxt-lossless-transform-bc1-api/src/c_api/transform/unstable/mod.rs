//! ABI-unstable BC1 transform functions for C API.
//!
//! **⚠️ ABI Instability Warning**: All functions in this module accept ABI-unstable
//! structures which may change between versions without major version bumps.
//! Function signatures and struct layouts are subject to change as the library evolves.
//!
//! ## Why Use These Functions?
//!
//! These functions provide maximum performance by avoiding builder pattern overhead
//! and allowing direct struct manipulation. They are ideal for performance-critical
//! inner loops where every allocation and function call matters.
//!
//! ## Why Are They Unstable?
//!
//! The C structs and function signatures may evolve as new transform options
//! are added or existing ones are modified. This allows the library to improve
//! without being constrained by ABI backwards compatibility.
//!
//! ## Recommended Alternative
//!
//! For production code, use the ABI-stable builder patterns instead:
//! - [`super::estimate_settings_builder`] for automatic optimization
//! - [`super::transform_settings_builder`] for manual configuration
//!
//! ## Migration Path
//!
//! If you're using these functions and experience breaking changes:
//! 1. Update your code to use the new signatures, or
//! 2. Switch to the stable builder patterns for long-term compatibility

// Individual modules for different functionality
pub mod transform_auto;
pub mod transform_with_settings;

// Note: We don't re-export the functions here to avoid encouraging their use
// Users must explicitly import from the submodules if they want to use unstable APIs
