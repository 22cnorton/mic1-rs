pub mod traits;

use crate::{
    io::IOBits,
    memory::traits::{IOMappedMemory, MutableMemory, ReadableMemory},
};
use num_traits::AsPrimitive;
use std::{collections::VecDeque, fmt::Debug, io::Write, iter};
use thiserror::Error;

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct IOMemory<T> {
    memory: Box<[T]>,
    input_buf: VecDeque<Option<u8>>,
}

#[derive(Debug, Error, PartialEq, Eq, Hash, Clone, Copy)]
pub enum IOMemoryError {
    #[error("Out of bounds memory access at {0}")]
    OutOfBounds(usize),

    #[error("Cannot receive as struct is immutable")]
    ImmutableMemory,

    #[error("No characters from stdin")]
    NoCharacers,
}

impl<T> IOMappedMemory for IOMemory<T> {
    const SIZE: usize = 0x1000;
}

impl<T> ReadableMemory for IOMemory<T>
where
    T: ToOwned + From<IOBits>,
    IOBits: From<<T as ToOwned>::Owned>,
{
    //Figure out how to get RO & RW memory to play nice so that there can RO machines
    type MemoryType = T;
    type MemoryError<'a>
        = (Option<&'a Self::MemoryType>, IOMemoryError)
    where
        T: 'a;
    fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError<'_>> {
        match index {
            Self::RECEIVER_ADDRESS => {
                if self.receiver_status().on() {
                    Err((Some(self.receiver()), IOMemoryError::ImmutableMemory))
                } else {
                    Ok(self.receiver())
                }
            }
            Self::RECEIVER_STATUS_ADDRESS => Ok(&self.memory[index]),
            Self::TRANSMITTER_ADDRESS => Ok(self.transmitter()),
            Self::TRANSMITTER_STATUS_ADDRESS => Ok(&self.memory[index]),
            _ => self
                .memory
                .get(index)
                .ok_or((None, IOMemoryError::OutOfBounds(index))),
        }
    }
}
impl<T> MutableMemory for IOMemory<T>
where
    T: ToOwned + From<IOBits> + AsPrimitive<u8>,
    IOBits: From<T> + From<<T as ToOwned>::Owned>, //TODO: refactor so IOBits is a generic param with some check and clear methods
    u8: Into<T>,
{
    type MemoryType = T;
    type MemoryError = IOMemoryError;
    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError> {
        match index {
            Self::RECEIVER_ADDRESS => self.set_receiver(value),
            Self::RECEIVER_STATUS_ADDRESS => {
                let bit_value: IOBits = IOBits::from(value);
                self.set_receiver_status(if bit_value.on() {
                    bit_value.with_busy(false).with_done(true)
                } else {
                    IOBits::from_bits(0)
                });
            }
            Self::TRANSMITTER_ADDRESS => {
                if self.transmitter_status().on() && self.transmitter_status().done() {
                    self.set_transmitter(value);
                    std::io::stdout()
                        .write_all(&[self.transmitter().as_()])
                        .unwrap();
                    // std::io::stdout().flush().unwrap();
                    self.set_transmitter_status(
                        self.transmitter_status().with_done(true).with_busy(false), // .with_interupt(true),
                    );
                }
            }
            Self::TRANSMITTER_STATUS_ADDRESS => {
                let bit_value = IOBits::from(value);

                self.set_transmitter_status(if bit_value.on() {
                    bit_value.with_done(true).with_busy(false)
                } else {
                    IOBits::from(0)
                });
            }
            _ => {
                *self
                    .memory
                    .get_mut(index)
                    .ok_or(IOMemoryError::OutOfBounds(index))? = value;
            }
        }
        Ok(())
    }

    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        match index {
            Self::RECEIVER_ADDRESS => {
                if self.receiver_status().on() && self.receiver_status().done() {
                    if self.input_buf.is_empty() {
                        let mut buf = Default::default();

                        match std::io::stdin().read_line(&mut buf) {
                            Ok(s) if s > 0 => {
                                self.input_buf.extend(buf.bytes().map(Some));
                            }

                            Err(_) | Ok(_) => {
                                return Err(IOMemoryError::NoCharacers);
                            }
                        }
                    }
                    // eprintln!("{:?}", self.input_buf);
                    if let Some(Some(byte)) = self.input_buf.pop_front() {
                        self.set_receiver(byte.into());
                        self.set_receiver_status(
                            self.receiver_status().with_busy(false).with_done(true),
                        );
                        self.input_buf.push_front(None);
                    }
                }

                Ok(self.receiver())
            }
            // Self::RECEIVER_STATUS_ADDRESS => Ok(&self.memory[index]),
            Self::TRANSMITTER_ADDRESS => Ok(self.transmitter()),
            Self::TRANSMITTER_STATUS_ADDRESS | Self::RECEIVER_STATUS_ADDRESS => {
                // eprintln!("{}",line!());
                Ok(&self.memory[index])
            }
            _ => self
                .memory
                .get(index)
                .ok_or(IOMemoryError::OutOfBounds(index)),
        }
    }
}

impl<T> IOMemory<T> {
    pub fn len(&self) -> usize {
        self.memory.len()
    }

    fn receiver(&self) -> &T {
        &self.memory[Self::RECEIVER_ADDRESS]
    }

    fn set_receiver(&mut self, value: T) {
        self.memory[Self::RECEIVER_ADDRESS] = value;
    }

    fn transmitter(&self) -> &T {
        &self.memory[Self::TRANSMITTER_ADDRESS]
    }

    fn set_transmitter(&mut self, value: T) {
        self.memory[Self::TRANSMITTER_ADDRESS] = value;
    }
}

impl<T> IOMemory<T>
where
    T: ToOwned,
    IOBits: From<<T as ToOwned>::Owned>,
{
    pub fn receiver_status(&self) -> IOBits {
        self.memory[Self::RECEIVER_STATUS_ADDRESS].to_owned().into()
    }

    pub fn transmitter_status(&self) -> IOBits {
        self.memory[Self::TRANSMITTER_STATUS_ADDRESS]
            .to_owned()
            .into()
    }
}
impl<T> IOMemory<T>
where
    T: From<IOBits>,
{
    pub fn set_transmitter_status(&mut self, transmitter_status: IOBits) {
        // eprintln!("{:?}", transmitter_status);
        self.memory[Self::TRANSMITTER_STATUS_ADDRESS] = transmitter_status.into();
    }
    pub fn set_receiver_status(&mut self, receiver_status: IOBits) {
        self.memory[Self::RECEIVER_STATUS_ADDRESS] = receiver_status.into()
    }
}
impl<T: Default> Default for IOMemory<T> {
    fn default() -> Self {
        Self {
            memory: Vec::from_iter(iter::repeat_with(Default::default).take(Self::SIZE)).into(),
            input_buf: Default::default(),
        }
    }
}

impl<T> TryFrom<Vec<T>> for IOMemory<T> {
    type Error = ();

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        if value.len() != Self::SIZE {
            return Err(());
        }
        Ok(Self {
            memory: value.into_boxed_slice(),
            input_buf: Default::default(),
        })
    }
}
