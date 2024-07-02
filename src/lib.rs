use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::group::ff::PrimeField,
    plonk::{Advice, Circuit, Column, ConstraintSystem, TableColumn},
    poly::Rotation,
};

// Sets the circuit, and also stores the private input
pub struct BracketCircuit<const L: usize, F: PrimeField> {
    input: [char; L],
    _p: PhantomData<F>,
}

impl<const L: usize, F: PrimeField> BracketCircuit<L, F> {
    pub fn new(input: [char; L]) -> Self {
        Self {
            input,
            _p: PhantomData,
        }
    }
}

// Stores the configuration of the table (columns) that the circuit needs
#[derive(Clone)]
pub struct Config {
    // For input
    input: Column<Advice>,
    // For allowed ASCII codes
    allowed: TableColumn,
}

impl<const L: usize, F: PrimeField> Circuit<F> for BracketCircuit<L, F> {
    type Config = Config;

    // Not important at this stage
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        todo!("Not needed at this stage.")
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let config = Config {
            input: meta.advice_column(),
            allowed: meta.lookup_table_column(),
        };

        meta.lookup(|table| {
            let input = table.query_advice(config.input, Rotation::cur());

            vec![(input, config.allowed)]
        });

        config
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        layouter.assign_table(
            || "allowed",
            |mut table| {
                table.assign_cell(|| "empty", config.allowed, 0, || Value::known(F::ZERO))?;
                table.assign_cell(
                    || "(",
                    config.allowed,
                    1,
                    || Value::known(F::from('(' as u64)),
                )?;
                table.assign_cell(
                    || ")",
                    config.allowed,
                    2,
                    || Value::known(F::from(')' as u64)),
                )?;

                Ok(())
            },
        )?;

        layouter.assign_region(|| "", |region| {
            self.input.iter().map(|sym: char| Value::known(F::from(sym as u64)).try_for_each(|_| {
                todo!()
            });
            region.assign_advice(|| "", config.input, offset, to)
        })
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{dev::MockProver, pasta::Fq};

    use super::*;

    const K: u32 = 10;

    #[test]
    fn unvalid_sym() {
        MockProver::run(K, &BracketCircuit::<1, Fq>::new(['*']), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }

    #[test]
    fn valid_1() {
        MockProver::run(K, &BracketCircuit::<2, Fq>::new(['(', ')']), vec![])
            .unwrap()
            .verify()
            .unwrap();
    }

    #[test]
    fn unvalid_order() {
        MockProver::run(K, &BracketCircuit::<2, Fq>::new([')', '(']), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }
}
