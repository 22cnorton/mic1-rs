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
