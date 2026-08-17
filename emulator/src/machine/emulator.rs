use crate::{
    machine::{
        clock::{Clock, Subtick},
        microcode::{self, MicroInstruction},
        registers::{RegisterSize, Registers},
    },
    memory::traits::Memory,
    messages::Command,
};
use crate::{
    memory::{
        immutable::ImmutableMemory,
        io_memory::IOMemory,
        traits::{ReadableMemory, WritableMemory},
    },
    messages::Event,
};
use anyhow::{Result, bail};
use derive_builder::Builder;
use flume::{Receiver, Sender};
use getset::Getters;
use std::fmt::Debug;

const MICROCODE_LENGTH: usize = 256;
#[derive(Debug, Clone, Builder)]
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
    #[builder(default = "self.default_mir()?")]
    mir: MicroInstruction,
    micro_pc: u8,
    a_bus: RegisterSize,
    b_bus: RegisterSize,
    c_bus: RegisterSize,
    mbr: RegisterSize,
    mar: RegisterSize, // Retype since this can only be twelve bits
}

pub enum MachineState {
    Halted,
    Running,
}

#[allow(dead_code)]
impl Machine {
    pub fn current_instruction(&mut self) -> u16 {
        *self
            .memory
            .read(*self.registers.pc() as usize)
            .expect("Never read out of bounds")
    }

    pub fn current_micro_instruction(&self) -> MicroInstruction {
        self.mir
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn microcode(&self) -> Vec<MicroInstruction> {
        (&self.micro_code).into()
    }

    pub fn registers(&self) -> &Registers {
        &self.registers
    }

    pub fn get_memory(
        &mut self,
        indicies: impl IntoIterator<Item = usize>,
    ) -> Vec<(usize, <IOMemory as Memory>::MemoryType)> {
        let mut data = vec![];
        for addr in indicies {
            if let Ok(&reg) = self.memory.read(addr) {
                data.push((addr, reg));
            }
        }

        data
    }
}

impl Machine {
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
        self.a_bus = *self.registers.read(self.mir.a() as usize).unwrap();
        self.b_bus = *self.registers.read(self.mir.b() as usize).unwrap();
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
            self.registers.write(self.mir.c() as usize, self.c_bus);
        }
        if self.mir.mbr() {
            self.mbr = self.c_bus;
        }
    }

    fn halt(&mut self) {
        self.blocking_io = true;
    }

    pub fn r#continue(&mut self) {
        self.blocking_io = false;
        self.micro_pc = 0;

        self.clock.set_tick(self.clock.tick().saturating_add(1));
        self.clock.set_subtick(Subtick::Load); // Reset subtick to Load for next instruction

        self.registers.set_pc(self.registers.pc().saturating_add(1));
    }

    pub fn pulse(&mut self) -> Result<MachineState> {
        match self.clock.subtick() {
            Subtick::Load => self.load(),
            Subtick::Gate => self.gate(),
            Subtick::Operation => self.calc(),
            Subtick::Store => self.store(),
        }

        if self.clock.subtick().is_load() {
            match (self.mir.rd(), self.mir.wr()) {
                (true, true) => {
                    self.halt();
                    return Ok(MachineState::Halted);
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

        Ok(MachineState::Running)
    }
}

impl MachineBuilder {
    fn default_mir(&self) -> Result<MicroInstruction, String> {
        let mir = self
            .micro_code
            .as_ref()
            .ok_or("Missing Microcode")?
            .immutable_read(0)
            .ok_or("Failed Read")?;
        Ok(*mir)
    }
}
