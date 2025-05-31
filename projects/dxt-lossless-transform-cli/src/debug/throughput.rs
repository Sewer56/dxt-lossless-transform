use bytesize::ByteSize;
use core::fmt;

/// A wrapper around [`ByteSize`] that represents throughput in bytes per second.
///
/// This type automatically appends "/s" to the display output to make it clear
/// that the value represents bytes per second throughput.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Throughput(pub ByteSize);

impl Throughput {
    /// Creates a new [`Throughput`] from bytes per second.
    pub fn from_bytes_per_sec(bytes_per_sec: u64) -> Self {
        Self(ByteSize(bytes_per_sec))
    }

    /// Returns the raw bytes per second value.
    pub fn bytes_per_sec(&self) -> u64 {
        self.0 .0
    }
}

impl fmt::Display for Throughput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/s", self.0)
    }
}

impl From<ByteSize> for Throughput {
    fn from(byte_size: ByteSize) -> Self {
        Self(byte_size)
    }
}

impl From<u64> for Throughput {
    fn from(bytes_per_sec: u64) -> Self {
        Self::from_bytes_per_sec(bytes_per_sec)
    }
}
