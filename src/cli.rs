use std::path::PathBuf;

use clap::Parser;

use crate::machine::{Machine, Mic1Error};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Mic1Args {
    #[arg(long, default_value = "prom.dat")]
    prom: PathBuf,

    #[arg(long, default_value = "inner.dat")]
    program: PathBuf,

    #[arg(long, default_value_t = 0)]
    program_counter: u16,

    #[arg(long, default_value_t = 0x0F80)]
    stack_pointer: u16,
}

impl Mic1Args {
    pub(crate) fn prom(&self) -> &PathBuf {
        &self.prom
    }

    pub(crate) fn program(&self) -> &PathBuf {
        &self.program
    }

    pub(crate) fn program_counter(&self) -> u16 {
        self.program_counter
    }

    pub(crate) fn stack_pointer(&self) -> u16 {
        self.stack_pointer
    }
}

pub fn initalize_machine() -> Result<Machine, Mic1Error> {
    Machine::from_args(Mic1Args::parse())
}
