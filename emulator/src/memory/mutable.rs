use crate::memory::traits::{self, FromBinaryStr, FromBinaryStrLines, Memory};
use std::{array, fmt::Debug};
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
#[repr(transparent)]
pub struct MutableMemory<T, const S: usize>(Box<[T; S]>);

impl<T, const S: usize> Memory for MutableMemory<T, S> {
    type MemoryType = T;
}

impl<T: Debug, const S: usize> MutableMemory<T, S> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T, const S: usize> Default for MutableMemory<T, S>
where
    T: Default + Copy + Debug,
{
    fn default() -> Self {
        Self([Default::default(); S].into())
    }
}

impl<T, const S: usize> TryFrom<Vec<T>> for MutableMemory<T, S> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl<T, const S: usize> From<[T; S]> for MutableMemory<T, S> {
    fn from(value: [T; S]) -> Self {
        Self(value.into())
    }
}

impl<T, const S: usize> traits::WritableMemory for MutableMemory<T, S> {
    type MemoryError = ();

    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError> {
        *self.0.get_mut(index).ok_or(())? = value;
        Ok(())
    }
}
impl<T, const S: usize> traits::ReadableMemory for MutableMemory<T, S> {
    type MemoryError = ();

    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        self.0.get(index).ok_or(())
    }
}

impl<T, const S: usize> FromBinaryStrLines for MutableMemory<T, S>
where
    <MutableMemory<T, S> as Memory>::MemoryType: FromBinaryStr,
    T: Default,
{
    type Error = <<MutableMemory<T, S> as Memory>::MemoryType as FromBinaryStr>::Error;

    fn from_binary_str_lines<R: AsRef<str>>(
        lines: impl IntoIterator<Item = R>,
    ) -> Result<Self, Self::Error> {
        let mut arr = array::from_fn(|_| Default::default());

        for (i, line) in lines.into_iter().take(S).enumerate() {
            arr[i] = <MutableMemory<T, S> as Memory>::MemoryType::from_binary_str(line.as_ref())?;
        }

        Ok(Self(Box::new(arr)))
    }
}
