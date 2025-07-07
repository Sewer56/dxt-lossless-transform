//! C API bindings for BC2 transform operations.
//!
//! This module provides C-compatible FFI exports for BC2 transform functionality with a focus
//! on ABI stability and ease of use from C/C++ code. The API is designed to closely mirror
//! the Rust API structure while providing C-compatible interfaces.
//!
//! ## API Structure
//!
//! ### Manual Transform Builder
//! - [`manual_transform_builder::Dltbc2ManualTransformBuilder`] - Direct equivalent of Rust's [`crate::Bc2ManualTransformBuilder`]
//! - Allows precise control over transform parameters like decorrelation mode and color endpoint splitting
//! - Use when you know the optimal settings for your use case
//!
//! ### Auto Transform Builder  
//! - [`auto_transform_builder::Dltbc2AutoTransformBuilder`] - Direct equivalent of Rust's [`crate::Bc2AutoTransformBuilder`]
//! - Automatically finds optimal transform settings using a size estimator
//! - [`dltbc2_AutoTransformBuilder_Transform`] returns a configured manual builder (like Rust API)
//!
//! [`dltbc2_AutoTransformBuilder_Transform`]: crate::c_api::transform::auto_transform_builder::dltbc2_AutoTransformBuilder_Transform
//!
//! ## Usage Patterns
//!
//! ### Pattern 1: Auto Transform
//! ```c
//! // Create auto builder and configure
//! Dltbc2AutoTransformBuilder* auto_builder = dltbc2_new_AutoTransformBuilder(estimator);
//! dltbc2_AutoTransformBuilder_SetUseAllDecorrelationModes(auto_builder, false);
//!
//! // Transform and get configured manual builder
//! Dltbc2ManualTransformBuilder* manual_builder =
//!     dltbc2_AutoTransformBuilder_Transform(
//!         auto_builder, data, data_len, output, output_len);
//!
//! // Use manual builder for untransformation
//! dltbc2_ManualTransformBuilder_Untransform(/*...*/);
//!
//! // Cleanup
//! dltbc2_free_ManualTransformBuilder(manual_builder);
//! dltbc2_free_AutoTransformBuilder(auto_builder);
//! ```
//!
//! ### Pattern 2: Manual Transform
//! ```c
//! // Create and configure manual builder
//! Dltbc2ManualTransformBuilder* builder = dltbc2_new_ManualTransformBuilder();
//! dltbc2_ManualTransformBuilder_SetDecorrelationMode(builder, YCOCG_VARIANT_1);
//! dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(builder, true);
//!
//! // Transform and untransform
//! dltbc2_ManualTransformBuilder_Transform(/*...*/);
//! dltbc2_ManualTransformBuilder_Untransform(/*...*/);
//!
//! // Cleanup
//! dltbc2_free_ManualTransformBuilder(builder);
//! ```
//!
//! ## Builder Modules (ABI-Stable)
//!
//! - [`auto_transform_builder`] - Builder pattern for automatic optimization settings
//! - [`manual_transform_builder`] - Builder pattern for manual transform configuration
//!
//! ## Unstable Functions (ABI-Unstable)
//!
//! For advanced users requiring maximum performance, unstable functions are available
//! in the core crate at `dxt_lossless_transform_bc2::c_api`. These functions may have
//! breaking changes between versions without major version bumps.
//!
//! **Production code should use the ABI-stable builder patterns above.**

// Builder modules (stable, recommended)
pub mod auto_transform_builder;
pub mod manual_transform_builder;
