use std::fmt;

use halo2_proofs::halo2curves::bn256::Fr;

use crate::register::{self, Registers};

#[derive(Default, Clone)]
pub struct Tables {
    pub processor_table: Vec<ProcessTableRow>,
    pub memory_table: Vec<MemoryTableRow>,
    pub instruction_table: Vec<InstructionTableRow>,
    pub input_table: Vec<InputTableRow>,
    pub output_table: Vec<OutputTableRow>,
}

impl Tables {}
#[derive(Clone)]
pub struct ProcessTableRow {
    pub clk: u64,
    pub ip: usize,
    pub ci: u8,
    pub ni: u8,
    pub mp: usize,
    pub mv: Fr,
    pub mvi: Fr,
}
impl fmt::Debug for ProcessTableRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "ProcessTableRow {{ clk: {}, ip: {}, ci: {}, ni: {}, mp: {}, mv: {:?}, mvi: {:?} }}\n",
            self.clk, self.ip, self.ci as char, self.ni as char, self.mp, self.mv, self.mvi
        )
    }
}
impl From<Registers> for ProcessTableRow {
    fn from(registers: Registers) -> Self {
        Self {
            clk: registers.clk,
            ip: registers.ip,
            ci: registers.ci,
            ni: registers.ni,
            mp: registers.mp,
            mv: registers.mv,
            mvi: registers.mvi,
        }
    }
}
#[derive(Clone)]
pub struct MemoryTableRow {
    pub clk: u64,
    pub mp: usize,
    pub mv: Fr,
}
impl fmt::Debug for MemoryTableRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "MemoryTableRow {{ clk: {},mp: {}, mv: {:?} }}\n",
            self.clk, self.mp, self.mv
        )
    }
}
#[derive(Clone)]

pub struct InstructionTableRow {
    pub ip: usize,
    pub ci: u8,
    pub ni: u8,
}
impl fmt::Debug for InstructionTableRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "InstructionTableRow {{ ip: {},ci: {}, ni: {} }}\n",
            self.ip, self.ci as char, self.ni as char
        )
    }
}
#[derive(Clone)]
pub struct InputTableRow {
    pub clk: u64,
    pub value: Fr,
}
#[derive(Clone)]
pub struct OutputTableRow {
    pub clk: u64,
    pub value: Fr,
}
