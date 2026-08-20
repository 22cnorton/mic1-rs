use emulator::machine::{
    microcode::MicroInstruction,
    registers::{RegisterSize, Registers},
};
use std::{fmt::Debug, num::NonZeroUsize};

#[derive(Debug)]
pub enum Command {
    ViewMemory(Vec<usize>),
    ViewRegisters,
    ViewMicrocode,
    Quit,
    Continue,
    ViewCycles,
}

#[derive(Debug)]
pub enum Event<T> {
    Memory(Vec<(usize, T)>),
    Registers(Registers),
    Cycles(usize),
    Microcode(Vec<MicroInstruction>),
    Halted,
    DoneInit {
        sp: RegisterSize,
        pc: RegisterSize,
        read_micro_instructions: usize,
        read_machine_instructions: usize,
    },
}
