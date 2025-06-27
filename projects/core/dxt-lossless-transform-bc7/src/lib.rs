#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![no_std]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

/// Used by BC7, since that has unusual non-standard bit order.
#[allow(dead_code)]
pub(crate) mod util {
    pub(crate) mod msb_extract_bits;
    pub(crate) mod msb_insert_bits;
}
