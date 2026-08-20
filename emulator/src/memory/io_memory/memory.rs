use crate::memory::{
    io::IOBits,
    io_memory::access::LineAccessor,
    mutable,
    traits::{FromBinaryStr, Memory, ReadableMemory, WritableMemory},
};
use derive_builder::Builder;
use derive_more::Debug;
use std::collections::VecDeque;
use std::num::ParseIntError;
use thiserror::Error;

const MEMORY_SIZE: usize = 0x1000;
#[derive(Debug, Clone, Builder)]
// #[builder(pattern = "owned")]
pub struct IOMemory<T: LineAccessor<String>> {
    memory: mutable::MutableMemory<<IOMemory<T> as Memory>::MemoryType, MEMORY_SIZE>,
    #[builder(setter(skip))]
    input_buf: VecDeque<Option<u8>>,

    #[debug(skip)]
    line_accessor: T,
}

impl FromBinaryStr for u16 {
    type Error = ParseIntError;

    fn from_binary_str(s: &str) -> Result<Self, Self::Error> {
        u16::from_str_radix(s, 2)
    }
}

impl<T: LineAccessor<String>> IOMemory<T> {
    const MEMORY_SIZE: usize = MEMORY_SIZE;
    const TRANSMITTER_STATUS_ADDRESS: usize = { IOMemory::<T>::MEMORY_SIZE - 1 };
    const TRANSMITTER_ADDRESS: usize = { IOMemory::<T>::MEMORY_SIZE - 2 };
    const RECEIVER_STATUS_ADDRESS: usize = { IOMemory::<T>::MEMORY_SIZE - 3 };
    const RECEIVER_ADDRESS: usize = { IOMemory::<T>::MEMORY_SIZE - 4 };
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum IOMemoryError<T: LineAccessor<String>> {
    #[error("Out of bounds memory access at {0}")]
    OutOfBounds(usize),

    #[error("No characters from stdin")]
    NoCharacters,

    #[error(transparent)]
    LineParse(#[from] ParseIntError),

    #[error("Failed to create IOMemory from {0:#04x?}")]
    ConstructFromVec(Vec<<IOMemory<T> as Memory>::MemoryType>),

    #[error("Write Failed")]
    WriteFail,
}

impl<T: LineAccessor<String>> Memory for IOMemory<T> {
    type MemoryType = u16;
}

impl<T: LineAccessor<String>> WritableMemory for IOMemory<T> {
    type MemoryError = IOMemoryError<T>;
    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError> {
        match index {
            Self::RECEIVER_STATUS_ADDRESS => {
                let bit_value = IOBits::from(value);
                self.set_receiver_status(if bit_value.on() {
                    bit_value.with_busy(false).with_done(true)
                } else {
                    IOBits::default()
                });
                Ok(())
            }
            Self::TRANSMITTER_ADDRESS => {
                if self.transmitter_status().can_write() {
                    self.set_transmitter(value);
                    let text = [(*self.transmitter() & 0xFF) as u8].to_vec();
                    self.line_accessor
                        .send_line(&String::from_utf8(text).map_err(|_| IOMemoryError::WriteFail)?);

                    let status = self.transmitter_status().with_done(true).with_busy(false);
                    self.set_transmitter_status(status);
                }
                Ok(())
            }
            Self::TRANSMITTER_STATUS_ADDRESS => {
                let bit_value = IOBits::from(value);

                self.set_transmitter_status(if bit_value.on() {
                    bit_value.with_done(true).with_busy(false)
                } else {
                    IOBits::default()
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
impl<T: LineAccessor<String>> ReadableMemory for IOMemory<T> {
    type MemoryError = IOMemoryError<T>;
    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        match index {
            Self::RECEIVER_ADDRESS => {
                if self.receiver_status().can_read() {
                    if self.input_buf.is_empty() {
                        self.input_buf
                            .extend(self.line_accessor.get_line().bytes().into_iter().map(Some));
                    }
                    if let Some(Some(byte)) = self.input_buf.pop_front() {
                        self.set_receiver(u16::from(byte));
                        let status = self.receiver_status().with_busy(false).with_done(true);
                        self.set_receiver_status(status);
                        self.input_buf.push_front(None);
                    }
                }

                Ok(self.receiver())
            }
            _ => self
                .memory
                .read(index)
                .or(Err(IOMemoryError::OutOfBounds(index))),
        }
    }
}

impl<T: LineAccessor<String>> IOMemory<T> {
    pub fn len(&self) -> usize {
        self.memory.len()
    }

    fn receiver(&mut self) -> &u16 {
        self.memory.read(Self::RECEIVER_ADDRESS).unwrap()
    }

    fn set_receiver(&mut self, value: u16) {
        self.memory.write(Self::RECEIVER_ADDRESS, value).unwrap();
    }

    fn transmitter(&mut self) -> &u16 {
        self.memory.read(Self::TRANSMITTER_ADDRESS).unwrap()
    }

    fn set_transmitter(&mut self, value: u16) {
        self.memory.write(Self::TRANSMITTER_ADDRESS, value).unwrap();
    }

    pub fn receiver_status(&mut self) -> IOBits {
        IOBits::from(*self.memory.read(Self::RECEIVER_STATUS_ADDRESS).unwrap())
    }

    pub fn transmitter_status(&mut self) -> IOBits {
        IOBits::from(*self.memory.read(Self::TRANSMITTER_STATUS_ADDRESS).unwrap())
    }

    pub fn set_transmitter_status(&mut self, transmitter_status: IOBits) {
        self.memory
            .write(Self::TRANSMITTER_STATUS_ADDRESS, transmitter_status.into())
            .unwrap();
    }
    pub fn set_receiver_status(&mut self, receiver_status: IOBits) {
        self.memory
            .write(Self::RECEIVER_STATUS_ADDRESS, receiver_status.into())
            .unwrap();
    }
}
