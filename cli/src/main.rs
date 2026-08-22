use crate::{cli::Mic1Args, io_access::StdIOAccessor};
use clap::Parser;

use emulator::{
    machine::{MachineBuilder, MachineState, registers::RegistersBuilder},
    memory::{
        immutable::ImmutableMemory, io_memory::IOMemoryBuilder, mutable::MutableMemory,
        traits::FromBinaryStrLines,
    },
};
use std::io::{self, Write, stdin, stdout};

mod cli;
mod io_access;

macro_rules! print_mem {
    ($addr:expr, $value:expr) => {{
        println!(
            "     the location {:4} has value {:016b} , or {1:5}  or signed {:6}",
            $addr,
            $value,
            $value.cast_signed()
        );
    }};

    ($map:expr) => {{
        for (addr, value) in $map {
            print_mem!(addr, value);
        }
    }};
}

fn main() -> anyhow::Result<()> {
    let args = Mic1Args::parse();
    let io_accessor = StdIOAccessor;
    let prom_data: Vec<_> = args.prom_data().collect();
    let memory_data: Vec<_> = args.memory_data()?.collect();
    let read_micro_instructions = prom_data.len();
    let read_machine_instructions = memory_data.len();
    let mut emulator = MachineBuilder::default() //TODO: rewrite to use if let and emit failed to init on failure
        .micro_code(ImmutableMemory::from_binary_str_lines(prom_data)?)
        .memory(
            IOMemoryBuilder::default()
                .memory(MutableMemory::from_binary_str_lines(memory_data)?)
                .line_accessor(io_accessor)
                .build()?,
        )
        .registers(
            RegistersBuilder::default()
                .sp(args.stack_pointer())
                .pc(args.program_counter())
                .build()?,
        )
        .build()?;

    let mut state = EmulatorState::Init;

    loop {
        state = match state {
            EmulatorState::Init => {
                println!("Read in {} micro instructions", read_micro_instructions);
                println!("Read in {} machine instructions", read_machine_instructions);
                println!(
                    "{:<15}: {1:016b}  base 10: {1:7}",
                    "Starting PC is",
                    emulator.registers().pc()
                );
                println!(
                    "{:<15}: {1:016b}  base 10: {1:7}",
                    "Starting SP is",
                    emulator.registers().sp()
                );
                println!();

                EmulatorState::Processing
            }

            EmulatorState::Processing => match emulator.pulse() {
                MachineState::Halted => EmulatorState::Stats,
                MachineState::Running => EmulatorState::Processing,
            },
            EmulatorState::ShowPC => {
                println!("\nThe new PC is  : {:016b}\n", emulator.registers().pc());

                EmulatorState::Processing
            }
            EmulatorState::Stats => {
                println!("{}\n", emulator.registers());

                println!("{:<15}: {}\n", "Total Cycles", emulator.clock().tick());

                EmulatorState::Menu
            }
            EmulatorState::Menu => match main_menu() {
                MenuOptions::Quit => EmulatorState::Quit,
                MenuOptions::Continue => EmulatorState::ShowPC,
                MenuOptions::ViewMicrocode => EmulatorState::DisplayMicrocode,
                MenuOptions::ViewMemory(index) => EmulatorState::MemoryMenu(index),
            },
            EmulatorState::MemoryMenu(index) => {
                print_mem!(emulator.get_memory([index]));

                match memory_submenu(index) {
                    Some(
                        MemorySubmenuOptions::Forward { indices }
                        | MemorySubmenuOptions::Backward { indices },
                    ) => EmulatorState::DisplayMemory(indices),
                    Some(MemorySubmenuOptions::Quit) => EmulatorState::Quit,
                    Some(MemorySubmenuOptions::Continue) | None => EmulatorState::Menu,
                }
            }

            EmulatorState::DisplayMemory(memory_indicies) => {
                let memory_map = emulator.get_memory(memory_indicies);
                print_mem!(memory_map);

                EmulatorState::Menu
            }
            EmulatorState::DisplayMicrocode => {
                for (i, instruction) in emulator.microcode().iter().enumerate() {
                    println!("{:>4}: {instruction:?}", i.saturating_add(1));
                }

                EmulatorState::Menu
            }
            EmulatorState::Quit => {
                println!("MIC-1 emulator finishing, goodbye");
                break;
            }
        };
    }

    Ok(())
}

fn main_menu() -> MenuOptions {
    print!("Type decimal address to view memory, q to quit or c to continue: ");
    io::stdout().flush().expect("Failed to flush stdout");
    let line = get_line().unwrap_or_default();

    match line.to_lowercase().as_str() {
        "q" => MenuOptions::Quit,
        "c" => MenuOptions::Continue,
        #[cfg(debug_assertions)]
        "m" => MenuOptions::ViewMicrocode,
        _ => {
            let index = line.parse().unwrap_or_default();

            MenuOptions::ViewMemory(index)
        }
    }
}
fn memory_submenu(starting_index: usize) -> Option<MemorySubmenuOptions> {
    fn get_memory_steps(direction: &str) -> Option<usize> {
        print!("Type the number of {} locations to dump: ", direction);
        stdout().flush().expect("Failed to flush stdout");
        get_line().unwrap_or_default().trim().parse().ok()
    }

    println!("Type  {:>7}  to continue debugging", "<Enter>");
    println!("Type  {:>7}  to quit", 'q');
    println!("Type  {:>7} for forward range", 'f');
    print!("Type  {:>7} for backward range: ", 'b');
    std::io::stdout().flush().expect("Failed to flush stdout");

    let line = get_line().unwrap_or_default().to_lowercase();
    match line.as_str() {
        "" => Some(MemorySubmenuOptions::Continue),
        "q" => Some(MemorySubmenuOptions::Quit),
        "f" => {
            let steps = get_memory_steps("forward")?;
            let indices = (starting_index.saturating_add(1)..=starting_index + steps).collect();
            Some(MemorySubmenuOptions::Forward { indices })
        }
        "b" => {
            let steps = get_memory_steps("backward")?;
            let indices = (starting_index.saturating_sub(steps)..starting_index)
                .rev()
                .collect();
            Some(MemorySubmenuOptions::Backward { indices })
        }
        _ => None,
    }
}

#[derive(Debug, Clone, derive_more::IsVariant)]
enum MemorySubmenuOptions {
    Forward { indices: Vec<usize> },
    Backward { indices: Vec<usize> },
    Quit,
    Continue,
}

#[derive(Debug, Clone, derive_more::IsVariant)]
enum MenuOptions {
    Quit,
    Continue,
    ViewMicrocode,
    ViewMemory(usize),
}

fn get_line() -> Option<String> {
    let mut buf = Default::default();
    stdin().read_line(&mut buf).ok()?;
    Some(buf.trim().into())
}

#[derive(Debug, Clone)]
enum EmulatorState {
    Init,
    Processing,
    Menu,
    MemoryMenu(usize),
    DisplayMemory(Vec<usize>),
    DisplayMicrocode,
    ShowPC,
    Stats,
    Quit,
}
