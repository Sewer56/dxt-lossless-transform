//! Unstable BC1 transform C API functions.
//!
//! **⚠️ ABI Instability Warning**: All functions in this module may have breaking changes
//! between library versions without major version bumps. These functions accept
//! structs directly for maximum performance but sacrifice ABI stability.
//!
//! For production code, use the ABI-stable builder patterns instead:
//! - [`super::auto_transform_builder`] for automatic optimization
//! - [`super::manual_transform_builder`] for manual configuration
//!
//! ## Migration Path
//!
//! If you're currently using these unstable functions and want to migrate to stable APIs:
//!
//! - Replace direct calls with builder patterns from the parent modules
//! - The stable APIs provide the same functionality with guaranteed ABI compatibility
//! - Performance difference is minimal in most use cases

// Individual modules for different functionality
pub mod transform_auto;
pub mod transform_with_settings;

// Note: We don't re-export the functions here to avoid encouraging their use
// Users must explicitly import from the submodules if they want to use unstable APIs
