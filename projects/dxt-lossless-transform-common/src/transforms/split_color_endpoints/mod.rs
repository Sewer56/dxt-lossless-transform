//! # Splitting the Colour Endpoints
//!
//! Each BC1 texture has 2 colour endpoints, `color0` and `color1`.
//! It is sometimes beneficial to separate these, i.e. store them separately.
//!
//! Take our optimized layout from earlier:
//!
//! ```text
//! +-------+-------+-------+
//! |C0  C1 |C0  C1 |C0  C1 |
//! +-------+-------+-------+
//! ```
//!
//! We can split the colour endpoints
//!
//! ```text
//! +-------+-------+ +-------+-------+
//! |C0  C0 |C0  C0 | |C1  C1 |C1  C1 |
//! +-------+-------+ +-------+-------+
//! ```

pub mod portable;
use crate::color_565::Color565;
use crate::transforms::split_color_endpoints::portable::portable_32;

/// Splits the colour endpoints
///
/// # Arguments
///
/// * `colors` - Pointer to the input array of colors
/// * `colors_out` - Pointer to the output array of colors
/// * `colors_len` - Number of bytes in the input array.
///
/// # Safety
///
/// - `colors` must be valid for reads of `colors_len` bytes
/// - `colors_out` must be valid for writes of `colors_len` bytes
/// - `colors_len` must be a multiple of 4
///
/// # Remarks
///
/// For performance it's recommended that colors and colors_out are 32-byte aligned.
/// As this method may use a SIMD implementation.
pub unsafe fn split_color_endpoints(
    colors: *const Color565,
    colors_out: *mut Color565,
    colors_len: usize,
) {
    portable_32(colors as *const u8, colors_out as *mut u8, colors_len);
}
