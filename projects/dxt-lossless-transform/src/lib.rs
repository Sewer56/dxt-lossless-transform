#![doc = include_str!("../../../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod raw;

/// Used by BC7, since that has unusual non-standard bit order.
#[allow(dead_code)]
pub(crate) mod util {
    pub(crate) mod msb_extract_bits;
    pub(crate) mod msb_insert_bits;
}

#[cfg(test)]
mod testutils {
    use safe_allocator_api::RawAlloc;
    use std::alloc::Layout;

    pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
        let layout = Layout::from_size_align(num_bytes, 64).unwrap();
        RawAlloc::new(layout).unwrap()
    }
}
