use crate::{
    io::IOBits,
    memory::traits::{Memory, MutableMemory, Receiver, Transmiter},
};
use num_traits::AsPrimitive;
use thiserror::Error;

// type MemorySize = u16;

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
#[repr(transparent)]
pub struct IOMemory<T, const S: usize> {
    memory: Box<[T; S]>,
} //TODO: convert to box slice, with read and write indexes as seperate members

pub mod traits {
    pub trait Memory {
        type MemoryType;
        type MemoryError<'a>
        where
            Self: 'a;
        fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError<'_>>;
    }

    pub trait MutableMemory {
        //TODO: some sort of special function to perform action on each pulse (like writting/reading characters)
        type MemoryType;
        type MemoryError;
        fn write(&mut self, index: usize, value: Self::MemoryType)
        -> Result<(), Self::MemoryError>;
        fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError>;
    }

    pub trait Actionable {
        type ActionResult;
        fn action(&mut self) -> Self::ActionResult;
    }

    pub trait Transmiter {
        const MONITORED_ADDRESS: Self;
        const VALUE_ADDRESS: Self;
    }
    pub trait Receiver {
        const MONITORED_ADDRESS: Self;
        const VALUE_ADDRESS: Self;
    }
}

impl Receiver for u16 {
    const MONITORED_ADDRESS: Self = 0xFFC;
    const VALUE_ADDRESS: Self = 0xFFD;
}

impl Transmiter for u16 {
    const MONITORED_ADDRESS: Self = 0xFFE;
    const VALUE_ADDRESS: Self = 0xFFF;
}

#[derive(Debug, Error, PartialEq, Eq, Hash, Clone, Copy)]
pub enum IOMemoryError {
    #[error("Out of bounds memory access at {0}")]
    OutOfBounds(usize),

    #[error("Cannot receive as struct is immutable")]
    ImmutableMemory,
}

impl<T, const S: usize> Memory for IOMemory<T, S>
where
    T: Receiver + Transmiter + AsPrimitive<usize>,
{
    //Figure out how to get RO & RW memory to play nice so that there can RO machines
    type MemoryType = T;
    type MemoryError<'a> = (Option<&'a Self::MemoryType>, IOMemoryError);
    fn read(&self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError<'_>> {
        match self.memory.get(index) {
            Some(v) => {
                if <T as Transmiter>::VALUE_ADDRESS.as_() == index
                    || <T as Receiver>::VALUE_ADDRESS.as_() == index
                {
                    Err((Some(v), IOMemoryError::ImmutableMemory))
                } else {
                    Ok(v)
                }
            }
            None => Err((None, IOMemoryError::OutOfBounds(index))),
        }
    }
}
impl<T, const S: usize> MutableMemory for IOMemory<T, S>
where
    T: Receiver + Transmiter + AsPrimitive<usize> + AsPrimitive<u16>, //TODO: refactor so IOBits is a generic param with some check and clear methods
    u16: Into<T>,
{
    type MemoryType = T;
    type MemoryError = IOMemoryError;
    fn write(&mut self, index: usize, value: Self::MemoryType) -> Result<(), Self::MemoryError> {
        match index {
            i if AsPrimitive::<usize>::as_(<T as Receiver>::MONITORED_ADDRESS) == i => {
                todo!("write RCVR status")
            }
            i if AsPrimitive::<usize>::as_(<T as Receiver>::VALUE_ADDRESS) == i => {
                todo!("write RCVR")
            }
            i if AsPrimitive::<usize>::as_(<T as Transmiter>::MONITORED_ADDRESS) == i => {
                todo!("write XMTR status")
            }
            i if AsPrimitive::<usize>::as_(<T as Transmiter>::VALUE_ADDRESS) == i => {
                todo!("write XMTR")
            }
            _ => {
                *self
                    .memory
                    .get_mut(index)
                    .ok_or(IOMemoryError::OutOfBounds(index))? = value;
                Ok(())
            }
        }
    }

    fn read(&mut self, index: usize) -> Result<&Self::MemoryType, Self::MemoryError> {
        if AsPrimitive::<usize>::as_(<T as Receiver>::MONITORED_ADDRESS) == index {
            let bits = IOBits::from_bits(
                self.memory
                    .get(AsPrimitive::<usize>::as_(
                        <T as Receiver>::MONITORED_ADDRESS,
                    ))
                    .ok_or_else(|| {
                        IOMemoryError::OutOfBounds(<T as Receiver>::MONITORED_ADDRESS.as_())
                    })?
                    .as_(),
            );

            if bits.on() {
                *self
                    .memory
                    .get_mut(AsPrimitive::<usize>::as_(
                        <T as Receiver>::MONITORED_ADDRESS,
                    ))
                    .ok_or_else(|| {
                        IOMemoryError::OutOfBounds(<T as Receiver>::MONITORED_ADDRESS.as_())
                    })? =
                    IOBits::into_bits(bits.with_done(false).with_busy(true).with_interupt(false))
                        .into();
            }
            // value
        }
        self.memory
            .get(index)
            .ok_or(IOMemoryError::OutOfBounds(index))
    }
}
impl<T, const S: usize> IOMemory<T, S> {
    pub fn len(&self) -> usize {
        self.memory.len()
    }
}

impl<T, const S: usize> TryFrom<Vec<T>> for IOMemory<T, S> {
    type Error = Vec<T>;

    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            memory: value.try_into()?,
        })
    }
}

// impl<T, const S: usize> Index<usize> for IOMemory<T, S>
// where
//     Self: Memory,
// {
//     type Output = <Self as Memory>::MemoryType;

//     fn index(&self, index: usize) -> &Self::Output {
//         match self.read(index) {
//             Ok(v) => v,
//             Err(_) => panic!("Out of bounds read"),
//         }
//     }
// }
