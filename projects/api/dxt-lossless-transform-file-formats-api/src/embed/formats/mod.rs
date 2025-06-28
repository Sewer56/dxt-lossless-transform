//! BCx format-specific embeddable implementations.

pub mod bc1;
pub mod bc2;
pub mod bc3;
pub mod bc7;

pub use bc1::EmbeddableBc1Details;
pub use bc2::EmbeddableBc2Details;
pub use bc3::EmbeddableBc3Details;
pub use bc7::EmbeddableBc7Details;
