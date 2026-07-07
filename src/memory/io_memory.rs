use crate::{
    io::IOBits,
    memory::{
        mutable,
        traits::{MutableReadableMemory, ReadableMemory, WritableMemory},
    },
};
use std::{collections::VecDeque, io::Write};
use thiserror::Error;
// const SIZE: usize = 0x1000;
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct IOMemory {
    memory: mutable::MutableMemory<u16, { Self::MEMORY_SIZE }>,
    input_buf: VecDeque<Option<u8>>,
}

impl IOMemory {
    const MEMORY_SIZE: usize = 0x1000;
    const TRANSMITTER_STATUS_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 1 };
    const TRANSMITTER_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 2 };
    const RECEIVER_STATUS_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 3 };
    const RECEIVER_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 4 };
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

impl ReadableMemory for IOMemory {
    // const SIZE: usize = 0x1000;
    //Figure out how to get RO & RW memory to play nice so that there can RO machines
    type MemoryType = u16;
    type MemoryError = IOMemoryError<u16>;
    fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        match index {
            i if i == Self::RECEIVER_ADDRESS => {
                if self.receiver_status().on() {
                    Err(IOMemoryError::ImmutableMemory {
                        value: Some(*self.receiver()),
                    })
                } else {
                    Ok(self.receiver())
                }
            }
            i if i == Self::TRANSMITTER_ADDRESS => Ok(self.transmitter()),
            // Self::RECEIVER_STATUS_ADDRESS => Ok(&self.memory[index]),
            // Self::TRANSMITTER_STATUS_ADDRESS => Ok(&self.memory[index]),
            _ => self
                .memory
                .read(index)
                .or(Err(IOMemoryError::OutOfBounds(index))),
        }
    }
}
impl WritableMemory for IOMemory {
    type MemoryType = u16;
    type MemoryError = IOMemoryError<u16>;
    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError> {
        match index {
            i if i == Self::RECEIVER_STATUS_ADDRESS => {
                let bit_value = IOBits::from(value);
                self.set_receiver_status(if bit_value.on() {
                    bit_value.with_busy(false).with_done(true)
                } else {
                    IOBits::default()
                });
                Ok(())
            }
            i if i == Self::TRANSMITTER_ADDRESS => {
                if self.transmitter_status().can_write() {
                    self.set_transmitter(value);
                    std::io::stdout()
                        .write_all(&[((*self.transmitter()) & 0xFF) as u8])
                        .unwrap();
                    // std::io::stdout().flush().unwrap();
                    self.set_transmitter_status(
                        self.transmitter_status().with_done(true).with_busy(false), // .with_interupt(true),
                    );
                }
                Ok(())
            }
            i if i == Self::TRANSMITTER_STATUS_ADDRESS => {
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
impl MutableReadableMemory for IOMemory {
    type MemoryType = u16;
    type MemoryError = IOMemoryError<u16>;
    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        match index {
            i if i == Self::RECEIVER_ADDRESS => {
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
                        self.set_receiver(u16::from(byte));
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

impl IOMemory {
    pub fn len(&self) -> usize {
        self.memory.len()
    }

    fn receiver(&self) -> &u16 {
        self.memory.read(Self::RECEIVER_ADDRESS).unwrap()
    }

    fn set_receiver(&mut self, value: u16) {
        self.memory.write(Self::RECEIVER_ADDRESS, value).unwrap();
    }

    fn transmitter(&self) -> &u16 {
        self.memory.read(Self::TRANSMITTER_ADDRESS).unwrap()
    }

    fn set_transmitter(&mut self, value: u16) {
        self.memory.write(Self::TRANSMITTER_ADDRESS, value).unwrap();
    }

    pub fn receiver_status(&self) -> IOBits {
        IOBits::from(*self.memory.read(Self::RECEIVER_STATUS_ADDRESS).unwrap())
    }

    pub fn transmitter_status(&self) -> IOBits {
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
impl Default for IOMemory {
    fn default() -> Self {
        Self {
            memory: Default::default(),
            input_buf: Default::default(),
        }
    }
}

impl TryFrom<Vec<u16>> for IOMemory {
    type Error = Vec<u16>;

    fn try_from(value: Vec<u16>) -> Result<Self, Self::Error> {
        Ok(Self {
            memory: value.try_into()?,
            input_buf: Default::default(),
        })
    }
}
impl From<[u16; 0x1000]> for IOMemory {
    fn from(value: [u16; 0x1000]) -> Self {
        Self {
            memory: value.into(),
            input_buf: Default::default(),
        }
    }
}
