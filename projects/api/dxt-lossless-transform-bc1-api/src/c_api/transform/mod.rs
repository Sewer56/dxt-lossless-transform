//! Transform-related C API functions for BC1 operations.
//!
//! This module contains several submodules providing different approaches to BC1 transformation.
//! For maximum compatibility and future-proofing, **use the ABI-stable builder patterns**
//! rather than direct transform functions.
//!
//! ## ABI-Stable Modules (Recommended for Production)
//!
//! - [`estimate_settings_builder`] - Builder pattern for automatic optimization settings
//! - [`transform_settings_builder`] - Builder pattern for manual transform configuration  
//!
//! ## ABI-Unstable Module (Maximum Performance)
//!
//! - [`unstable`] - Direct transformation with unstable structs for performance-critical scenarios
//!
//! **Recommendation**: Use the ABI-stable modules for production code to ensure
//! compatibility across library versions. The ABI-unstable module is provided for
//! performance-critical scenarios where you can accept potential breaking changes.

// ABI-stable modules (recommended for production)
pub mod estimate_settings_builder;
pub mod transform_settings_builder;

// ABI-unstable modules (advanced users only)
pub mod unstable;
