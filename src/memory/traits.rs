use std::fmt::Debug;

pub trait ReadableMemory<const SIZE: usize> {
    type MemoryType;
    type MemoryError;
    // const SIZE: usize;
    fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError>;
}

pub trait MutableMemory<const SIZE: usize>: ReadableMemory<SIZE> {
    //TODO: Split into readable and writeable traits
    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError>;
    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError>;
}

// pub trait IOMappedMemory<
//     const SIZE: usize,

// >: MutableMemory<SIZE>
// {

// }

/// Trait for I/O bit types that can be used with IOMemory
pub trait IOBitsType<T: IOMemoryType>
where
    Self: Eq + From<T> + Into<T> + Default,
{
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

pub trait IOMemoryType: Copy + Debug + Into<u8> + From<u8> + Default {}

// Implement for u16 (the type used in the codebase)
// impl IOMemoryType for u16 {
//     fn as_byte(self) -> u8 {
//         self as u8
//     }

//     fn from_byte(value: u8) -> Self {
//         value as Self
//     }
// }
