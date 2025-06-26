#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
pub mod dds;

// Re-export the DDS handler for convenient access
pub use dds::format_handler::DdsHandler;
