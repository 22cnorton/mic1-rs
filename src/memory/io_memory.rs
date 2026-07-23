use crate::{
    io::IOBits,
    memory::{
        mutable,
        traits::{FromBinaryStr, FromBinaryStrLines, ReadableMemory, WritableMemory},
    },
};
use std::{collections::VecDeque, io::Write, num::ParseIntError};
use thiserror::Error;
// const SIZE: usize = 0x1000;
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct IOMemory {
    memory: mutable::MutableMemory<u16, { Self::MEMORY_SIZE }>,
    input_buf: VecDeque<Option<u8>>,
}
type MemoryType = u16;

impl FromBinaryStr for MemoryType {
    type Error = ParseIntError;

    fn from_binary_str(s: &str) -> Result<Self, Self::Error> {
        MemoryType::from_str_radix(s, 2)
    }
}

impl IOMemory {
    const MEMORY_SIZE: usize = 0x1000;
    const TRANSMITTER_STATUS_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 1 };
    const TRANSMITTER_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 2 };
    const RECEIVER_STATUS_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 3 };
    const RECEIVER_ADDRESS: usize = { IOMemory::MEMORY_SIZE - 4 };
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum IOMemoryError {
    #[error("Out of bounds memory access at {0}")]
    OutOfBounds(usize),

    // #[error("Reading from immutable memory that performs action on read")]
    // ImmutableMemory { value: Option<T> },
    #[error("No characters from stdin")]
    NoCharacters,

    #[error(transparent)]
    LineParse(#[from] ParseIntError),

    #[error("Failed to create IOMemory from {0:#04x?}")]
    ConstructFromVec(Vec<MemoryType>),
}

// impl ReadableMemory<u16> for IOMemory {
//     // const SIZE: usize = 0x1000;
//     //Figure out how to get RO & RW memory to play nice so that there can RO machines
//     // type MemoryType = u16;
//     type MemoryError = IOMemoryError<u16>;
//     fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
//         match index {
//             i if i == Self::RECEIVER_ADDRESS => {
//                 if self.receiver_status().on() {
//                     Err(IOMemoryError::ImmutableMemory {
//                         value: Some(*self.receiver()),
//                     })
//                 } else {
//                     Ok(self.receiver())
//                 }
//             }
//             i if i == Self::TRANSMITTER_ADDRESS => Ok(self.transmitter()),
//             // Self::RECEIVER_STATUS_ADDRESS => Ok(&self.memory[index]),
//             // Self::TRANSMITTER_STATUS_ADDRESS => Ok(&self.memory[index]),
//             _ => self
//                 .memory
//                 .read(index)
//                 .or(Err(IOMemoryError::OutOfBounds(index))),
//         }
//     }
// }
impl WritableMemory<MemoryType> for IOMemory {
    // type MemoryType = u16;
    type MemoryError = IOMemoryError;
    fn write(&mut self, index: usize, value: MemoryType) -> Result<(), Self::MemoryError> {
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
                    let status = self.transmitter_status().with_done(true).with_busy(false);
                    self.set_transmitter_status(
                        status, // .with_interupt(true),
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
impl ReadableMemory<MemoryType> for IOMemory {
    // type MemoryType = u16;
    type MemoryError = IOMemoryError;
    fn read(&mut self, index: usize) -> Result<&MemoryType, Self::MemoryError> {
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
                        let status = self.receiver_status().with_busy(false).with_done(true);
                        self.set_receiver_status(status);
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
impl Default for IOMemory {
    fn default() -> Self {
        Self {
            memory: Default::default(),
            input_buf: Default::default(),
        }
    }
}

impl TryFrom<Vec<MemoryType>> for IOMemory {
    type Error = Vec<MemoryType>;

    fn try_from(value: Vec<MemoryType>) -> Result<Self, Self::Error> {
        Ok(Self {
            memory: value.try_into()?,
            input_buf: Default::default(),
        })
    }
}
impl From<[MemoryType; Self::MEMORY_SIZE]> for IOMemory {
    fn from(value: [MemoryType; Self::MEMORY_SIZE]) -> Self {
        Self {
            memory: value.into(),
            input_buf: Default::default(),
        }
    }
}

impl FromBinaryStrLines for IOMemory {
    type Error = IOMemoryError;

    fn from_binary_str_lines<S: AsRef<str>>(
        lines: impl IntoIterator<Item = S>,
    ) -> Result<Self, Self::Error> {
        let mut vec = vec![Default::default(); Self::MEMORY_SIZE];
        for (i, line) in lines.into_iter().enumerate() {
            vec[i] = MemoryType::from_binary_str(line.as_ref())?;
        }

        Ok(vec
            .try_into()
            .map_err(|e| IOMemoryError::ConstructFromVec(e))?)
    }
}
