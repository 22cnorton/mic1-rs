use crate::cli::Mic1Args;
use clap::Parser;
use derive_more::Debug;
use emulator::{
    machine::{MachineBuilder, registers::RegistersBuilder},
    memory::{
        IOMemoryBuilder, immutable::ImmutableMemory, mutable::MutableMemory,
        traits::FromBinaryStrLines,
    },
    messages::{Command, Event},
};
use std::{
    io::{self, Write, stdout},
    iter,
};

mod cli;

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
    let (command_tx, command_rx) = flume::bounded(1);
    let (event_tx, event_rx) = flume::unbounded();
    let (input_tx, input_rx) = flume::unbounded::<String>(); // TODO: channel to send input to the emulator thread
    let (output_tx, output_rx) = flume::unbounded::<String>(); // TODO: channel to send output from the emulator thread

    let args = Mic1Args::parse();

    let prom_data: Vec<_> = args.prom_data().collect();
    let memory_data: Vec<_> = args.memory_data()?.collect();
    let read_micro_instructions = prom_data.len();
    let read_machine_instructions = memory_data.len();

    let mut machine = MachineBuilder::default()
        .micro_code(ImmutableMemory::from_binary_str_lines(prom_data)?)
        .memory(
            IOMemoryBuilder::default()
                .memory(MutableMemory::from_binary_str_lines(memory_data)?)
                .event_tx(event_tx.clone())
                .command_rx(command_rx.clone())
                .build()?,
        )
        .event_tx(event_tx.clone())
        .command_rx(command_rx.clone())
        .registers(
            RegistersBuilder::default()
                .sp(args.stack_pointer())
                .pc(args.program_counter())
                .build()?,
        )
        .build()?;

    std::thread::spawn(move || -> anyhow::Result<()> {
        event_tx.send(emulator::messages::Event::DoneInit {
            sp: args.stack_pointer(),
            pc: args.program_counter(),
            read_micro_instructions,
            read_machine_instructions,
        })?;

        loop {
            machine.pulse()?;
        }
    });

    let mut state = EmulatorState::Init;
    while !event_rx.is_disconnected() {
        // while let Ok(event) = event_rx.recv() {
        //TODO: what we should do is do the menu, then send a command and act on the result. The menu can only be shown after the first halt event, so use a variable to keep track of if we have halted or not, reset when continuing

        state = match state {
            EmulatorState::Init => {
                let event = event_rx.recv()?;
                if let Event::DoneInit {
                    read_micro_instructions,
                    read_machine_instructions,
                    sp,
                    pc,
                } = event
                {
                    println!("Read in {} micro instructions", read_micro_instructions);
                    println!("Read in {} machine instructions", read_machine_instructions);
                    println!("{:<15}: {1:016b}  base 10: {1:7}", "Starting PC is", pc);
                    println!("{:<15}: {1:016b}  base 10: {1:7}", "Starting SP is", sp);
                    println!();
                }
                EmulatorState::Processing
            }

            EmulatorState::Processing => match event_rx.recv()? {
                Event::Halted => EmulatorState::Stats,
                _ => EmulatorState::Processing,
            },
            EmulatorState::ShowPC => {
                command_tx.send(Command::ViewRegisters)?;
                if let Event::Registers(registers) = event_rx.recv()? {
                    println!("\nThe new PC is  : {:016b}\n", registers.pc());
                }
                command_tx.send(Command::Continue)?;
                EmulatorState::Processing
            }
            EmulatorState::Stats => {
                command_tx.send(Command::ViewRegisters)?;
                if let Event::Registers(registers) = event_rx.recv()? {
                    println!("{registers}\n");
                }

                command_tx.send(Command::ViewCycles)?;
                if let Event::Cycles(cycles) = event_rx.recv()? {
                    println!("{:<15}: {}\n", "Total Cycles", cycles);
                }

                EmulatorState::Menu
            }
            EmulatorState::Menu => match main_menu() {
                MenuOptions::Quit => EmulatorState::Quit,
                MenuOptions::Continue => EmulatorState::ShowPC,
                MenuOptions::ViewMicrocode => EmulatorState::DisplayMicrocode,
                MenuOptions::ViewMemory(index) => {
                    command_tx.send(Command::ViewMemory(vec![index]))?;
                    EmulatorState::MemoryMenu
                }
            },
            EmulatorState::MemoryMenu => {
                let index = if let Event::Memory(memory_map) = event_rx.recv()? {
                    let index = memory_map[0].0;
                    print_mem!(memory_map);
                    index
                } else {
                    Default::default()
                };

                match memory_submenu(index) {
                    Some(
                        MemorySubmenuOptions::Forward { indices }
                        | MemorySubmenuOptions::Backward { indices },
                    ) => {
                        command_tx.send(Command::ViewMemory(indices))?;
                        EmulatorState::DisplayMemory
                    }
                    Some(MemorySubmenuOptions::Quit) => EmulatorState::Quit,
                    Some(MemorySubmenuOptions::Continue) | None => EmulatorState::Menu,
                }
            }

            EmulatorState::DisplayMemory => {
                if let Event::Memory(memory_map) = event_rx.recv()? {
                    print_mem!(memory_map);
                }

                EmulatorState::Menu
            }
            EmulatorState::DisplayMicrocode => {
                command_tx.send(Command::ViewMicrocode)?;
                if let Event::Microcode(micro_code) = event_rx.recv()? {
                    for (i, instruction) in iter::zip(
                        iter::successors(Some(1), |i: &i32| Some(i.saturating_add(1))),
                        micro_code.iter(),
                    ) {
                        println!("{i:>4}: {instruction:?}");
                    }
                }

                EmulatorState::Menu
            }
            EmulatorState::Quit => {
                command_tx.send(Command::Quit)?;
                println!("MIC-1 emulator finishing, goodbye");
                return Ok(());
            }
        }
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
        let input = get_line().unwrap_or_default();
        input.trim().parse().ok()
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
            let indices = (starting_index + 1..=starting_index + steps).collect();
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

fn get_line() -> io::Result<String> {
    let mut input = Default::default();

    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(input.trim().to_string()),
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone, Copy)]
enum EmulatorState {
    Init,
    Processing,
    Menu,
    MemoryMenu,
    DisplayMemory,
    DisplayMicrocode,
    ShowPC,
    Stats,
    Quit,
}
