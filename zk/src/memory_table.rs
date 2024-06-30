use std::env::consts;

use halo2_proofs::{
    arithmetic::Field,
    halo2curves::bn256::Fr,
    plonk::{Advice, Column, ConstraintSystem, Expression, Selector},
    poly::Rotation,
};
use vm::interpreter::{ADD, GETCHAR, LB, PUTCHAR, RB, SHL, SHR, SUB};
struct MemoryTableConfig {
    clk: Column<Advice>,
    mp: Column<Advice>,
    mv: Column<Advice>,
    s_m: Selector,
}
struct MemoryTableChip {
    config: MemoryTableConfig,
}
impl MemoryTableChip {
    pub fn construct(config: MemoryTableConfig) -> Self {
        Self { config }
    }
    pub fn configure(meta: &mut ConstraintSystem<Fr>) -> MemoryTableConfig {
        let clk = meta.advice_column();
        let mp = meta.advice_column();
        let mv = meta.advice_column();
        let s_m = meta.selector();
        let ZERO = Expression::Constant(Fr::ZERO);
        let ONE = Expression::Constant(Fr::ONE);
        let TWO = Expression::Constant(Fr::from(2));

        meta.create_gate("memory table transition constraints", |meta| {
            let cur_mp_cell = meta.query_advice(mp, Rotation::cur());
            let next_mp_cell = meta.query_advice(mp, Rotation::next());
            let next_mv_cell = meta.query_advice(mv, Rotation::next());
            let cur_mv_cell = meta.query_advice(mv, Rotation::cur());
            let cur_clk_cell = meta.query_advice(clk, Rotation::cur());
            let next_clk_cell = meta.query_advice(clk, Rotation::next());
            let s_m_cell = meta.query_selector(s_m);

            let constraint_m0 = (next_mp_cell.clone() - cur_mp_cell.clone() - ONE.clone())
                * (next_mp_cell.clone() - cur_mp_cell.clone());
            let constraint_m1 = (next_mp_cell.clone() - cur_mp_cell.clone() - ONE.clone())
                * (next_mv_cell.clone() - cur_mv_cell)
                * (next_clk_cell - cur_clk_cell - ONE);
            let constraint_m2 = (next_mp_cell - cur_mp_cell) * next_mv_cell;
            vec![
                s_m_cell.clone() * constraint_m0,
                s_m_cell.clone() * constraint_m1,
                s_m_cell * constraint_m2,
            ]
        });
        meta.loo

        MemoryTableConfig { clk, mp, mv, s_m }
    }
}
