//! Transform-related C API functions for BC1 operations.
//!
//! This module contains several submodules providing different approaches to BC1 transformation:
//!
//! ## ABI-Stable Modules (Recommended for Production)
//! - [`estimate_settings_builder`] - Builder pattern for automatic optimization settings
//! - [`transform_settings_builder`] - Builder pattern for manual transform configuration  
//! - [`transform_context`] - Context management for storing transform settings
//!
//! ## ABI-Unstable Modules (Maximum Performance)
//! - [`transform_auto`] - Direct automatic transformation with unstable structs
//! - [`transform_with_settings`] - Direct transformation with explicit unstable settings
//!
//! **Recommendation**: Use the ABI-stable modules for production code to ensure
//! compatibility across library versions. The ABI-unstable modules are provided for
//! performance-critical scenarios where you can accept potential breaking changes.

pub mod estimate_settings_builder;
pub mod transform_auto;
pub mod transform_context;
pub mod transform_settings_builder;
pub mod transform_with_settings;
