use crate::memory::{
    mutable,
    traits::{self, IOBitsType, IOMemoryType, ReadableMemory,WritableMemory},
};
use std::{collections::VecDeque, io::Write, marker::PhantomData};
use thiserror::Error;
// const SIZE: usize = 0x1000;
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct IOMemory<
    T,
    I,
    const S: usize,
    const TRANSMITTER_STATUS_ADDRESS: usize,
    const TRANSMITTER_ADDRESS: usize,
    const RECEIVER_STATUS_ADDRESS: usize,
    const RECEIVER_ADDRESS: usize,
> where
    T: IOMemoryType,
    I: IOBitsType<T>,
{
    memory: mutable::MutableMemory<T, S>,
    input_buf: VecDeque<Option<u8>>,
    _phantom: PhantomData<I>,
}

#[derive(Debug, Error, PartialEq, Eq, Hash, Clone, Copy)]
pub enum IOMemoryError<T> {
    #[error("Out of bounds memory access at {0}")]
    OutOfBounds(usize),

    #[error("Reading from immutable memory that performs action on read")]
    ImmutableMemory { value: Option<T> },

    #[error("No characters from stdin")]
    NoCharacters,
}

impl<
    T,
    I,
    const S: usize,
    const TRANSMITTER_STATUS_ADDRESS: usize,
    const TRANSMITTER_ADDRESS: usize,
    const RECEIVER_STATUS_ADDRESS: usize,
    const RECEIVER_ADDRESS: usize,
> ReadableMemory
    for IOMemory<
        T,
        I,
        S,
        TRANSMITTER_STATUS_ADDRESS,
        TRANSMITTER_ADDRESS,
        RECEIVER_STATUS_ADDRESS,
        RECEIVER_ADDRESS,
    >
where
    T: IOMemoryType,
    I: IOBitsType<T>,
{
    // const SIZE: usize = 0x1000;
    //Figure out how to get RO & RW memory to play nice so that there can RO machines
    type MemoryType = T;
    type MemoryError = IOMemoryError<T>;
    fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        match index {
            i if i == RECEIVER_ADDRESS => {
                if self.receiver_status().on() {
                    Err(IOMemoryError::ImmutableMemory {
                        value: Some(*self.receiver()),
                    })
                } else {
                    Ok(self.receiver())
                }
            }
            i if i == TRANSMITTER_ADDRESS => Ok(self.transmitter()),
            // Self::RECEIVER_STATUS_ADDRESS => Ok(&self.memory[index]),
            // Self::TRANSMITTER_STATUS_ADDRESS => Ok(&self.memory[index]),
            _ => self
                .memory
                .read(index)
                .or(Err(IOMemoryError::OutOfBounds(index))),
        }
    }
}
impl<
    T,
    I,
    const S: usize,
    const TRANSMITTER_STATUS_ADDRESS: usize,
    const TRANSMITTER_ADDRESS: usize,
    const RECEIVER_STATUS_ADDRESS: usize,
    const RECEIVER_ADDRESS: usize,
> traits::WritableMemory
    for IOMemory<
        T,
        I,
        S,
        TRANSMITTER_STATUS_ADDRESS,
        TRANSMITTER_ADDRESS,
        RECEIVER_STATUS_ADDRESS,
        RECEIVER_ADDRESS,
    >
where
    T: IOMemoryType,
    I: IOBitsType<T>,
{
    type MemoryType = T;
    type MemoryError = IOMemoryError<T>;
    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError> {
        match index {
            i if i == RECEIVER_STATUS_ADDRESS => {
                let bit_value: I = I::from(value);
                self.set_receiver_status(if bit_value.on() {
                    bit_value.with_busy(false).with_done(true)
                } else {
                    I::default()
                });
                Ok(())
            }
            i if i == TRANSMITTER_ADDRESS => {
                if self.transmitter_status().can_write() {
                    self.set_transmitter(value);
                    std::io::stdout()
                        .write_all(&[(*self.transmitter()).into()])
                        .unwrap();
                    // std::io::stdout().flush().unwrap();
                    self.set_transmitter_status(
                        self.transmitter_status().with_done(true).with_busy(false), // .with_interupt(true),
                    );
                }
                Ok(())
            }
            i if i == TRANSMITTER_STATUS_ADDRESS => {
                let bit_value = I::from(value);

                self.set_transmitter_status(if bit_value.on() {
                    bit_value.with_done(true).with_busy(false)
                } else {
                    I::default()
                });
                Ok(())
            }
            _ => self
                .memory
                .write(index, value)
                .or(Err(IOMemoryError::OutOfBounds(index))),
        }
    }
}
impl<
    T,
    I,
    const S: usize,
    const TRANSMITTER_STATUS_ADDRESS: usize,
    const TRANSMITTER_ADDRESS: usize,
    const RECEIVER_STATUS_ADDRESS: usize,
    const RECEIVER_ADDRESS: usize,
> traits::MutableReadableMemory
    for IOMemory<
        T,
        I,
        S,
        TRANSMITTER_STATUS_ADDRESS,
        TRANSMITTER_ADDRESS,
        RECEIVER_STATUS_ADDRESS,
        RECEIVER_ADDRESS,
    >
