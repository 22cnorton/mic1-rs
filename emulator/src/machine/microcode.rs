use bitfield_struct::{bitenum, bitfield};

#[bitfield(u32, hash = true, order = msb)]
#[derive(Eq, PartialEq)]
pub struct MicroInstruction {
    #[bits(1, default = false)]
    pub(crate) amux: bool, // controls left ALU input: 0 = A latch, 1 = MBR
    #[bits(2, default = Jump::None)]
    pub(crate) cond: Jump,
    #[bits(2, default = Operation::Add)]
    pub(crate) alu: Operation,
    #[bits(2, default = Shift::None)]
    pub(crate) sh: Shift,
    #[bits(1, default = false)]
    pub(crate) mbr: bool, // loads MBR from shifter: 0 = don't load MBR, 1 = load MBR
    #[bits(1, default = false)]
    pub(crate) mar: bool, // loads MAR from B latch: 0 = don't load MAR, 1 = load MAR
    #[bits(1, default = true)]
    pub(crate) rd: bool, // requests memory read: 0 = no read, 1 = load MBR from memory
    #[bits(1, default = true)]
    pub(crate) wr: bool, // requests memory write: 0 = no write, 1 = write MBR to memory
    #[bits(1, default = false)]
    pub(crate) enc: bool, // controls storing into scratchpad: 0 = don't store, 1 = store
    #[bits(4, default = 0)]
    pub(crate) c: u8, // selects register for storing into if ENC =1: 0 = PC, 1 = Ac, etc.
    #[bits(4, default = 0)]
    pub(crate) b: u8, // selects B bus source: 0 = PC, 1 = AC, etc.
    #[bits(4, default = 0)]
    pub(crate) a: u8, // selects A bus source: 0 = PC, 1 = AC, etc.
    #[bits(8, default = 0)]
    pub(crate) addr: u8, // next address to go to
}

impl crate::memory::traits::FromBinaryStr for MicroInstruction {
    type Error = std::num::ParseIntError;

    fn from_binary_str(s: &str) -> Result<Self, Self::Error> {
        Ok(MicroInstruction::from_bits(u32::from_str_radix(s, 2)?))
    }
}

#[bitenum]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Jump {
    #[fallback]
    None = 0,
    Negative = 1,
    Zero = 2,
    Always = 3,
}

#[bitenum]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Operation {
    #[fallback]
    Add = 0,
    And = 1,
    Assign = 2,
    Invert = 3,
}

#[bitenum]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Shift {
    #[fallback]
    None = 0,
    Right = 1,
    Left = 2,
}
