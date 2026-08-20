use crate::{cli::Mic1Args, io_access::ChannelLineAccessor};
use clap::Parser;

use emulator::{
    machine::{MachineBuilder, MachineState, registers::RegistersBuilder},
    memory::{
        immutable::ImmutableMemory, io_memory::IOMemoryBuilder, mutable::MutableMemory,
        traits::FromBinaryStrLines,
    },
};
use flume::Receiver;
use messages::{Command, Event};
use std::io::{self, Write, stdin, stdout};

mod cli;
mod io_access;
mod messages;

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
    let (input_tx, input_rx) = flume::unbounded(); // TODO: channel to send input to the emulator thread
    let (output_tx, output_rx) = flume::unbounded(); // TODO: channel to send output from the emulator thread
    let emulator_input_rx = input_rx.clone();

    std::thread::spawn(move || {
        let args = Mic1Args::parse();
        let io_accessor = ChannelLineAccessor {
            tx: output_tx.clone(),
            rx: emulator_input_rx,
        };
        let prom_data: Vec<_> = args.prom_data().collect();
        let memory_data: Vec<_> = args.memory_data().unwrap().collect();
        let read_micro_instructions = prom_data.len();
        let read_machine_instructions = memory_data.len();
        let mut emulator =
            MachineBuilder::default() //TODO: rewrite to use if let and emit failed to init on failure
                .micro_code(ImmutableMemory::from_binary_str_lines(prom_data).unwrap())
                .memory(
                    IOMemoryBuilder::<ChannelLineAccessor>::default()
                        .memory(MutableMemory::from_binary_str_lines(memory_data).unwrap())
                        .line_accessor(io_accessor)
                        .build()
                        .unwrap(),
                )
                .registers(
                    RegistersBuilder::default()
                        .sp(args.stack_pointer())
                        .pc(args.program_counter())
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap();
        event_tx
            .send(messages::Event::DoneInit {
                sp: args.stack_pointer(),
                pc: args.program_counter(),
                read_micro_instructions,
                read_machine_instructions,
            })
            .unwrap();
        let mut halted = false;
        loop {
            if !halted {
                let state = emulator.pulse(); // pulse machine
                if let MachineState::Halted = state {
                    event_tx.send(Event::Halted).unwrap();
                    halted = true;
                }
            }
            // manage commands
            while let Ok(command) = command_rx.try_recv() {
                // eprintln!("Processing {command:?}");/
                event_tx
                    .send(match command {
                        Command::ViewMemory(indicies) => {
                            Event::Memory(emulator.get_memory(indicies))
                        }
                        Command::ViewRegisters => Event::Registers(*emulator.registers()),
                        Command::ViewMicrocode => Event::Microcode(emulator.microcode()),
                        Command::Quit => return,
                        Command::Continue => {
                            halted = false;
                            emulator.r#continue();
                            break;
                        }
                        Command::ViewCycles => Event::Cycles(*emulator.clock().tick()),
                    })
                    .unwrap();
            }
        }
    });

    std::thread::spawn(move || {
        for line in stdin().lines() {
            if let Ok(line) = line {
                let mut bytes = line;
                bytes.push('\n');

                if matches!(input_tx.send(bytes), Err(_)) {
                    return;
                }
            }
        }
    });

    let mut state = EmulatorState::Init;
    let input_buffer = input_rx.clone();

    while !event_rx.is_disconnected() {
        // while let Ok(event) = event_rx.recv() {

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

            EmulatorState::Processing => flume::Selector::new()
                .recv(&event_rx, |event| match event {
                    Ok(Event::Halted) => EmulatorState::Stats,
                    _ => EmulatorState::Processing,
                })
                .recv(&output_rx, |output| {
                    if let Ok(line) = output {
                        print!("{line}");

                        stdout().flush().unwrap();
                    }
                    EmulatorState::Processing
                })
                .wait(),
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
            EmulatorState::Menu => match main_menu(&input_buffer) {
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

                match memory_submenu(index, &input_buffer) {
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
                    for (i, instruction) in micro_code.iter().enumerate() {
                        println!("{:>4}: {instruction:?}", i.saturating_add(1));
                    }
                }

                EmulatorState::Menu
            }
            EmulatorState::Quit => {
                command_tx.send(Command::Quit)?;
                println!("MIC-1 emulator finishing, goodbye");
                break;
            }
        }
    }

    Ok(())
}

fn main_menu(input_buffer: &Receiver<String>) -> MenuOptions {
    print!("Type decimal address to view memory, q to quit or c to continue: ");
    io::stdout().flush().expect("Failed to flush stdout");
    let line = get_line(input_buffer).unwrap_or_default();

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
fn memory_submenu(
    starting_index: usize,
    input_buffer: &Receiver<String>,
) -> Option<MemorySubmenuOptions> {
    fn get_memory_steps(direction: &str, input_buffer: &Receiver<String>) -> Option<usize> {
        print!("Type the number of {} locations to dump: ", direction);
        stdout().flush().expect("Failed to flush stdout");
        get_line(input_buffer)
            .unwrap_or_default()
            .trim()
            .parse()
            .ok()
    }

    println!("Type  {:>7}  to continue debugging", "<Enter>");
    println!("Type  {:>7}  to quit", 'q');
    println!("Type  {:>7} for forward range", 'f');
    print!("Type  {:>7} for backward range: ", 'b');
    std::io::stdout().flush().expect("Failed to flush stdout");

    let line = get_line(input_buffer).unwrap_or_default().to_lowercase();
    match line.as_str() {
        "" => Some(MemorySubmenuOptions::Continue),
        "q" => Some(MemorySubmenuOptions::Quit),
        "f" => {
            let steps = get_memory_steps("forward", input_buffer)?;
            let indices = (starting_index.saturating_add(1)..=starting_index + steps).collect();
            Some(MemorySubmenuOptions::Forward { indices })
        }
        "b" => {
            let steps = get_memory_steps("backward", input_buffer)?;
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

fn get_line(input_buffer: &Receiver<String>) -> Option<String> {
    input_buffer.recv().ok().map(|str| str.trim().to_string())
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
