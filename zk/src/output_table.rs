use std::env::consts;

use halo2_proofs::{
    arithmetic::Field,
    circuit::Value,
    halo2curves::bn256::Fr,
    plonk::{Advice, Column, ConstraintSystem, Expression, Instance, Selector},
    poly::Rotation,
};
use vm::{
    interpreter::{ADD, GETCHAR, LB, PUTCHAR, RB, SHL, SHR, SUB},
    table::Tables,
};
#[derive(Clone)]

pub struct OutputTableConfig {
    pub clk: Column<Advice>,
    pub value: Column<Instance>,
}
pub struct OutputTableChip {
    config: OutputTableConfig,
}
impl OutputTableChip {
    pub fn construct(config: OutputTableConfig) -> Self {
        Self { config }
    }
    pub fn configure(meta: &mut ConstraintSystem<Fr>) -> OutputTableConfig {
        let clk = meta.advice_column();
        let value = meta.instance_column();

        OutputTableConfig { clk, value }
    }
    pub fn assign(
        &self,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fr>,
        tables: &Tables,
    ) -> Result<(), halo2_proofs::plonk::ErrorFront> {
        layouter.assign_region(
            || "Output table",
            |mut region| {
                for (offset, row) in tables.output_table.iter().enumerate() {
                    region.assign_advice(
                        || "clk",
                        self.config.clk,
                        offset,
                        || Value::known(Fr::from(row.clk)),
                    )?;
                }

                Ok(())
            },
        )
    }
}
