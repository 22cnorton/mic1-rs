pub trait ReadableMemory<MemoryType> {
    type MemoryError;

    fn read(&mut self, index: usize) -> Result<&MemoryType, Self::MemoryError>;
}

pub trait WritableMemory<MemoryType> {
    type MemoryError;
    
    fn write(&mut self, index: usize, value: MemoryType) -> Result<(), Self::MemoryError>;
}

pub trait Memory<MemoryType>
where
    Self: ReadableMemory<MemoryType> + WritableMemory<MemoryType>,
{
}

pub trait FromBinaryStr: Sized {
    type Error;

    fn from_binary_str(s: &str) -> Result<Self, Self::Error>;
}

pub trait FromBinaryStrLines: Sized {
    type Error;

    fn from_binary_str_lines<S: AsRef<str>>(
        lines: impl IntoIterator<Item = S>,
    ) -> Result<Self, Self::Error>;
}
