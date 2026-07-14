use crate::{
    io::IOBits,
    memory::{
        mutable,
        traits::{ReadableMemory, WritableMemory},
    },
};
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Read, Write},
};
use thiserror::Error;

const MEMORY_SIZE: usize = 0x1000;

#[derive(Debug)]
pub struct IOMemory<R: Read, W: Write> {
    memory: mutable::MutableMemory<u16, { MEMORY_SIZE }>,
    input_buf: VecDeque<Option<u8>>,

    input_stream: BufReader<R>,
    output_stream: W,
}
type MemoryType = u16;

impl<R: Read, W: Write> IOMemory<R, W> {
    const TRANSMITTER_STATUS_ADDRESS: usize = { MEMORY_SIZE - 1 };
    const TRANSMITTER_ADDRESS: usize = { MEMORY_SIZE - 2 };
    const RECEIVER_STATUS_ADDRESS: usize = { MEMORY_SIZE - 3 };
    const RECEIVER_ADDRESS: usize = { MEMORY_SIZE - 4 };
}

#[derive(Debug, Error, PartialEq, Eq, Hash, Clone, Copy)]
pub enum IOMemoryError {
    #[error("Out of bounds memory access at {0}")]
    OutOfBounds(usize),

    // #[error("Reading from immutable memory that performs action on read")]
    // ImmutableMemory { value: Option<T> },
    #[error("No characters from stdin")]
    NoCharacters,
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
impl<R: Read, W: Write> WritableMemory<MemoryType> for IOMemory<R, W> {
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
                // eprintln!("{}",self.transmitter_status().can_write());
                if self.transmitter_status().can_write() {
                    self.set_transmitter(value);
                    let transmit_byte = ((*self.transmitter()) & 0xFF) as u8;
                    self.output_stream.write_all(&[transmit_byte]).unwrap();
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
impl<R: Read, W: Write> ReadableMemory<MemoryType> for IOMemory<R, W> {
    // type MemoryType = u16;
    type MemoryError = IOMemoryError;
    fn read(&mut self, index: usize) -> Result<&MemoryType, Self::MemoryError> {
        match index {
            i if i == Self::RECEIVER_ADDRESS => {
                if self.receiver_status().can_read() {
                    if self.input_buf.is_empty() {
                        let mut buf = Default::default();

                        match self.input_stream.read_line(&mut buf) {
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

impl<R: Read, W: Write> IOMemory<R, W> {
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
impl<R: Read + Default, W: Write + Default> Default for IOMemory<R, W> {
    fn default() -> Self {
        Self {
            memory: Default::default(),
            input_buf: Default::default(),
            input_stream: BufReader::new(R::default()),
            output_stream: W::default(),
        }
    }
}

impl<R: Read, W: Write> IOMemory<R, W> {
    pub fn try_from_vec(value: Vec<u16>,reader:R,writer:W) -> Result<Self, Vec<u16>> {
        Ok(Self {
            memory: value.try_into()?,
            input_buf: Default::default(),
            input_stream: BufReader::new(reader),
            output_stream: writer
        })
    }
}

impl<R: Read + Default, W: Write + Default> TryFrom<Vec<u16>> for IOMemory<R, W> {
    type Error = Vec<u16>;

    fn try_from(value: Vec<u16>) -> Result<Self, Self::Error> {
        Ok(Self {
            memory: value.try_into()?,
            input_buf: Default::default(),
            input_stream: BufReader::new(R::default()),
            output_stream: W::default(),
        })
    }
}
impl<R: Read + Default, W: Write + Default> From<[u16; MEMORY_SIZE]> for IOMemory<R, W> {
    fn from(value: [u16; MEMORY_SIZE]) -> Self {
        Self {
            memory: value.into(),
            input_buf: Default::default(),
            input_stream: BufReader::new(R::default()),
            output_stream: W::default(),
        }
    }
}
