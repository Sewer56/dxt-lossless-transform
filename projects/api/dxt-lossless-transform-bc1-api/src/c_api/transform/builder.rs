//! C API for BC1 transform options builder.

use crate::{
    c_api::transform::transform_context::{Dltbc1TransformContext, get_context_mut},
    transform::builder::Bc1TransformOptionsBuilder,
};
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

/// Set the decorrelation mode for the context.
///
/// # Parameters
/// - `context`: The BC1 context to modify
/// - `mode`: The decorrelation mode to use
///
/// # Safety
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_SetDecorrelationMode(
    context: *mut Dltbc1TransformContext,
    mode: YCoCgVariant,
) {
    if context.is_null() {
        return;
    }

    let inner = unsafe { get_context_mut(context) };
    inner.builder = inner.builder.decorrelation_mode(mode);
}

/// Set whether to split colour endpoints for the context.
///
/// # Parameters
/// - `context`: The BC1 context to modify
/// - `split`: Whether to split colour endpoints
///
/// # Safety
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_SetSplitColourEndpoints(
    context: *mut Dltbc1TransformContext,
    split: bool,
) {
    if context.is_null() {
        return;
    }

    let inner = unsafe { get_context_mut(context) };
    inner.builder = inner.builder.split_colour_endpoints(split);
}

/// Reset the context to default settings.
///
/// # Parameters
/// - `context`: The BC1 context to reset
///
/// # Safety
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_ResetToDefaults(
    context: *mut Dltbc1TransformContext,
) {
    if context.is_null() {
        return;
    }

    let inner = unsafe { get_context_mut(context) };
    inner.builder = Bc1TransformOptionsBuilder::new();
}
