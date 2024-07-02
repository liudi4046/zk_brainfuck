use std::env::consts;

use halo2_proofs::{
    arithmetic::Field,
    circuit::Value,
    halo2curves::bn256::Fr,
    plonk::{Advice, Column, ConstraintSystem, Expression, Selector},
    poly::Rotation,
};
use vm::{
    interpreter::{ADD, GETCHAR, LB, PUTCHAR, RB, SHL, SHR, SUB},
    table::Tables,
};
#[derive(Clone)]

pub struct InstructionTableConfig {
    pub ip: Column<Advice>,
    pub ci: Column<Advice>,
    pub ni: Column<Advice>,
    pub s_i: Selector,
}
pub struct InstructionTableChip {
    config: InstructionTableConfig,
}
impl InstructionTableChip {
    pub fn construct(config: InstructionTableConfig) -> Self {
        Self { config }
    }
    pub fn configure(meta: &mut ConstraintSystem<Fr>) -> InstructionTableConfig {
        let ip = meta.advice_column();
        let ci = meta.advice_column();
        let ni = meta.advice_column();
        let s_i = meta.selector();
        let ONE = Expression::Constant(Fr::ONE);
        meta.create_gate("instruction table transition constraints", |meta| {
            let ip_add_one = meta.query_advice(ip, Rotation::next())
                - meta.query_advice(ip, Rotation::cur())
                - ONE;
            let s_i_cell = meta.query_selector(s_i);

            vec![
                s_i_cell.clone()
                    * ip_add_one.clone()
                    * (meta.query_advice(ip, Rotation::next())
                        - meta.query_advice(ip, Rotation::cur())),
                s_i_cell.clone()
                    * ip_add_one.clone()
                    * (meta.query_advice(ci, Rotation::next())
                        - meta.query_advice(ci, Rotation::cur())),
                s_i_cell
                    * ip_add_one
                    * (meta.query_advice(ni, Rotation::next())
                        - meta.query_advice(ni, Rotation::cur())),
            ]
        });

        InstructionTableConfig { ip, ci, ni, s_i }
    }
    pub fn assign(
        &self,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fr>,
        tables: &Tables,
    ) -> Result<(), halo2_proofs::plonk::ErrorFront> {
        layouter.assign_region(
            || "instruction table",
            |mut region| {
                for (offset, row) in tables.processor_table.iter().enumerate() {
                    region.assign_advice(
                        || "ip",
                        self.config.ip,
                        offset,
                        || Value::known(Fr::from(row.ip as u64)),
                    )?;
                    region.assign_advice(
                        || "ci",
                        self.config.ci,
                        offset,
                        || Value::known(Fr::from(row.ci as u64)),
                    )?;
                    region.assign_advice(
                        || "ni",
                        self.config.ni,
                        offset,
                        || Value::known(Fr::from(row.ni as u64)),
                    )?;
                    if offset != tables.memory_table.len() - 1 {
                        region.enable_selector(|| "s_m", &self.config.s_i, offset)?;
                    }
                }

                Ok(())
            },
        )
    }
}
