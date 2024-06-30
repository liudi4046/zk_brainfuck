use std::default;

use halo2_proofs::halo2curves::bn256::Fr;

#[derive(Default, Clone, Debug)]
pub struct Registers {
    pub clk: u64,
    pub ip: usize,
    pub ci: u8,
    pub ni: u8,
    pub mp: usize,
    pub mv: Fr,
    pub mvi: Fr,
}
//12 + 18+1 = 31
