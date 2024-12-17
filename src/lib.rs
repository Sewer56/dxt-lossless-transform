#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "c-exports")]
pub mod exports;

pub mod raw;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
