use std::{any::Any, array, iter};

use thiserror::Error;

use crate::memory::traits::{FromBinaryStr, FromBinaryStrLines, Memory, ReadableMemory};
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
#[repr(transparent)]
pub struct ImmutableMemory<T, const S: usize>(Box<[T; S]>);

impl<T, const S: usize> Memory for ImmutableMemory<T, S> {
    type MemoryType = T;
}

impl<T, const S: usize> ReadableMemory for ImmutableMemory<T, S> {
    type MemoryError = ();

    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        self.0.get(index).ok_or(())
    }
}
impl<T: Default, const S: usize> From<Vec<T>> for ImmutableMemory<T, S> {
    fn from(value: Vec<T>) -> Self {
        Self(unsafe {
            value
                .into_iter()
                .chain(iter::repeat_with(Default::default))
                .take(S)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap_unchecked()
        })
    }
}

impl<T: Default, const S: usize> From<ImmutableMemory<T, S>> for Vec<T> {
    fn from(value: ImmutableMemory<T, S>) -> Self {
        (*value.0).into()
    }
}

impl<T: Default, const S: usize> From<&ImmutableMemory<T, S>> for Vec<T>
where
    ImmutableMemory<T, S>: Clone,
{
    fn from(value: &ImmutableMemory<T, S>) -> Self {
        value.clone().into()
    }
}

impl<T, const S: usize> From<[T; S]> for ImmutableMemory<T, S> {
    fn from(value: [T; S]) -> Self {
        Self(value.into())
    }
}

#[derive(Error, Debug)]
pub enum ImmutableMemoryFromBinaryStrLinesError {
    #[error("Failed to parse {content:?} at line {line}")]
    ParseError { line: usize, content: String },
}

impl<T, const S: usize> FromBinaryStrLines for ImmutableMemory<T, S>
where
    T: FromBinaryStr + Default,
{
    type Error = ImmutableMemoryFromBinaryStrLinesError;

    fn from_binary_str_lines<R: AsRef<str>>(
        lines: impl IntoIterator<Item = R>,
    ) -> Result<Self, Self::Error> {
        let mut vec = vec![];
        for (i, line) in lines.into_iter().enumerate() {
            vec.push(T::from_binary_str(line.as_ref()).map_err(|_| {
                ImmutableMemoryFromBinaryStrLinesError::ParseError {
                    line: i + 1,
                    content: line.as_ref().into(),
                }
            })?);
        }
        Ok(vec.into())
    }
}

impl<T: Default, const S: usize> Default for ImmutableMemory<T, S> {
    fn default() -> Self {
        let vec = array::from_fn(|_| Default::default());
        Self(vec.into())
    }
}

impl<T, const S: usize> ImmutableMemory<T, S> {
    pub fn immutable_read(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }
}
