use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner},
    pasta::group::ff::PrimeField,
    plonk::{Circuit, ConstraintSystem},
};

// Sets the circuit, and also stores the private input
#[derive(Default)]
struct BracketCircuit<F: PrimeField> {
    _p: PhantomData<F>,
}

// Stores the configuration of the table (columns) that the circuit needs
#[derive(Clone)]
struct Config {}

impl<F: PrimeField> Circuit<F> for BracketCircuit<F> {
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

    #[test]
    fn simple() {
        MockProver::run(1, &BracketCircuit::<Fq>::default(), vec![]).unwrap();
    }
}
