//! This module contains accelerated routines/intrinsics for performing combined detransformation steps
//! from the various formats that transformed data may be in.

pub(crate) mod standard;
pub(crate) mod with_recorrelate;
pub(crate) mod with_split_colour;
pub(crate) mod with_split_colour_and_recorr;
