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

use crate::gadgets::less_than::{LtChip, LtConfig};
#[derive(Clone)]

pub struct OutputTableConfig {
    pub clk: Column<Advice>,
    pub value: Column<Instance>,
    pub s: Selector,
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
        let s = meta.selector();

        let lt_config: LtConfig<4> = LtChip::configure(
            meta,
            |cell| cell.query_selector(s),
            |cell| cell.query_advice(clk, Rotation::cur()),
            |cell| cell.query_advice(clk, Rotation::next()),
        );
        meta.create_gate("cur clk < next clk", |meta| {
            let s = meta.query_selector(s);
            let lt = meta.query_advice(lt_config.lt, Rotation::cur());
            vec![s * (lt - Expression::Constant(Fr::one()))]
        });

        OutputTableConfig { clk, value, s }
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
                    if offset != tables.output_table.len() - 1 {
                        region.enable_selector(|| "output selector", &self.config.s, offset)?;
                    }
                }

                Ok(())
            },
        )
    }
}
