use crate::{
    machine::{
        microcode::MicroInstruction,
        registers::{RegisterSize, Registers},
    },
    memory::immutable::ImmutableMemory,
};

use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    num::NonZeroUsize,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Line {
    String(String),
    Bytes(Vec<u8>),
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Line::String(s) => write!(f, "{s}"),
            Line::Bytes(bytes) => write!(f, "{bytes:?}"),
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Line(String),
    ViewMemory(Vec<usize>),
    ViewRegisters,
    ViewMicrocode,
    Tick { count: NonZeroUsize },
    Quit,
    Continue,
    ViewCycles,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Event<T> {
    Memory(Vec<(usize, T)>),
    Continue,
    Write(Line),
    Registers(Registers),
    Cycles(usize),
    Microcode(Vec<MicroInstruction>),
    Halted,
    Finished,
    AwaitingLine,
    AwaitingCommand,
    AwaitingMemoryLocation,
    DoneProcessing,
    // FailedToInit(anyhow::Error),
    DoneInit {
        sp: RegisterSize,
        pc: RegisterSize,
        read_micro_instructions: usize,
        read_machine_instructions: usize,
    },
}
