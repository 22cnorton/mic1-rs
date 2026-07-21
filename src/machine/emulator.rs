use crate::cli::Mic1Args;
use crate::machine::clock::{Clock, Subtick};
use crate::memory::IOMemory;
use crate::memory::immutable::ImmutableMemory;
use crate::memory::traits::{Memory, ReadableMemory, WritableMemory};
use crate::microcode::{self, MicroInstruction};
use crate::registers::{RegisterSize, Registers};
use anyhow::Result;
use derive_builder::Builder;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead};
use std::iter;
use thiserror::Error;

const MICROCODE_LENGTH: usize = 256;
#[derive(Eq, PartialEq, Debug, Clone, Hash, Default, Builder)]
#[builder(setter(skip))]
pub struct Machine {
    #[builder(setter)]
    memory: IOMemory,
    #[builder(setter)]
    micro_code: ImmutableMemory<MicroInstruction, { MICROCODE_LENGTH }>,

    #[builder(setter)]
    registers: Registers,
    blocking_io: bool,
    clock: Clock,
    mir: MicroInstruction,
    micro_pc: u8,
    a_bus: RegisterSize,
    b_bus: RegisterSize,
    c_bus: RegisterSize,
    mbr: RegisterSize,
    mar: RegisterSize, // Retype since this can only be twelve bits

    read_micro_instructions: u8, // TODO: make ctor that returns machine with these values instead of carrying it arround
    read_machine_instructions: u16,
}

impl Machine {
    pub const MEMORY_SIZE: usize = 4096;
    pub const MICROCODE_LENGTH: usize = MICROCODE_LENGTH;
    #[allow(dead_code)]
    pub fn current_instruction(&mut self) -> u16 {
        *self
            .memory
            .read(self.registers.pc() as usize)
            .expect("Never read out of bounds")
    }
    #[allow(dead_code)]
    pub fn current_micro_instruction(&self) -> MicroInstruction {
        self.mir
    }

    fn instruction_at(&mut self, addr: u8) -> MicroInstruction {
        *self
            .micro_code
            .read(addr as usize)
            .unwrap_or(&Default::default())
    }

    fn load(&mut self) {
        self.mir = self.instruction_at(self.micro_pc);
    }

    fn gate(&mut self) {
        self.a_bus = self.registers.read_from_reg(self.mir.a());
        self.b_bus = self.registers.read_from_reg(self.mir.b());
    }

    fn calc(&mut self) {
        let a_value = if self.mir.amux() {
            self.mbr
        } else {
            self.a_bus
        };
        if self.mir.mar() {
            self.mar = self.b_bus & 0xFFF
        };
        let b_value = self.b_bus;

        let c_value = Self::alu(a_value, b_value, self.mir.alu());
        self.micro_pc = self.next_micro_instruction(c_value, self.mir.cond());

        self.c_bus = Self::shift(c_value, self.mir.sh());
        if self.mir.mbr() {
            self.mbr = self.c_bus;
        }
    }

    fn alu(a_value: RegisterSize, b_value: RegisterSize, op: microcode::Operation) -> RegisterSize {
        match op {
            microcode::Operation::Add => a_value.wrapping_add(b_value),
            microcode::Operation::And => a_value & b_value,
            microcode::Operation::Assign => a_value,
            microcode::Operation::Invert => !a_value,
        }
    }

    fn shift(value: RegisterSize, sh: microcode::Shift) -> RegisterSize {
        match sh {
            microcode::Shift::None => value,
            microcode::Shift::Left => value << 1,
            microcode::Shift::Right => value >> 1,
        }
    }

    fn next_micro_instruction(&self, value: RegisterSize, cond: microcode::Jump) -> u8 {
        match cond {
            microcode::Jump::None => self.micro_pc.wrapping_add(1),
            microcode::Jump::Negative => {
                if (value as i16) < 0 {
                    self.mir.addr()
                } else {
                    self.micro_pc.wrapping_add(1)
                }
            }
            microcode::Jump::Zero => {
                if value == 0 {
                    self.mir.addr()
                } else {
                    self.micro_pc.wrapping_add(1)
                }
            }
            microcode::Jump::Always => self.mir.addr(),
        }
    }

