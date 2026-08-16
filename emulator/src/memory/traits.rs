pub trait ReadableMemory
where
    Self: Memory,
{
    type MemoryError;

    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError>;
}

pub trait WritableMemory
where
    Self: Memory,
{
    type MemoryError;

    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError>;
}

pub trait Memory {
    type MemoryType;
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