where
    T: IOMemoryType,
    I: IOBitsType<T>,
{
    type MemoryType = T;
    type MemoryError = IOMemoryError<T>;
    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        match index {
            i if i == RECEIVER_ADDRESS => {
                if self.receiver_status().can_read() {
                    if self.input_buf.is_empty() {
                        let mut buf = Default::default();

                        match std::io::stdin().read_line(&mut buf) {
                            Ok(s) if s > 0 => {
                                self.input_buf.extend(buf.bytes().map(Some));
                            }

                            Err(_) | Ok(_) => {
                                return Err(IOMemoryError::NoCharacters);
                            }
                        }
                    }
                    // eprintln!("{:?}", self.input_buf);
                    if let Some(Some(byte)) = self.input_buf.pop_front() {
                        self.set_receiver(T::from(byte));
                        self.set_receiver_status(
                            self.receiver_status().with_busy(false).with_done(true),
                        );
                        self.input_buf.push_front(None);
                    }
                }

                Ok(self.receiver())
            }
            // Self::RECEIVER_STATUS_ADDRESS => Ok(&self.memory[index]),
            _ => self
                .memory
                .read(index)
                .or(Err(IOMemoryError::OutOfBounds(index))),
        }
    }
}

impl<
    T,
    I,
    const S: usize,
    const TRANSMITTER_STATUS_ADDRESS: usize,
    const TRANSMITTER_ADDRESS: usize,
    const RECEIVER_STATUS_ADDRESS: usize,
    const RECEIVER_ADDRESS: usize,
>
    IOMemory<
        T,
        I,
        S,
        TRANSMITTER_STATUS_ADDRESS,
        TRANSMITTER_ADDRESS,
        RECEIVER_STATUS_ADDRESS,
        RECEIVER_ADDRESS,
    >
where
    T: IOMemoryType,
    I: IOBitsType<T>,
{
    pub fn len(&self) -> usize {
        self.memory.len()
    }

    fn receiver(&self) -> &T {
        self.memory.read(RECEIVER_ADDRESS).unwrap()
    }

    fn set_receiver(&mut self, value: T) {
        self.memory.write(RECEIVER_ADDRESS, value).unwrap();
    }

    fn transmitter(&self) -> &T {
        self.memory.read(TRANSMITTER_ADDRESS).unwrap()
    }

    fn set_transmitter(&mut self, value: T) {
        self.memory.write(TRANSMITTER_ADDRESS, value).unwrap();
    }

    pub fn receiver_status(&self) -> I {
        I::from(*self.memory.read(RECEIVER_STATUS_ADDRESS).unwrap())
    }

    pub fn transmitter_status(&self) -> I {
        I::from(*self.memory.read(TRANSMITTER_STATUS_ADDRESS).unwrap())
    }

    pub fn set_transmitter_status(&mut self, transmitter_status: I) {
        self.memory
            .write(TRANSMITTER_STATUS_ADDRESS, transmitter_status.into())
            .unwrap();
    }
    pub fn set_receiver_status(&mut self, receiver_status: I) {
        self.memory
            .write(RECEIVER_STATUS_ADDRESS, receiver_status.into())
            .unwrap();
    }
}
impl<
    T,
    I,
    const S: usize,
    const TRANSMITTER_STATUS_ADDRESS: usize,
    const TRANSMITTER_ADDRESS: usize,
    const RECEIVER_STATUS_ADDRESS: usize,
    const RECEIVER_ADDRESS: usize,
> Default
    for IOMemory<
        T,
        I,
        S,
        TRANSMITTER_STATUS_ADDRESS,
        TRANSMITTER_ADDRESS,
        RECEIVER_STATUS_ADDRESS,
        RECEIVER_ADDRESS,
    >
where
    T: Default + IOMemoryType,
    I: IOBitsType<T>,
{
    fn default() -> Self {
        Self {
            memory: Default::default(),
            input_buf: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<
    T,
    I,
    const S: usize,
    const TRANSMITTER_STATUS_ADDRESS: usize,
    const TRANSMITTER_ADDRESS: usize,
    const RECEIVER_STATUS_ADDRESS: usize,
    const RECEIVER_ADDRESS: usize,
> TryFrom<Vec<T>>
    for IOMemory<
        T,
        I,
        S,
        TRANSMITTER_STATUS_ADDRESS,
        TRANSMITTER_ADDRESS,
        RECEIVER_STATUS_ADDRESS,
        RECEIVER_ADDRESS,
    >
where
    T: IOMemoryType,
    I: IOBitsType<T>,
{
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            memory: value.try_into()?,
            input_buf: Default::default(),
            _phantom: PhantomData,
        })
    }
}
impl<
    T,
    I,
    const S: usize,
    const TRANSMITTER_STATUS_ADDRESS: usize,
    const TRANSMITTER_ADDRESS: usize,
    const RECEIVER_STATUS_ADDRESS: usize,
    const RECEIVER_ADDRESS: usize,
> From<[T; S]>
    for IOMemory<
        T,
        I,
        S,
        TRANSMITTER_STATUS_ADDRESS,
        TRANSMITTER_ADDRESS,
        RECEIVER_STATUS_ADDRESS,
        RECEIVER_ADDRESS,
    >
where
    T: IOMemoryType,
    I: IOBitsType<T>,
{
    fn from(value: [T; S]) -> Self {
        Self {
            memory: value.into(),
            input_buf: Default::default(),
            _phantom: PhantomData,
        }
    }
}