    fn store(&mut self) {
        if self.mir.enc() {
            self.registers.write_to_reg(self.mir.c(), self.c_bus);
        }
        if self.mir.mbr() {
            self.mbr = self.c_bus;
        }
    }

    fn halt(&mut self) -> Result<()> {
        self.blocking_io = true;

        println!("{}", self.registers);
        println!();
        println!("{:<15}: {}", "Total Cycles", self.clock.tick());
        println!();

        macro_rules! quit {
            () => {{
                println!("MIC-1 emulator finishing, goodbye");
                anyhow::bail!("Quitting MIC-1 emulator");
            }};
        }

        macro_rules! get_range {
            ($direction:literal) => {{
                print!("Type the number of {} locations to dump: ", $direction);
                io::stdout().flush().expect("Failed to flush stdout");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                input.trim().parse::<usize>().ok()
            }};
        }

        loop {
            print!("Type decimal address to view memory, q to quit or c to continue: ");
            io::stdout().flush().expect("Failed to flush stdout");
            let mut input = String::new();

            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");

            let input = input.trim();
            match input.to_lowercase().as_str() {
                "q" => quit!(),
                "c" => {
                    self.blocking_io = false;
                    self.clock.set_subtick(Subtick::Load); // Reset subtick to Load for next instruction
                    break Ok(());
                }
                #[cfg(debug_assertions)]
                "m" => {
                    println!("Micro Code");
                    println!("{:?}", self.micro_code)
                }
                _ => {
                    if let Ok(addr) = input.parse() {
                        if addr < self.memory.len() {
                            self.display_memory(iter::once(addr));
                            println!("Type  {:>7}  to continue debugging", "<Enter>");
                            println!("Type  {:>7}  to quit", 'q');
                            println!("Type  {:>7} for forward range", 'f');
                            print!("Type  {:>7} for backward range: ", 'b');
                            io::stdout().flush().expect("Failed to flush stdout");
                            let mut input = String::new();

                            io::stdin()
                                .read_line(&mut input)
                                .expect("Failed to read input");
                            let input = input.trim();
                            match input {
                                "q" => quit!(),
                                "f" => match get_range!("forward") {
                                    Some(end) => self.display_memory(addr + 1..=addr + end),
                                    None => continue,
                                },
                                "b" => match get_range!("backward") {
                                    Some(end) => self.display_memory((addr - end..addr).rev()),
                                    None => continue,
                                },
                                _ => continue,
                            }
                        } else {
                            println!(
                                "BAD LOCATION VALUE, MUST BE BETWEEN 0 and {}",
                                self.memory.len() - 1
                            );
                        }
                    }
                }
            }
        }
    }

    fn display_memory<I>(&mut self, indicies: I)
    //TODO: refactor into Display trait on Memory
    where
        I: Iterator<Item = usize>,
    {
        for addr in indicies {
            if let Some(&reg) = self.memory.read(addr).ok() {
                println!(
                    "     the location {:4} has value {:016b} , or {1:5}  or signed {:6}",
                    addr,
                    u16::from(reg),
                    u16::from(reg).cast_signed()
                );
            }
        }
    }

