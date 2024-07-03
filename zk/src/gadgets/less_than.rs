use halo2_proofs::{
    halo2curves::bn256::Fr,
    plonk::{Advice, Column, ConstraintSystem, Expression, Fixed, VirtualCells},
    poly::Rotation,
};
pub struct LtConfig<const N_BYTES: usize> {
    pub lt: Column<Advice>,
    pub diff: [Column<Advice>; N_BYTES],
    pub u8: Column<Fixed>,
    pub range: Fr,
}
impl<const N_BYTES: usize> LtConfig<N_BYTES> {
    pub fn is_lt(&self, meta: &mut VirtualCells<Fr>) -> Expression<Fr> {
        meta.query_advice(self.lt, Rotation::cur())
    }
}

pub struct LtChip<const N_BYTES: usize> {
    config: LtConfig<N_BYTES>,
}

impl<const N_BYTES: usize> LtChip<N_BYTES> {
    pub fn configure(
        meta: &mut ConstraintSystem<Fr>,
        q_enable: impl FnOnce(&mut VirtualCells<Fr>) -> Expression<Fr>,
        lhs: impl FnOnce(&mut VirtualCells<Fr>) -> Expression<Fr>,
        rhs: impl FnOnce(&mut VirtualCells<Fr>) -> Expression<Fr>,
    ) -> LtConfig<N_BYTES> {
        let lt = meta.advice_column();
        let diff = [(); N_BYTES].map(|_| meta.advice_column());
        let range = Fr::from(2u64.pow(N_BYTES as u32 * 8));
        let u8 = meta.fixed_column();

        meta.create_gate("lt gate", |meta| {
            let q_enable = q_enable(meta);
            let lt = meta.query_advice(lt, Rotation::cur());

            let diff_bytes = diff
                .iter()
                .map(|c| meta.query_advice(*c, Rotation::cur()))
                .collect::<Vec<Expression<Fr>>>();

            let check_a =
                lhs(meta) - rhs(meta) - expr_from_bytes(&diff_bytes) + (lt.clone() * range);

            let check_b = bool_check(lt);

            [check_a, check_b]
                .into_iter()
                .map(move |poly| q_enable.clone() * poly)
        });

        meta.annotate_lookup_any_column(u8, || "LOOKUP_u8");

        diff[0..N_BYTES].iter().for_each(|column| {
            meta.lookup_any("range check for u8", |meta| {
                let u8_cell = meta.query_advice(*column, Rotation::cur());
                let u8_range = meta.query_fixed(u8, Rotation::cur());
                vec![(u8_cell, u8_range)]
            });
        });

        LtConfig {
            lt,
            diff,
            range,
            u8,
        }
    }

    pub fn construct(config: LtConfig<N_BYTES>) -> LtChip<N_BYTES> {
        LtChip { config }
    }
}
pub fn expr_from_bytes(bytes: &[Expression<Fr>]) -> Expression<Fr> {
    let mut value = Expression::Constant(Fr::zero());
    let mut multiplier = Fr::one();
    for byte in bytes.iter() {
        value = value + byte.clone() * multiplier;
        multiplier *= Fr::from(256);
    }
    value
}
fn bool_check(value: Expression<Fr>) -> Expression<Fr> {
    value.clone() * (value - Expression::Constant(Fr::one()))
}
