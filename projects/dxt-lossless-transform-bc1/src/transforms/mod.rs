//! This module contains accelerated routines/intrinsics for performing combined detransformation steps
//! from the various formats that transformed data may be in.

pub mod standard;
pub mod with_recorrelate;
pub mod with_split_colour;
pub mod with_split_colour_and_recorr;
