use core::fmt;
use std::ops::{Index, IndexMut};

use derive_builder::Builder;
use getset::{Getters, Setters, WithSetters};

pub type RegisterSize = u16;

#[repr(C)]
#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Builder, Getters, Setters, WithSetters)]
#[builder(default)]
#[getset(get = "pub", set = "pub")]
pub struct Registers {
    pc: RegisterSize,
    ac: RegisterSize,
    sp: RegisterSize,
    ir: RegisterSize,
    tir: RegisterSize,
    zero: RegisterSize,
    one: RegisterSize,
    neg_one: RegisterSize,
    amask: RegisterSize,
    smask: RegisterSize,
    a: RegisterSize,
    b: RegisterSize,
    c: RegisterSize,
    d: RegisterSize,
    e: RegisterSize,
    f: RegisterSize,
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\n{:<15}: {1:016b}  base 10: {1:7}",
            "ProgramCounter", self.pc
        )?;
        writeln!(
            f,
            "{:<15}: {1:016b}  base 10: {1:7}",
            "Accumulator", self.ac
        )?;
        writeln!(
            f,
            "{:<15}: {1:016b}  base 10: {1:7}",
            "InstructionReg", self.ir
        )?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "TempInstr", self.tir)?;
        writeln!(
            f,
            "{:<15}: {1:016b}  base 10: {1:7}",
            "StackPointer", self.sp
        )?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "ARegister", self.a)?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "BRegister", self.b)?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "CRegister", self.c)?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "DRegister", self.d)?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "ERegister", self.e)?;
        write!(f, "{:<15}: {1:016b}  base 10: {1:7}", "FRegister", self.f)?;

        Ok(())
    }
}

impl Index<u8> for Registers {
    type Output = RegisterSize;

    fn index(&self, index: u8) -> &Self::Output {
        if index > 15 {
            panic!("Invalid register index: {}", index);
        }
        unsafe {
            let array_ptr = self as *const _ as *const [_; 16];
            &(*array_ptr)[index as usize]
        }
    }
}

impl IndexMut<u8> for Registers {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        if index > 15 {
            panic!("Invalid register index: {}", index);
        }
        unsafe {
            let array_ptr = self as *mut _ as *mut [_; 16];
            &mut (*array_ptr)[index as usize]
        }
    }
}

impl Registers {
    pub fn read_from_reg(&self, index: u8) -> RegisterSize {
        self[index]
    }

    pub fn write_to_reg(&mut self, index: u8, value: RegisterSize) {
        self[index] = value;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            zero: (0),
            one: (1),
            neg_one: (u16::MAX),
            amask: (0x0FFF),
            smask: (0x00FF),
            ..[Default::default(); _].into()
        }
    }
}

impl From<[RegisterSize; 16]> for Registers {
    fn from(value: [RegisterSize; 16]) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
