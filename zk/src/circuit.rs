use halo2_proofs::{
    arithmetic::Field,
    circuit::SimpleFloorPlanner,
    halo2curves::bn256::Fr,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Expression, Selector},
    poly::Rotation,
};
use vm::{
    interpreter::{ADD, GETCHAR, LB, PUTCHAR, RB, SHL, SHR, SUB},
    table::Tables,
};

use crate::{
    input_table::{InputTableChip, InputTableConfig},
    instruction_table::{self, InstructionTableChip, InstructionTableConfig},
    memory_table::{self, MemoryTableChip, MemoryTableConfig},
    output_table::{OutputTableChip, OutputTableConfig},
    processor_table::{self, ProcessTableChip, ProcessorTableConfig},
};
#[derive(Clone)]
struct BrainfuckConfig {
    processor_table: ProcessorTableConfig,
    memory_table: MemoryTableConfig,
    instruction_table: InstructionTableConfig,
    input_table: InputTableConfig,
    output_table: OutputTableConfig,
}
#[derive(Default)]
struct BrainfuckCircuit {
    tables: Tables,
}
impl Circuit<Fr> for BrainfuckCircuit {
    type Config = BrainfuckConfig;
    type FloorPlanner = SimpleFloorPlanner;
    fn without_witnesses(&self) -> Self {
        Self::default()
    }
    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        let processor_table = ProcessTableChip::configure(meta);
        let memory_table = MemoryTableChip::configure(meta);
        let instruction_table = InstructionTableChip::configure(meta);
        let input_table = InputTableChip::configure(meta);
        let output_table = OutputTableChip::configure(meta);

        meta.lookup_any("memory table permutation constraints", |meta| {
            let processor_clk = meta.query_advice(processor_table.clk, Rotation::cur());
            let processor_mp = meta.query_advice(processor_table.mp, Rotation::cur());
            let processor_mv = meta.query_advice(processor_table.mv, Rotation::cur());
            let memory_clk = meta.query_advice(memory_table.clk, Rotation::cur());
            let memory_mp = meta.query_advice(memory_table.mp, Rotation::cur());
            let memory_mv = meta.query_advice(memory_table.mv, Rotation::cur());
            vec![
                (memory_clk, processor_clk),
                (memory_mp, processor_mp),
                (memory_mv, processor_mv),
            ]
        });

        meta.lookup_any("instruction table permutation constraints", |meta| {
            let processor_ip = meta.query_advice(processor_table.ip, Rotation::cur());
            let processor_ci = meta.query_advice(processor_table.ci, Rotation::cur());
            let processor_ni = meta.query_advice(processor_table.ni, Rotation::cur());
            let instruction_ip = meta.query_advice(instruction_table.ip, Rotation::cur());
            let instruction_ci = meta.query_advice(instruction_table.ci, Rotation::cur());
            let instruction_ni = meta.query_advice(instruction_table.ni, Rotation::cur());
            vec![
                (instruction_ip, processor_ip),
                (instruction_ci, processor_ci),
                (instruction_ni, processor_ni),
            ]
        });
        meta.lookup_any("permutation constraint: input table", |meta| {
            let processor_clk = meta.query_advice(processor_table.clk, Rotation::cur());
            let processor_mv = meta.query_advice(processor_table.mv, Rotation::cur());
            let input_clk = meta.query_advice(input_table.clk, Rotation::cur());
            let input_value = meta.query_instance(input_table.value, Rotation::cur());
            vec![(input_clk, processor_clk), (input_value, processor_mv)]
        });
        meta.lookup_any("permutation constraint: output table", |meta| {
            let processor_clk = meta.query_advice(processor_table.clk, Rotation::cur());
            let processor_mv = meta.query_advice(processor_table.mv, Rotation::cur());
            let output_clk = meta.query_advice(output_table.clk, Rotation::cur());
            let output_value = meta.query_instance(output_table.value, Rotation::cur());
            vec![(output_clk, processor_clk), (output_value, processor_mv)]
        });

        Self::Config {
            processor_table,
            memory_table,
            instruction_table,
            input_table,
            output_table,
        }
    }
    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fr>,
    ) -> Result<(), halo2_proofs::plonk::ErrorFront> {
        let processor_chip = ProcessTableChip::construct(config.processor_table);
        let memory_chip = MemoryTableChip::construct(config.memory_table);
        let instruction_chip = InstructionTableChip::construct(config.instruction_table);
        let input_chip = InputTableChip::construct(config.input_table);
        let output_chip = OutputTableChip::construct(config.output_table);

        processor_chip.assign(layouter.namespace(|| "processor table"), &self.tables)?;
        memory_chip.assign(layouter.namespace(|| "memory table"), &self.tables)?;
        instruction_chip.assign(layouter.namespace(|| "instruction table"), &self.tables)?;
        input_chip.assign(layouter.namespace(|| "input table"), &self.tables)?;
        output_chip.assign(layouter.namespace(|| "output table"), &self.tables)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use halo2_proofs::dev::MockProver;
    use vm::interpreter::Interpreter;

    use super::*;

    #[test]
    fn test_run() {
        let code = vec![
            ADD, ADD, SHR, GETCHAR, SHL, LB, SHR, ADD, PUTCHAR, SHL, SUB, RB,
        ];
        let input = vec![Fr::from(97)];
        let mut interpreter = Interpreter::new(code, input);
        interpreter.run();
        let tables = interpreter.tables;
        let circuit = BrainfuckCircuit {
            tables: tables.clone(),
        };
        let input_val = tables
            .clone()
            .input_table
            .iter()
            .map(|v| v.value)
            .collect::<Vec<Fr>>();
        let output_val = tables
            .clone()
            .output_table
            .iter()
            .map(|v| v.value)
            .collect::<Vec<Fr>>();
        let prover = MockProver::run(9, &circuit, vec![output_val, input_val]).unwrap();
        prover.assert_satisfied();
    }
}
