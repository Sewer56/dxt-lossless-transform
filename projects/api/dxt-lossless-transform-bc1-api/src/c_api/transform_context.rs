//! Transform context management for BC1 C API.
//!
//! This module provides an opaque transform context type that stores BC1 transform
//! configuration. The context is per-thread and must be explicitly freed.

use core::ptr;

use crate::transform_options_builder::Bc1TransformOptionsBuilder;
use dxt_lossless_transform_bc1::{Bc1DetransformDetails, Bc1TransformDetails};

/// Opaque transform context type for BC1 transform operations.
///
/// This context stores the current transform configuration and must be:
///
/// - Created with [`dltbc1_transform_context_create()`]
/// - Modified using the builder functions
/// - Passed to transform operations
/// - Freed with [`dltbc1_transform_context_free()`] when no longer needed
///
/// The context is NOT thread-safe and should not be shared between threads.
/// Each thread should create its own context.
#[repr(C)]
pub struct Dltbc1TransformContext {
    // Private field to ensure it's opaque
    _private: [u8; 0],
}

/// Internal representation of the transform context
pub(crate) struct Dltbc1TransformContextInner {
    pub(crate) builder: Bc1TransformOptionsBuilder,
}

/// Create a new BC1 transform context with default settings.
///
/// The returned context must be freed with [`dltbc1_transform_context_free()`] when no longer needed.
///
/// # Returns
/// A pointer to a newly allocated transform context, or null if allocation fails.
#[unsafe(no_mangle)]
pub extern "C" fn dltbc1_transform_context_create() -> *mut Dltbc1TransformContext {
    let inner = Box::new(Dltbc1TransformContextInner {
        builder: Bc1TransformOptionsBuilder::new(),
    });

    Box::into_raw(inner) as *mut Dltbc1TransformContext
}

/// Free a BC1 transform context.
///
/// # Safety
/// - `context` must be a valid pointer returned by [`dltbc1_transform_context_create()`]
/// - `context` must not have been freed already
/// - After calling this function, `context` becomes invalid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_transform_context_free(context: *mut Dltbc1TransformContext) {
    if !context.is_null() {
        unsafe {
            drop(Box::from_raw(context as *mut Dltbc1TransformContextInner));
        }
    }
}

/// Clone a BC1 transform context.
///
/// Creates a new context with the same settings as the source context.
/// The returned context must be freed independently.
///
/// # Safety
/// - `context` must be a valid pointer to a Dltbc1TransformContext
///
/// # Returns
/// A pointer to a newly allocated transform context with the same settings, or null if allocation fails.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_transform_context_clone(
    context: *const Dltbc1TransformContext,
) -> *mut Dltbc1TransformContext {
    if context.is_null() {
        return ptr::null_mut();
    }

    let inner = unsafe { &*(context as *const Dltbc1TransformContextInner) };
    let cloned = Box::new(Dltbc1TransformContextInner {
        builder: inner.builder,
    });

    Box::into_raw(cloned) as *mut Dltbc1TransformContext
}

/// Get the transform details from a transform context.
///
/// # Safety
/// - `context` must be a valid pointer to a Dltbc1TransformContext
pub(crate) unsafe fn get_transform_details(
    context: *const Dltbc1TransformContext,
) -> Bc1TransformDetails {
    debug_assert!(!context.is_null());
    let inner = unsafe { &*(context as *const Dltbc1TransformContextInner) };
    inner.builder.build()
}

/// Get the detransform details from a transform context.
///
/// # Safety
/// - `context` must be a valid pointer to a Dltbc1TransformContext
pub(crate) unsafe fn get_detransform_details(
    context: *const Dltbc1TransformContext,
) -> Bc1DetransformDetails {
    let transform_details = unsafe { get_transform_details(context) };
    transform_details.into()
}

/// Get mutable access to the inner transform context.
///
/// # Safety
/// - `context` must be a valid pointer to a Dltbc1TransformContext
pub(crate) unsafe fn get_context_mut(
    context: *mut Dltbc1TransformContext,
) -> &'static mut Dltbc1TransformContextInner {
    debug_assert!(!context.is_null());
    unsafe { &mut *(context as *mut Dltbc1TransformContextInner) }
}
