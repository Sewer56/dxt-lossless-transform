//! Convenient unaligned read/write operations for various pointer types.
//!
//! This module provides a trait-based solution to avoid explicit pointer casts
//! when performing unaligned reads and writes.

/// Trait providing convenient unaligned read/write operations for pointer types.
///
/// This trait eliminates the need for explicit casts when reading from or writing to
/// typed pointers (e.g., `*const u32`, `*mut u16`) by providing methods that handle
/// the casting internally.
pub trait UnalignedReadWrite {
    /// Reads a [`u16`] value at the given byte offset from this pointer.
    ///
    /// # Safety
    ///
    /// - The pointer must be valid for reads at the specified offset
    /// - The offset must not cause the read to go beyond valid memory
    unsafe fn read_u16_at(self, offset: usize) -> u16;

    /// Reads a [`u32`] value at the given byte offset from this pointer.
    ///
    /// # Safety
    ///
    /// - The pointer must be valid for reads at the specified offset
    /// - The offset must not cause the read to go beyond valid memory
    unsafe fn read_u32_at(self, offset: usize) -> u32;
}

/// Trait providing convenient unaligned write operations for mutable pointer types.
pub trait UnalignedWrite {
    /// Writes a [`u16`] value at the given byte offset to this pointer.
    ///
    /// # Safety
    ///
    /// - The pointer must be valid for writes at the specified offset
    /// - The offset must not cause the write to go beyond valid memory
    unsafe fn write_u16_at(self, offset: usize, value: u16);

    /// Writes a [`u32`] value at the given byte offset to this pointer.
    ///
    /// # Safety
    ///
    /// - The pointer must be valid for writes at the specified offset
    /// - The offset must not cause the write to go beyond valid memory
    unsafe fn write_u32_at(self, offset: usize, value: u32);
}

// Implementations for const pointers
impl<T> UnalignedReadWrite for *const T {
    #[inline(always)]
    unsafe fn read_u16_at(self, offset: usize) -> u16 {
        ((self as *const u8).add(offset) as *const u16).read_unaligned()
    }

    #[inline(always)]
    unsafe fn read_u32_at(self, offset: usize) -> u32 {
        ((self as *const u8).add(offset) as *const u32).read_unaligned()
    }
}

// Implementations for mutable pointers (both read and write)
impl<T> UnalignedReadWrite for *mut T {
    #[inline(always)]
    unsafe fn read_u16_at(self, offset: usize) -> u16 {
        ((self as *const u8).add(offset) as *const u16).read_unaligned()
    }

    #[inline(always)]
    unsafe fn read_u32_at(self, offset: usize) -> u32 {
        ((self as *const u8).add(offset) as *const u32).read_unaligned()
    }
}

impl<T> UnalignedWrite for *mut T {
    #[inline(always)]
    unsafe fn write_u16_at(self, offset: usize, value: u16) {
        ((self as *mut u8).add(offset) as *mut u16).write_unaligned(value);
    }

    #[inline(always)]
    unsafe fn write_u32_at(self, offset: usize, value: u32) {
        ((self as *mut u8).add(offset) as *mut u32).write_unaligned(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unaligned_operations() {
        unsafe {
            let mut buffer = [0u8; 16];
            let ptr = buffer.as_mut_ptr();
            
            // Test writing
            ptr.write_u16_at(0, 0x1234);
            ptr.write_u32_at(4, 0x12345678);
            
            // Test reading
            let value16 = ptr.read_u16_at(0);
            let value32 = ptr.read_u32_at(4);
            
            assert_eq!(value16, 0x1234);
            assert_eq!(value32, 0x12345678);
            
            // Test with typed pointers
            let u16_ptr = ptr as *mut u16;
            let u32_ptr = ptr as *mut u32;
            
            assert_eq!(u16_ptr.read_u16_at(0), 0x1234);
            assert_eq!(u32_ptr.read_u32_at(4), 0x12345678);
        }
    }
}