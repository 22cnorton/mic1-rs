use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
};

use clap::Parser;
use either::Either;

use crate::machine::{Machine, Mic1Error};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Mic1Args {
    #[arg(
        long,
        help = "Path to PROM file. If not provided, the default PROM will be used"
    )]
    prom: Option<PathBuf>,

    #[arg(long, help = "Path to program file to run, as binary strings")]
    program: PathBuf,

    #[arg(long, default_value_t = 0, help = "Initial program counter value")]
    program_counter: u16,

    #[arg(long, default_value_t = 0x0F80, help = "Initial stack pointer value")]
    stack_pointer: u16,
}

impl Mic1Args {
    pub(crate) fn prom_path(&self) -> Option<&PathBuf> {
        self.prom.as_ref()
    }

    pub(crate) fn prom_data(&self) -> impl Iterator<Item = String> {
        if let Some(path) = &self.prom_path() {
            if let Ok(file) = File::open(path) {
                return Either::Left(
                    io::BufReader::new(file)
                        .lines()
                        .flatten()
                        .filter(|line| !line.chars().all(char::is_whitespace)),
                );
            }
        }

        Either::Right(include_str!("../prom.dat").lines().map(String::from))
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
