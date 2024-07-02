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
pub struct ProcessTableChip {
    config: ProcessorTableConfig,
}
#[derive(Clone)]

pub struct ProcessorTableConfig {
    pub clk: Column<Advice>,
    pub ip: Column<Advice>,
    pub ci: Column<Advice>,
    pub ni: Column<Advice>,
    pub mp: Column<Advice>,
    pub mv: Column<Advice>,
    pub mvi: Column<Advice>,
    pub s_b: Selector,
    pub s_c: Selector,
    pub s_p: Selector,
}
impl ProcessTableChip {
    pub fn construct(config: ProcessorTableConfig) -> Self {
        Self { config }
    }
    pub fn configure(meta: &mut ConstraintSystem<Fr>) -> ProcessorTableConfig {
        let clk = meta.advice_column();
        let ip = meta.advice_column();
        let ci = meta.advice_column();
        let ni = meta.advice_column();
        let mp = meta.advice_column();
        let mv = meta.advice_column();
        let mvi = meta.advice_column();
        let s_b = meta.selector();
        let s_c = meta.selector();
        let s_p = meta.selector();

        let instructions = [ADD, SUB, SHL, SHR, GETCHAR, PUTCHAR, LB, RB];

        let ZERO = Expression::Constant(Fr::ZERO);
        let ONE = Expression::Constant(Fr::ONE);
        let TWO = Expression::Constant(Fr::from(2));

        //Boundary Constraints
        meta.create_gate("boundary constraints", |meta| {
            let clk_cell = meta.query_advice(clk, Rotation::cur());
            let ip_cell = meta.query_advice(ip, Rotation::cur());
            let mp_cell = meta.query_advice(mp, Rotation::cur());
            let mv_cell = meta.query_advice(mv, Rotation::cur());
            let s = meta.query_selector(s_b);
            vec![
                s.clone() * clk_cell,
                s.clone() * ip_cell,
                s.clone() * mp_cell,
                s * mv_cell,
            ]
        });

        //Consistency Constraints
        meta.create_gate("Consistency constraints", |meta| {
            let mv_cell = meta.query_advice(mv, Rotation::cur());
            let mvi_cell = meta.query_advice(mvi, Rotation::cur());

            let s = meta.query_selector(s_c);
            vec![
                s.clone() * mv_cell.clone() * (mv_cell.clone() * mvi_cell.clone() - ONE.clone()),
                s * mvi_cell.clone() * (mv_cell * mvi_cell - ONE.clone()),
            ]
        });

        //transition Constraints
        meta.create_gate("procerssor table transition constraints", |meta| {
            let s_p_cell = meta.query_selector(s_p);
            let cur_ip_cell = meta.query_advice(ip, Rotation::cur());
            let next_ip_cell = meta.query_advice(ip, Rotation::next());
            let cur_mvi_cell = meta.query_advice(mvi, Rotation::cur());
            let cur_ni_cell = meta.query_advice(ni, Rotation::cur());
            let cur_mp_cell = meta.query_advice(mp, Rotation::cur());
            let next_mp_cell = meta.query_advice(mp, Rotation::next());
            let next_mv_cell = meta.query_advice(mv, Rotation::next());
            let cur_mv_cell = meta.query_advice(mv, Rotation::cur());
            let cur_clk_cell = meta.query_advice(clk, Rotation::cur());
            let next_clk_cell = meta.query_advice(clk, Rotation::next());

            let constraint_p1 = instructions
                .iter()
                .map(|&x| {
                    let deselector = create_deselector(x, &instructions);
                    deselector
                        * match x {
                            LB => {
                                cur_mv_cell.clone()
                                    * (next_ip_cell.clone() - cur_ip_cell.clone() - TWO.clone())
                                    + (cur_mv_cell.clone() * cur_mvi_cell.clone() - ONE.clone())
                                        * (next_ip_cell.clone() - cur_ni_cell.clone())
                            }

                            RB => {
                                (cur_mv_cell.clone() * cur_mvi_cell.clone() - ONE.clone())
                                    * (next_ip_cell.clone() - cur_ip_cell.clone() - TWO.clone())
                                    + cur_mv_cell.clone()
                                        * (next_ip_cell.clone() - cur_ni_cell.clone())
                            }

                            _ => next_ip_cell.clone() - cur_ip_cell.clone() - ONE.clone(),
                        }
                })
                .fold(ZERO.clone(), |acc, cur| acc + cur);

            let constraint_p2 = instructions
                .iter()
                .map(|&x| {
                    let deselector = create_deselector(x, &instructions);
                    deselector
                        * match x {
                            SHR => next_mp_cell.clone() - cur_mp_cell.clone() - ONE.clone(),
                            SHL => next_mp_cell.clone() - cur_mp_cell.clone() + ONE.clone(),
                            _ => next_mp_cell.clone() - cur_mp_cell.clone(),
                        }
                })
                .fold(ZERO.clone(), |acc, cur| acc + cur);

            let constraint_p3 = instructions
                .iter()
                .map(|&x| {
                    let deselector = create_deselector(x, &instructions);
                    deselector
                        * match x {
                            ADD => next_mv_cell.clone() - cur_mv_cell.clone() - ONE.clone(),
                            SUB => next_mv_cell.clone() - cur_mv_cell.clone() + ONE.clone(),
                            SHR | SHL | GETCHAR => ZERO.clone(),
                            LB | RB | PUTCHAR => next_mv_cell.clone() - cur_mv_cell.clone(),
                            _ => unreachable!(),
                        }
                })
                .fold(ZERO.clone(), |acc, cur| acc + cur);
            vec![
                s_p_cell.clone() * (next_clk_cell - cur_clk_cell - ONE),
                s_p_cell.clone() * constraint_p1,
                s_p_cell.clone() * constraint_p2,
                s_p_cell * constraint_p3,
            ]
        });

        ProcessorTableConfig {
            clk,
            ip,
            ci,
            ni,
            mp,
            mv,
            mvi,
            s_b,
            s_c,
            s_p,
        }
    }
    pub fn assign(
        &self,
        mut layouter: impl halo2_proofs::circuit::Layouter<Fr>,
        tables: &Tables,
    ) -> Result<(), halo2_proofs::plonk::ErrorFront> {
        layouter.assign_region(
            || "processor table",
            |mut region| {
                for (offset, row) in tables.processor_table.iter().enumerate() {
                    region.assign_advice(
                        || "clk",
                        self.config.clk,
                        offset,
                        || Value::known(Fr::from(row.clk)),
                    )?;
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
                    region.assign_advice(
                        || "mp",
                        self.config.mp,
                        offset,
                        || Value::known(Fr::from(row.mp as u64)),
                    )?;
                    region.assign_advice(
                        || "mv",
                        self.config.mv,
                        offset,
                        || Value::known(row.mv),
                    )?;
                    region.assign_advice(
                        || "ip",
                        self.config.ip,
                        offset,
                        || Value::known(Fr::from(row.ip as u64)),
                    )?;
                    if offset == 0 {
                        region.enable_selector(|| "s_b", &self.config.s_b, offset)?;
                    }
                    region.enable_selector(|| "s_c", &self.config.s_c, offset)?;
                    if offset != tables.processor_table.len() - 1 {
                        region.enable_selector(|| "s_p", &self.config.s_p, offset)?;
                    }
                }

                Ok(())
            },
        )
    }
}
fn create_deselector(instruction: u8, instructions: &[u8]) -> Expression<Fr> {
    let one = Expression::Constant(Fr::ONE);

    instructions
        .into_iter()
        .filter(|&&x| x != instruction)
        .fold(one, |acc, cur| {
            acc * (Expression::Constant(Fr::from(*cur as u64))
                - Expression::Constant(Fr::from(*cur as u64)))
        })
}
