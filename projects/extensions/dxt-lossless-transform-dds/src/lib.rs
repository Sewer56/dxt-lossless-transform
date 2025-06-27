#![doc = include_str!("../README.MD")]
#![no_std]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
pub mod test_prelude;

pub mod dds;

// Re-export the DDS handler for convenient access
pub use dds::format_handler::DdsHandler;
