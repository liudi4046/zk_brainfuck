use std::env::consts;

use gadgets::less_than::{LtChip, LtConfig, LtInstruction};
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

pub struct InputTableConfig {
    pub clk: Column<Advice>,
    pub value: Column<Instance>,
}
pub struct InputTableChip {
    config: InputTableConfig,
}
impl InputTableChip {
    pub fn construct(config: InputTableConfig) -> Self {
        Self { config }
    }
    pub fn configure(meta: &mut ConstraintSystem<Fr>) -> InputTableConfig {
        let clk = meta.advice_column();
        let value = meta.instance_column();
        let s = meta.complex_selector();

        // let lt_config = LtChip::configure(
        //     meta,
        //     |cell| cell.query_selector(selector),
        //     |cell| cell.query_advice(clk, Rotation::cur()),
        //     |cell| cell.query_advice(clk, Rotation::next()),
        // );

        InputTableConfig { clk, value }
    }
    pub fn assign(
        &self,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fr>,
        tables: &Tables,
    ) -> Result<(), halo2_proofs::plonk::ErrorFront> {
        layouter.assign_region(
            || "input table",
            |mut region| {
                for (offset, row) in tables.input_table.iter().enumerate() {
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
