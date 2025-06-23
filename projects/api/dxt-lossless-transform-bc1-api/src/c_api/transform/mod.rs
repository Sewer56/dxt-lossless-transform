//! C API bindings for BC1 transform operations.
//!
//! This module provides C-compatible FFI exports for BC1 transform functionality with a focus
//! on ABI stability and ease of use from C/C++ code. The API is designed to closely mirror
//! the Rust API structure while providing C-compatible interfaces.
//!
//! ## API Structure
//!
//! ### Manual Transform Builder
//! - [`manual_transform_builder::Dltbc1ManualTransformBuilder`] - Direct equivalent of Rust's [`crate::Bc1ManualTransformBuilder`]
//! - Allows precise control over transform parameters like decorrelation mode and color endpoint splitting
//! - Use when you know the optimal settings for your use case
//!
//! ### Auto Transform Builder  
//! - [`auto_transform_builder::Dltbc1AutoTransformBuilder`] - Direct equivalent of Rust's [`crate::Bc1AutoTransformBuilder`]
//! - Automatically finds optimal transform settings using a size estimator
//! - `dltbc1_AutoTransformBuilder_Transform` returns a configured manual builder (like Rust API)
//!
//! ## Usage Patterns
//!
//! ### Pattern 1: Auto Transform
//! ```c
//! // Create auto builder and configure
//! Dltbc1AutoTransformBuilder* auto_builder = dltbc1_new_AutoTransformBuilder(estimator);
//! dltbc1_AutoTransformBuilder_SetUseAllDecorrelationModes(auto_builder, false);
//!
//! // Transform and get configured manual builder
//! Dltbc1ManualTransformBuilder* manual_builder =
//!     dltbc1_AutoTransformBuilder_Transform(
//!         auto_builder, data, data_len, output, output_len);
//!
//! // Use manual builder for untransformation
//! dltbc1_ManualTransformBuilder_Untransform(/*...*/);
//!
//! // Cleanup
//! dltbc1_free_ManualTransformBuilder(manual_builder);
//! dltbc1_free_AutoTransformBuilder(auto_builder);
//! ```
//!
//! ### Pattern 2: Manual Transform
//! ```c
//! // Create and configure manual builder
//! Dltbc1ManualTransformBuilder* builder = dltbc1_new_ManualTransformBuilder();
//! dltbc1_ManualTransformBuilder_SetDecorrelationMode(builder, YCOCG_VARIANT_1);
//! dltbc1_ManualTransformBuilder_SetSplitColourEndpoints(builder, true);
//!
//! // Transform and untransform
//! dltbc1_ManualTransformBuilder_Transform(/*...*/);
//! dltbc1_ManualTransformBuilder_Untransform(/*...*/);
//!
//! // Cleanup
//! dltbc1_free_ManualTransformBuilder(builder);
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
//! in the core crate at `dxt_lossless_transform_bc1::c_api`. These functions may have
//! breaking changes between versions without major version bumps.
//!
//! **Production code should use the ABI-stable builder patterns above.**

// Builder modules (stable, recommended)
pub mod auto_transform_builder;
pub mod manual_transform_builder;
