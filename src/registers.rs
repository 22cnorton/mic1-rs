use core::fmt;

use derive_builder::Builder;

pub type RegisterSize = u16;

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash, Builder)]
#[builder(setter(skip), default)]
pub struct Registers {
    #[builder(setter, default)]
    pc: RegisterSize,
    ac: RegisterSize,
    #[builder(setter, default)]
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

impl Registers {
    pub fn sp(&self) -> RegisterSize {
        self.sp
    }
    pub fn pc(&self) -> RegisterSize {
        self.pc
    }

    pub fn read_from_reg(&self, index: u8) -> RegisterSize {
        match index {
            0 => self.pc,
            1 => self.ac,
            2 => self.sp,
            3 => self.ir,
            4 => self.tir,
            5 => self.zero,
            6 => self.one,
            7 => self.neg_one,
            8 => self.amask,
            9 => self.smask,
            10 => self.a,
            11 => self.b,
            12 => self.c,
            13 => self.d,
            14 => self.e,
            15 => self.f,
            _ => panic!("Invalid register index: {}", index),
        }
    }

    pub fn write_to_reg(&mut self, index: u8, value: RegisterSize) {
        // eprintln!("Writing value {:016b} to register index {}", value, index);
        match index {
            0 => self.pc = value,
            1 => self.ac = value,
            2 => self.sp = value,
            3 => self.ir = value,
            4 => self.tir = value,
            5 => self.zero = value,
            6 => self.one = value,
            7 => self.neg_one = value,
            8 => self.amask = value,
            9 => self.smask = value,
            10 => self.a = value,
            11 => self.b = value,
            12 => self.c = value,
            13 => self.d = value,
            14 => self.e = value,
            15 => self.f = value,
            _ => panic!("Invalid register index: {}", index),
        }
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
        Self {
            pc: value[0],
            ac: value[1],
            sp: value[2],
            ir: value[3],
            tir: value[4],
            zero: value[5],
            one: value[6],
            neg_one: value[7],
            amask: value[8],
            smask: value[9],
            a: value[10],
            b: value[11],
            c: value[12],
            d: value[13],
            e: value[14],
            f: value[15],
        }
    }
}
