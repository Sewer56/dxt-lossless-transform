//! # Stable API Re-exports
//!
//! This module contains stable re-exports of types from internal crates that have a risk of
//! changing in the future but must remain stable in this public API.
//!
//! ## Purpose
//!
//! The types in this module serve as a stable API boundary between the public API and the
//! internal implementation crates. While the internal implementation may evolve, break, or
//! change significantly, these re-exported types provide:
//!
//! - **API Stability**: Types here maintain backward compatibility across versions
//! - **Future-Proofing**: Insulates users from internal crate restructuring
//! - **Clear Contracts**: Well-documented, stable interfaces for public consumption
//!
//! ## Design Philosophy
//!
//! Types are re-exported here when:
//! 1. They are used in public APIs but defined in internal/unstable crates
//! 2. The internal implementation is likely to change or be refactored
//! 3. We need to provide API stability guarantees to users
//! 4. Direct dependency on internal crates would expose unstable interfaces
//!
//! ## Implementation Notes
//!
//! Each re-exported type typically includes:
//! - Conversion functions to/from the internal representation
//! - Complete API documentation independent of internal crates
//! - Stability guarantees and deprecation policies where applicable
//!
//! When internal types change, only the conversion functions need updating,
//! keeping the public API stable.

/// Color565-related stable re-exports.
///
/// Contains stable versions of types related to [`Color565`] operations that are
/// used in public APIs but defined in internal crates.
///
/// [`Color565`]: https://docs.rs/dxt-lossless-transform-common/latest/dxt_lossless_transform_common/color_565/struct.Color565.html
pub mod color_565;
