use crate::cli::initalize_machine;
use anyhow::Context;

mod cli;
mod io;
mod machine;
mod memory;
mod microcode;
mod registers;

fn main() -> anyhow::Result<()> {
    let mut machine = initalize_machine().context("Failed to create machine")?;

    // while let Ok(()) = machine.pulse() {}
    loop {
        machine.pulse()?
    }
}
