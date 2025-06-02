//! This module contains accelerated routines for performing certain combined detransformation steps.

pub(crate) mod split_and_decorrelate;
pub(crate) mod unsplit_split_colour_split_blocks;
pub(crate) use unsplit_split_colour_split_blocks::unsplit_split_colour_split_blocks;
