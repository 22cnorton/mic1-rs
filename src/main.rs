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
    let prom_data: Vec<_> = args.prom_data().collect();
    let memory_data: Vec<_> = args.memory_data()?.collect();

    let mut machine = MachineBuilder::default()
        .read_micro_instructions(prom_data.len() as u8)
        .read_machine_instructions(memory_data.len() as u16)
        .micro_code(ImmutableMemory::from_binary_str_lines(prom_data)?)
        .memory(IOMemory::from_binary_str_lines(memory_data)?)
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
