use std::mem;

use halo2_proofs::halo2curves::bn256::Fr;

use crate::{
    register::{self, Registers},
    table::{
        InputTableRow, InstructionTableRow, MemoryTableRow, OutputTableRow, ProcessTableRow, Tables,
    },
};

pub struct Interpreter {
    code: Vec<u8>,
    program: Vec<u8>,
    registers: Registers,
    tables: Tables,
    memory: Vec<Fr>,
    input: Vec<Fr>,
    output: Vec<Fr>,
}
pub const SHL: u8 = 60;
pub const SHR: u8 = 62;
pub const ADD: u8 = 43;
pub const SUB: u8 = 45;
pub const GETCHAR: u8 = 44;
pub const PUTCHAR: u8 = 46;
pub const LB: u8 = 91;
pub const RB: u8 = 93;

impl Interpreter {
    pub fn new(code: Vec<u8>, input: Vec<Fr>) -> Self {
        Self {
            program: compile_code(&code),
            code,
            registers: Registers::default(),
            tables: Tables::default(),
            memory: vec![Fr::zero(); 50],
            input,
            output: Vec::new(),
        }
    }
    pub fn run(&mut self) {
        while self.registers.ip < self.program.len() {
            let instruction = self.program[self.registers.ip];
            self.registers.mv = self.memory[self.registers.mp];
            self.registers.mvi = self.registers.mv.invert().unwrap_or(Fr::zero());
            self.registers.ci = self.program[self.registers.ip];
            self.registers.ni = self.program[self.registers.ip + 1];

            self.tables
                .processor_table
                .push(ProcessTableRow::from(self.registers.clone()));

            match instruction {
                SHL => {
                    self.registers.mp -= 1;
                    self.registers.ip += 1;
                }
                SHR => {
                    self.registers.mp += 1;
                    self.registers.ip += 1;
                }
                ADD => {
                    self.memory[self.registers.mp] += Fr::one();
                    self.registers.ip += 1;
                }
                SUB => {
                    self.memory[self.registers.mp] -= Fr::one();
                    self.registers.ip += 1;
                }
                GETCHAR => {
                    let input_num = self.input.remove(0);
                    self.tables.input_table.push(InputTableRow {
                        clk: self.registers.clk,
                        value: input_num,
                    });
                    self.memory[self.registers.mp] = input_num;
                    self.registers.ip += 1;
                }
                PUTCHAR => {
                    let output_num = self.registers.mv;
                    self.tables.output_table.push(OutputTableRow {
                        clk: self.registers.clk,
                        value: output_num,
                    });
                    self.registers.ip += 1;
                }
                LB => {
                    //program:: ++>,<[14>+.<-]7
                    if self.registers.mv != Fr::zero() {
                        self.registers.ip += 2;
                    } else {
                        self.registers.ip = self.program[self.registers.ip + 1] as usize;
                    }
                }
                RB => {
                    if self.registers.mv != Fr::zero() {
                        self.registers.ip = self.program[self.registers.ip + 1] as usize;
                    } else {
                        self.registers.ip += 2;
                    }
                }
                _ => unreachable!(),
            }
            self.registers.clk += 1;
        }
        self.registers.ci = 0;
        self.registers.ni = 0;
        self.tables
            .processor_table
            .push(ProcessTableRow::from(self.registers.clone()));

        //memory table
        self.tables.memory_table = self
            .tables
            .processor_table
            .iter()
            .map(|row| MemoryTableRow {
                clk: row.clk,
                mp: row.mp,
                mv: row.mv,
            })
            .collect();

        self.tables
            .memory_table
            .sort_by(|a, b| match a.mp.cmp(&b.mp) {
                std::cmp::Ordering::Equal => a.clk.cmp(&b.clk),
                other => other,
            });

        //intruction table
        self.tables.instruction_table = self
            .tables
            .processor_table
            .iter()
            .map(|row| InstructionTableRow {
                ip: row.ip,
                ci: row.ci,
                ni: row.ni,
            })
            .collect();
        //program:: ++>,<[14>+.<-]7
        let mut program = Vec::new();
        for (i, &value) in self.program.iter().enumerate() {
            if is_insturction(value) {
                let ni = if i + 1 < self.program.len() {
                    self.program[i + 1]
                } else {
                    0
                };
                program.push(InstructionTableRow {
                    ip: i,
                    ci: value,
                    ni,
                })
            }
        }
        // println!("program :{:?}", program);
        self.tables.instruction_table.append(&mut program);
        self.tables
            .instruction_table
            .sort_by(|a, b| a.ip.cmp(&b.ip));
        // println!("processor table:{:?}", self.tables.processor_table);

        // println!("instruction table:{:?}", self.tables.instruction_table);
    }
}
// fn fr_to_usize(num: Fr) -> usize {
//     let mut slice = [0u8; 8];
//     slice.copy_from_slice(&num.to_bytes()[..8]);
//     usize::from_le_bytes(slice)
// }

fn compile_code(code: &[u8]) -> Vec<u8> {
    let mut program = Vec::new();
    let mut stack: Vec<usize> = Vec::new();

    for &item in code.iter() {
        program.push(item);
        if item == LB || item == RB {
            program.push(0);
        }
    }
    for (index, &item) in program.clone().iter().enumerate() {
        if item == LB {
            stack.push(index);
        }
        if item == RB {
            let lb_index = stack.pop();
            if let Some(lb_index) = lb_index {
                program[lb_index + 1] = index as u8 + 2;
                program[index + 1] = lb_index as u8 + 2;
            }
        }
    }
    program
}

fn is_insturction(value: u8) -> bool {
    matches!(value, SHL | SHR | ADD | SUB | GETCHAR | PUTCHAR | LB | RB)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_run() {
        let code = vec![
            ADD, ADD, SHR, GETCHAR, SHL, LB, SHR, ADD, PUTCHAR, SHL, SUB, RB,
        ];
        let input = vec![Fr::from(97)];
        let mut interpreter = Interpreter::new(code, input);
        interpreter.run();
    }

    // #[test]
    // fn test_basic_operations() {
    //     let code = vec![ADD, ADD, SHR, SUB];
    //     assert_eq!(compile_code(&code), vec![43, 43, 62, 45]);
    // }

    // #[test]
    // fn test_nested_loops() {
    //     let code = vec![
    //         ADD, GETCHAR, SHR, ADD, LB, SHR, ADD, LB, ADD, RB, SHR, SUB, RB,
    //     ];
    //     let compiled = compile_code(&code);
    //     let expected = vec![
    //         ADD, GETCHAR, SHR, ADD, LB, 17, SHR, ADD, LB, 13, ADD, RB, 10, SHR, SUB, RB, 6,
    //     ];
    //     assert_eq!(compiled, expected);
    // }

    // #[test]
    // fn test_user_example() {
    //     let code = vec![ADD, LB, SHR, ADD, SHL, SUB, RB, ADD];
    //     let compiled = compile_code(&code);
    //     let expected = vec![43, 91, 9, 62, 43, 60, 45, 93, 3, 43];

    //     assert_eq!(compiled, expected);
    // }
}
