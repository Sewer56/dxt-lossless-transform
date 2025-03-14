#![doc = include_str!("../../../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]

/// Used by BC7, since that has unusual non-standard bit order.
#[allow(dead_code)]
pub(crate) mod util {
    pub(crate) mod msb_extract_bits;
    pub(crate) mod msb_insert_bits;
}
