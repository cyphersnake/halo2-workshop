use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    pasta::group::ff::PrimeField,
    plonk::{Circuit, ConstraintSystem},
};

// Sets the circuit, and also stores the private input
struct BracketCircuit<const L: usize, F: PrimeField> {
    _input: [char; L],
    _p: PhantomData<F>,
}

impl<const L: usize, F: PrimeField> BracketCircuit<L, F> {
    pub fn new(input: [char; L]) -> Self {
        Self {
            _input: input,
            _p: PhantomData,
        }
    }
}

// Stores the configuration of the table (columns) that the circuit needs
#[derive(Clone)]
struct Config {}

impl<const L: usize, F: PrimeField> Circuit<F> for BracketCircuit<L, F> {
    type Config = Config;

    // Not important at this stage
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        todo!("Not needed at this stage.")
    }

    fn configure(_meta: &mut ConstraintSystem<F>) -> Self::Config {
        todo!(
            "This specifies the table structure:
            - columns
            - gates (constraints)
            - lookup tables"
        )
    }

    fn synthesize(
        &self,
        _config: Self::Config,
        _layouter: impl Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        todo!("This is where the table cells will be filled in")
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{dev::MockProver, pasta::Fq};

    use super::*;

    const K: u32 = 10;

    #[test]
    fn valid_1() {
        MockProver::run(K, &BracketCircuit::<2, Fq>::new(['(', ')']), vec![]).unwrap();
    }
}
