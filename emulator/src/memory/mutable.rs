use std::fmt::Debug;

use crate::memory::traits;
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
#[repr(transparent)]
pub struct MutableMemory<T, const S: usize>(Box<[T; S]>);
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

impl<T, const S: usize> traits::WritableMemory<T> for MutableMemory<T, S> {
    type MemoryError = ();

    fn write(&mut self, index: usize, value: T) -> Result<(), Self::MemoryError> {
        *self.0.get_mut(index).ok_or(())? = value;
        Ok(())
    }
}
impl<T, const S: usize> traits::ReadableMemory<T> for MutableMemory<T, S> {
    type MemoryError = ();

    fn read(&mut self, index: usize) -> Result<&T, Self::MemoryError> {
        self.0.get(index).ok_or(())
    }
}
