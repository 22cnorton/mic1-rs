use core::fmt;

pub type RegisterSize = u16;

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
pub struct Registers([RegisterSize; 16]);
// #[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
// pub struct Registers {

//     pc: RegisterSize,
//     ac: RegisterSize,
//     sp: RegisterSize,
//     ir: RegisterSize,
//     tir: RegisterSize,
//     zero: RegisterSize,
//     one: RegisterSize,
//     neg_one: RegisterSize,
//     amask: RegisterSize,
//     smask: RegisterSize,
//     a: RegisterSize,
//     b: RegisterSize,
//     c: RegisterSize,
//     d: RegisterSize,
//     e: RegisterSize,
//     f: RegisterSize,
// }

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\n{:<15}: {1:016b}  base 10: {1:7}",
            "ProgramCounter",
            self.pc()
        )?;
        writeln!(
            f,
            "{:<15}: {1:016b}  base 10: {1:7}",
            "Accumulator",
            self.ac()
        )?;
        writeln!(
            f,
            "{:<15}: {1:016b}  base 10: {1:7}",
            "InstructionReg",
            self.ir()
        )?;
        writeln!(
            f,
            "{:<15}: {1:016b}  base 10: {1:7}",
            "TempInstr",
            self.tir()
        )?;
        writeln!(
            f,
            "{:<15}: {1:016b}  base 10: {1:7}",
            "StackPointer",
            self.sp()
        )?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "ARegister", self.a())?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "BRegister", self.b())?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "CRegister", self.c())?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "DRegister", self.d())?;
        writeln!(f, "{:<15}: {1:016b}  base 10: {1:7}", "ERegister", self.e())?;
        write!(f, "{:<15}: {1:016b}  base 10: {1:7}", "FRegister", self.f())
    }
}

impl Registers {
    const STACK_POINTER: usize = 2;
    const PROGRAM_COUNTER: usize = 0;
    const ACCUMULATOR: usize = 1;
    const INSTRUCTION_REGISTER: usize = 3;
    const TEMP_INSTRUCTION_REGISTER: usize = 4;
    const A_REGISTER: usize = 10;
    const B_REGISTER: usize = 11;
    const C_REGISTER: usize = 12;
    const D_REGISTER: usize = 13;
    const E_REGISTER: usize = 14;
    const F_REGISTER: usize = 15;

    pub fn new(stack_pointer: RegisterSize, program_counter: RegisterSize) -> Self {
        let mut arr = [Default::default(); 16];
        arr[Self::STACK_POINTER] = stack_pointer;
        arr[Self::PROGRAM_COUNTER] = program_counter;
        Self(arr)
    }

    fn a(&self) -> RegisterSize {
        self.0[Self::A_REGISTER]
    }
    fn b(&self) -> RegisterSize {
        self.0[Self::B_REGISTER]
    }
    fn c(&self) -> RegisterSize {
        self.0[Self::C_REGISTER]
    }
    fn d(&self) -> RegisterSize {
        self.0[Self::D_REGISTER]
    }
    fn e(&self) -> RegisterSize {
        self.0[Self::E_REGISTER]
    }
    fn f(&self) -> RegisterSize {
        self.0[Self::F_REGISTER]
    }

    fn ac(&self) -> RegisterSize {
        self.0[Self::ACCUMULATOR]
    }

    fn ir(&self) -> RegisterSize {
        self.0[Self::INSTRUCTION_REGISTER]
    }
    fn tir(&self) -> RegisterSize {
        self.0[Self::TEMP_INSTRUCTION_REGISTER]
    }

    pub fn sp(&self) -> RegisterSize {
        self.0[Self::STACK_POINTER]
    }
    pub fn pc(&self) -> RegisterSize {
        self.0[Self::PROGRAM_COUNTER]
    }

    pub fn read_from_reg(&self, index: u8) -> RegisterSize {
        self.0[index as usize]
    }

    pub fn write_to_reg(&mut self, index: u8, value: RegisterSize) {
        self.0[index as usize] = value;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self([
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            (0),
            (1),
            (u16::MAX),
            (0x0FFF),
            (0x00FF),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        ])
    }
}
