use crate::{
    memory::{
        io::IOBits,
        mutable,
        traits::{FromBinaryStr, Memory, ReadableMemory, WritableMemory},
    },
    messages::{Command, Event, Line},
};
use derive_builder::Builder;
use flume::{Receiver, Sender};
use std::collections::VecDeque;
use std::num::ParseIntError;
use thiserror::Error;

const MEMORY_SIZE: usize = 0x1000;
#[derive(Debug, Clone, Builder)]
pub struct IOMemory {
    memory: mutable::MutableMemory<<IOMemory as Memory>::MemoryType, MEMORY_SIZE>,
    #[builder(setter(skip))]
    input_buf: VecDeque<Option<u8>>,

    command_rx: Receiver<Command>,
    event_tx: Sender<Event<<IOMemory as Memory>::MemoryType>>,
}

impl FromBinaryStr for <IOMemory as Memory>::MemoryType {
    type Error = ParseIntError;

    fn from_binary_str(s: &str) -> Result<Self, Self::Error> {
        <IOMemory as Memory>::MemoryType::from_str_radix(s, 2)
    }
}

impl IOMemory {
    const MEMORY_SIZE: usize = MEMORY_SIZE;
    const TRANSMITTER_STATUS_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 1 };
    const TRANSMITTER_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 2 };
    const RECEIVER_STATUS_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 3 };
    const RECEIVER_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 4 };
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum IOMemoryError {
    #[error("Out of bounds memory access at {0}")]
    OutOfBounds(usize),

    #[error("No characters from stdin")]
    NoCharacters,

    #[error(transparent)]
    LineParse(#[from] ParseIntError),

    #[error("Failed to create IOMemory from {0:#04x?}")]
    ConstructFromVec(Vec<<IOMemory as Memory>::MemoryType>),
}

impl Memory for IOMemory {
    type MemoryType = u16;
}

impl WritableMemory for IOMemory {
    type MemoryError = IOMemoryError;
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
                    let event =
                        Event::Write(Line::Bytes([(*self.transmitter() & 0xFF) as u8].to_vec()));
                    self.event_tx.send(event).unwrap();

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
impl ReadableMemory for IOMemory {
    type MemoryError = IOMemoryError;
    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        match index {
            Self::RECEIVER_ADDRESS => {
                if self.receiver_status().can_read() {
                    if self.input_buf.is_empty() {
                        match self.command_rx.recv() {
                            Ok(Command::Line(str)) => self.input_buf.extend(str.bytes().map(Some)),
                            _ => return Err(IOMemoryError::NoCharacters),
                        };
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

impl IOMemory {
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
