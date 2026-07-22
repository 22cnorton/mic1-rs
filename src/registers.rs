use core::fmt;
use std::{
    fmt::{Binary, Display},
    ops::{Index, IndexMut},
};

use derive_builder::Builder;
use getset::{Getters, Setters, WithSetters};

pub type RegisterSize = u16;

#[repr(C)]
#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Builder, Getters, Setters, WithSetters)]
#[builder]
#[getset(get = "pub", set = "pub")]
pub struct Registers<T> {
    pc: T,
    ac: T,
    sp: T,
    ir: T,
    tir: T,
    zero: T,
    one: T,
    neg_one: T,
    amask: T,
    smask: T,
    a: T,
    b: T,
    c: T,
    d: T,
    e: T,
    f: T,
}

impl<T> fmt::Display for Registers<T>
where
    T: Binary + Display,
{
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

impl<T> Index<usize> for Registers<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index > 15 {
            panic!("Invalid register index: {}", index);
        }
        unsafe {
            let array_ptr = self as *const _ as *const [_; 16];
            &(*array_ptr)[index as usize]
        }
    }
}

impl<T> IndexMut<usize> for Registers<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index > 15 {
            panic!("Invalid register index: {}", index);
        }
        unsafe {
            let array_ptr = self as *mut _ as *mut [_; 16];
            &mut (*array_ptr)[index as usize]
        }
    }
}

impl<T> Registers<T> {
    pub fn read_from_reg(&self, index: usize) -> &T {
        &self[index]
    }

    pub fn write_to_reg(&mut self, index: usize, value: T) {
        self[index] = value;
    }
}

impl Default for Registers<u16> {
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

impl<T> From<[T; 16]> for Registers<T> {
    fn from(value: [T; 16]) -> Self {
        let [
            pc,
            ac,
            sp,
            ir,
            tir,
            zero,
            one,
            neg_one,
            amask,
            smask,
            a,
            b,
            c,
            d,
            e,
            f,
        ] = value;
        Self {
            pc,
            ac,
            sp,
            ir,
            tir,
            zero,
            one,
            neg_one,
            amask,
            smask,
            a,
            b,
            c,
            d,
            e,
            f,
        }
    }
}
