use std::array;

use crate::memory::traits::ReadableMemory;
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
#[repr(transparent)]
pub struct ImmutableMemory<T, const S: usize>(Box<[T; S]>);

impl<T, const S: usize> ReadableMemory<T> for ImmutableMemory<T, S> {
    type MemoryError = ();

    fn read(&mut self, index: usize) -> Result<&T, Self::MemoryError> {
        self.0.get(index).ok_or(())
    }
}
impl<T, const S: usize> TryFrom<Vec<T>> for ImmutableMemory<T, S> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl<T, const S: usize> From<[T; S]> for ImmutableMemory<T, S> {
    fn from(value: [T; S]) -> Self {
        Self(value.into())
    }
}

impl<T: Default, const S: usize> Default for ImmutableMemory<T, S> {
    fn default() -> Self {
        let vec = array::from_fn(|_| Default::default());
        Self(vec.into())
    }
}


