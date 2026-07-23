use crate::{
    cli::Mic1Args,
    machine::MachineBuilder,
    memory::{IOMemory, immutable::ImmutableMemory, traits::FromBinaryStrLines},
    registers::RegistersBuilder,
};

use clap::Parser;

mod cli;
mod io;
mod machine;
mod memory;
mod microcode;
mod registers;

fn main() -> anyhow::Result<()> {
    let args = Mic1Args::parse();
    let mut machine = MachineBuilder::default()
        .micro_code(ImmutableMemory::from_binary_str_lines(args.prom_data())?)
        .memory(IOMemory::from_binary_str_lines(args.memory_data()?)?)
        .registers(
            RegistersBuilder::default()
                .sp(args.stack_pointer())
                .pc(args.program_counter())
                .build()
                .unwrap(),
        )
        .build()?;

    loop {
        machine.pulse()?
    }
}
