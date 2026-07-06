pub trait ReadableMemory {
    type MemoryType;
    type MemoryError;
    // const SIZE: usize;
    fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError>;
}

pub trait WritableMemory {
    type MemoryType;
    type MemoryError;
    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError>;
}

pub trait MutableReadableMemory {
    type MemoryType;
    type MemoryError;
    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError>;
}
