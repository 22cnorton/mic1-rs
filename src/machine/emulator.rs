use crate::cli::Mic1Args;
use crate::io::MoloneyIOBits;
use crate::machine::clock::{Clock, Subtick};
use crate::memory::immutable::ImmutableMemory;
use crate::memory::traits::MutableMemory;
use crate::memory::{IOMemory, traits::ReadableMemory};
use crate::microcode::{self, MicroInstruction};
use crate::registers::{self, RegisterSize};
use anyhow::Result;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead};
use std::iter;
use thiserror::Error;


pub mod io_mem {
    use crate::memory::traits::IOMemoryType;

    use crate::io::MoloneyIOBits;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
    #[repr(transparent)]
    pub struct IOMem(u16);

    impl From<u16> for IOMem {
        fn from(value: u16) -> Self {
            Self(value)
        }
    }

    impl From<IOMem> for u16 {
        fn from(value: IOMem) -> Self {
            value.0
        }
    }

    impl From<u8> for IOMem {
        fn from(value: u8) -> Self {
            IOMem(value as u16)
        }
    }

    impl From<IOMem> for u8 {
        fn from(value: IOMem) -> Self {
            value.0 as u8
        }
    }

    impl From<MoloneyIOBits> for IOMem {
        fn from(value: MoloneyIOBits) -> Self {
            Self(value.into())
        }
    }

    impl From<IOMem> for MoloneyIOBits {
        fn from(value: IOMem) -> Self {
            value.0.into()
        }
    }

    impl IOMemoryType for IOMem {}
}

const ARCH_IO_MEM_SIZE: usize = 0x1000;
type ArchIOMem<const S: usize> = IOMemory<
    io_mem::IOMem,
    MoloneyIOBits,
    ARCH_IO_MEM_SIZE,
    { ARCH_IO_MEM_SIZE - 1 },
    { ARCH_IO_MEM_SIZE - 2 },
    { ARCH_IO_MEM_SIZE - 3 },
    { ARCH_IO_MEM_SIZE - 4 },
>;
// self.micro_code.iter().enumerate().for_each(|(i, instr)| { // TODO: figure out how to make the micro_code print this for debuging
//                         println!("Addr: {:02X}  Instr: {:?}", i, instr);
//                     });

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub struct Machine {
    // TODO: make generic for different memory types
    memory: ArchIOMem<{ Self::MEMORY_SIZE }>,
    micro_code: ImmutableMemory<MicroInstruction, { Self::MICROCODE_LENGTH }>,

    registers: registers::Registers,
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
    pub const MICROCODE_LENGTH: usize = 256;
    #[allow(dead_code)]
    pub fn current_instruction(&self) -> &io_mem::IOMem {
        ReadableMemory::read(&self.memory, self.registers.pc as usize)
            .expect("Never read out of bounds")
    }
    #[allow(dead_code)]
    pub fn current_micro_instruction(&self) -> MicroInstruction {
        self.mir
    }

    fn instruction_at(&self, addr: u8) -> MicroInstruction {
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

        let c_value = self.alu(a_value, b_value, self.mir.alu());
        self.micro_pc = self.next_micro_instruction(c_value, self.mir.cond());

        self.c_bus = self.shift(c_value, self.mir.sh());
        if self.mir.mbr() {
            self.mbr = self.c_bus;
        }
    }

    fn alu(
        &self,
        a_value: RegisterSize,
        b_value: RegisterSize,
        op: microcode::Operation,
    ) -> RegisterSize {
        match op {
            microcode::Operation::Add => a_value.wrapping_add(b_value),
            microcode::Operation::And => a_value & b_value,
            microcode::Operation::Assign => a_value,
            microcode::Operation::Invert => !a_value,
        }
    }

    fn shift(&self, value: RegisterSize, sh: microcode::Shift) -> RegisterSize {
        match sh {
            microcode::Shift::None => value,
            microcode::Shift::Left => value << 1,
            microcode::Shift::Right => value >> 1,
        }
    }

    fn next_micro_instruction(&self, value: RegisterSize, cond: microcode::Jump) -> u8 {
        match cond {
            microcode::Jump::None => self.micro_pc.saturating_add(1),
            microcode::Jump::Negative => {
                if (value as i16) < 0 {
                    self.mir.addr()
                } else {
                    self.micro_pc.saturating_add(1)
                }
            }
            microcode::Jump::Zero => {
                if value == 0 {
                    self.mir.addr()
                } else {
                    self.micro_pc.saturating_add(1)
                }
            }
            microcode::Jump::Always => self.mir.addr(),
        }
    }

    fn store(&mut self) {
        if self.mir.enc() {
            self.registers.write_to_reg(self.mir.c(), self.c_bus);
            // eprintln!("PC: {:016b}", self.registers.pc);
            // eprintln!(
            //     "Writing value {:016b} to register index {}",
            //     self.c_bus,
            //     self.mir.c()
            // );
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
                    self.clock.set_subtick (Subtick::Load); // Reset subtick to Load for next instruction
                    break;
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
        Ok(())
    }

    fn display_memory<I>(&self, indicies: I)
    //TODO: refactor into Display trait on Memory
    where
        I: Iterator<Item = usize>,
    {
        for addr in indicies {
            let reg = ReadableMemory::read(&self.memory, addr)
                .map_or_else(|(value, _)| value.expect("Not out of bounds"), |&v| v);
            // .expect("Not out of bounds");
            println!(
                "     the location {:4} has value {:016b} , or {1:5}  or signed {:6}",
                addr,
                u16::from(reg),
                u16::from(reg).cast_signed()
            );
        }
    }

    pub fn pulse(&mut self) -> Result<()> {
        match self.clock.subtick(){
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
                        "Starting PC is", self.registers.pc
                    );
                    println!(
                        "{:<15}: {1:016b}  base 10: {1:7}",
                        "Starting SP is", self.registers.sp
                    );
                    println!();
                };

                self.load()
            }
            Subtick::Gate => self.gate(),
            Subtick::Operation => self.calc(),
            Subtick::Store => self.store(),
        }
        // (&mut self.memory).action();//TODO: figure out how

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
                    self.mbr = (*MutableMemory::read(&mut self.memory, self.mar as usize)
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
            .map(io_mem::IOMem::from)
            .chain(iter::repeat(Default::default()))
            .take(Self::MEMORY_SIZE)
            .collect::<Vec<_>>()
            .try_into()
            .expect("Only took MEMORY_SIZE from iterator");

        let prom_file = File::open(args.prom())?;
        let prom_vec = io::BufReader::new(prom_file)
            .lines()
            .enumerate()
            .filter(|(_, line)| match line {
                Ok(l) => l.chars().all(|c| c == '0' || c == '1'),
                Err(_) => false,
            })
            .map(|(i, line_result)| {
                let line = line_result?;
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
            registers: registers::Registers {
                sp: args.stack_pointer(),
                pc: args.program_counter(),
                ..Default::default()
            },
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
