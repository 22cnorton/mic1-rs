pub trait ReadableMemory<MemoryType> {
    // type MemoryType;
    type MemoryError;
    // const SIZE: usize;
    fn read(&mut self, index: usize) -> Result<&MemoryType, Self::MemoryError>;
}

pub trait WritableMemory<MemoryType> {
    // type MemoryType;
    type MemoryError;
    fn write(&mut self, index: usize, value: MemoryType) -> Result<(), Self::MemoryError>;
}

pub trait Memory<MemoryType>
where
    Self: ReadableMemory<MemoryType> + WritableMemory<MemoryType>,
{
    fn read(
        &mut self,
        index: usize,
    ) -> Result<&MemoryType, <Self as ReadableMemory<MemoryType>>::MemoryError> {
        <Self as ReadableMemory<MemoryType>>::read(self, index)
    }
    fn write(
        &mut self,
        index: usize,
        value: MemoryType,
    ) -> Result<(), <Self as WritableMemory<MemoryType>>::MemoryError> {
        <Self as WritableMemory<MemoryType>>::write(self, index, value)
    }
}

// pub trait MutableReadableMemory {
//     type MemoryType;
//     type MemoryError;
//     fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError>;
// }