    pub fn pulse(&mut self) -> Result<()> {
        match self.clock.subtick() {
            Subtick::Load => {
                if self.clock.tick() == 0 {
                    println!(
                        "Read in {} micro instructions",
                        self.read_micro_instructions
                    );
                    println!(
                        "Read in {} machine instructions",
                        self.read_machine_instructions
                    );
                    println!(
                        "{:<15}: {1:016b}  base 10: {1:7}",
                        "Starting PC is",
                        self.registers.pc()
                    );
                    println!(
                        "{:<15}: {1:016b}  base 10: {1:7}",
                        "Starting SP is",
                        self.registers.sp()
                    );
                    println!();
                };

                self.load()
            }
            Subtick::Gate => self.gate(),
            Subtick::Operation => self.calc(),
            Subtick::Store => self.store(),
        }

        if self.clock.subtick() == Subtick::Load {
            match (self.mir.rd(), self.mir.wr()) {
                (true, true) => {
                    // eprintln!("Should be halting");
                    self.halt()?;
                }
                (false, true) => {
                    self.memory
                        .write(self.mar as usize, self.mbr.into())
                        .expect("Never out of bounds");
                }
                (true, false) => {
                    self.mbr = (*self
                        .memory
                        .read(self.mar as usize)
                        .expect("Never read out of bounds"))
                    .into();
                }
                (false, false) => {}
            }
        }

        self.clock.pulse();
        Ok(())
    }

    pub fn from_args(args: Mic1Args) -> Result<Self, Mic1Error> {
        let program_file = File::open(args.program())?;
        let memory_vec = io::BufReader::new(program_file)
            .lines()
            .enumerate()
            .map(|(i, line_result)| {
                let line: String = line_result?;
                u16::from_str_radix(line.trim(), 2).map_err(|e| Mic1Error::ParseError {
                    line: i + 1,
                    content: line,
                    source: e,
                    file: FileType::Program,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let read_machine_instructions = memory_vec.len();
        if read_machine_instructions > Self::MEMORY_SIZE {
            return Err(Mic1Error::ProgramTooLarge {
                size: read_machine_instructions,
                max: Self::MEMORY_SIZE,
                file: FileType::Program,
            });
        }

        let memory = memory_vec
            .into_iter()
            .chain(iter::repeat(Default::default()))
            .take(Self::MEMORY_SIZE)
            .collect::<Vec<_>>()
            .try_into()
            .expect("Only took MEMORY_SIZE from iterator");

        let prom_vec = args
            .prom_data()
            .enumerate()
            .map(|(i, line)| {
                u32::from_str_radix(line.trim(), 2)
                    .map_err(|e| Mic1Error::ParseError {
                        line: i + 1,
                        content: line,
                        source: e,
                        file: FileType::Microcode,
                    })
                    .map(MicroInstruction::from_bits)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let read_micro_instructions = prom_vec.len();
        if read_micro_instructions > Self::MICROCODE_LENGTH {
            return Err(Mic1Error::ProgramTooLarge {
                size: read_micro_instructions,
                max: Self::MICROCODE_LENGTH,
                file: FileType::Microcode,
            });
        }

        let mir = prom_vec[0];
        let micro_code = prom_vec
            .into_iter()
            .chain(iter::repeat(Default::default()))
            .take(Self::MICROCODE_LENGTH) // Take exactly MICROCODE_LENGTH
            .collect::<Vec<_>>() // Collect to Vec temporarily
            .try_into()
            .expect("Only take MICROCODE_LENGTH from iterator"); // always safe since we took exactly MICROCODE_LENGTH

        Ok(Self {
            registers: Registers::new(args.stack_pointer(), args.program_counter()),
            memory,
            micro_code,
            mir,
            clock: Default::default(),
            blocking_io: false,
            micro_pc: Default::default(),
            a_bus: Default::default(),
            b_bus: Default::default(),
            c_bus: Default::default(),
            mbr: Default::default(),
            mar: Default::default(),

            read_machine_instructions: read_machine_instructions.try_into().unwrap(),
            read_micro_instructions: read_micro_instructions.try_into().unwrap(),
        })
    }
}

#[derive(Error, Debug)]
pub enum Mic1Error {
    #[error("failed to read program file: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid data in {file:?} at line {line}: '{content}'")]
    ParseError {
        line: usize,
        content: String,
        #[source]
        source: std::num::ParseIntError,
        file: FileType,
    },

    #[error("program size ({size}) exceeds maximum memory of {max}")]
    ProgramTooLarge {
        size: usize,
        max: usize,
        file: FileType,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum FileType {
    Microcode,
    Program,
}
