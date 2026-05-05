use crate::io::MoloneyIOBits;
pub trait ReadableMemory {
    type MemoryType;
    type MemoryError<'a>
    where
        Self: 'a;
    fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError<'_>>;
}

pub trait MutableMemory {
    type MemoryType;
    type MemoryError;
    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError>;
    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError>;
}

pub trait IOMappedMemory {
    const SIZE: usize;
    const RECEIVER_ADDRESS: usize = Self::SIZE - 4;
    const RECEIVER_STATUS_ADDRESS: usize = Self::SIZE - 3;
    const TRANSMITTER_ADDRESS: usize = Self::SIZE - 2;
    const TRANSMITTER_STATUS_ADDRESS: usize = Self::SIZE - 1;
}

/// Trait for I/O bit types that can be used with IOMemory
pub trait IOBitsType: Eq + From<Self::ValueType> + Into<Self::ValueType> + Default {
    type ValueType;

    fn on(&self) -> bool;
    fn done(&self) -> bool;
    #[allow(dead_code)]
    fn busy(&self) -> bool;
    #[allow(dead_code)]
    fn interupt(&self) -> bool;

    #[allow(dead_code)]
    fn with_on(self, value: bool) -> Self;
    fn with_done(self, value: bool) -> Self;
    fn with_busy(self, value: bool) -> Self;
    #[allow(dead_code)]
    fn with_interupt(self, value: bool) -> Self;

    // fn from_bits(bits: Self::ValueType) -> Self;

    /// Check if it's okay to read from the receiver/transmitter
    fn can_read(&self) -> bool {
        self.on() && self.done()
    }

    /// Check if it's okay to write to the transmitter
    fn can_write(&self) -> bool {
        self.on() && self.done()
    }

    // // / Create a zero value
    // fn zero() -> Self;
}

// Implement for IOBits

/// Trait for types that can be used as values in IOMemory.
///
/// Types implementing this trait can:
/// - Be converted to u8
/// - Be debugged/printed to a stream
/// - Optionally be converted from/to IOBits (for I/O operations)
pub trait IOValue<I>: Copy + From<I> {
    /// Convert this value to a u8
    fn as_u8(self) -> u8;
}

// Implement for u16 (the type used in the codebase)
impl IOValue<MoloneyIOBits> for u16 {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

// Implement IOValue for u16 with u16 as IOBits
impl IOValue<u16> for u16 {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

// Example: How to implement IOValue for a custom type
/*
impl IOValue for MyCustomType {
    fn as_u8(self) -> u8 {
        // Your custom conversion logic here
        // For example, if MyCustomType has a value field:
        self.value as u8
    }

    // If your custom type can convert to/from IOBits, implement these:
    fn from_iobits(bits: IOBits) -> Self {
        // Your conversion from IOBits
        MyCustomType { value: bits.into() }
    }

    fn to_iobits(self) -> IOBits {
        // Your conversion to IOBits
        self.value.into()
    }
}
*/
