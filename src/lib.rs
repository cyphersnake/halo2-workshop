use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::group::ff::PrimeField,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Expression, Selector, TableColumn},
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
    s_input: Selector,
    // For input
    input: Column<Advice>,
    // For allowed ASCII codes
    allowed: TableColumn,
    accum: Column<Advice>,
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
            s_input: meta.selector(),
            input: meta.advice_column(),
            allowed: meta.lookup_table_column(),
            accum: meta.advice_column(),
        };

        // f(x) = 81 - 2*input
        meta.create_gate("", |meta| {
            let _81 = Expression::Constant(F::from(81));
            let _2 = Expression::Constant(F::from(2));

            let s_input = meta.query_selector(config.s_input);
            let input = meta.query_advice(config.input, Rotation::cur());
            let prev = meta.query_advice(config.accum, Rotation::cur());
            let result = meta.query_advice(config.accum, Rotation::next());

            vec![s_input * (prev + (_81 - _2 * input) - result)]
        });

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

        layouter.assign_region(
            || "",
            |mut region| {
                self.input
                    .iter()
                    .map(|sym| Value::known(F::from(*sym as u64)))
                    .enumerate()
                    .try_for_each(|(offset, sym)| {
                        region
                            .assign_advice(|| "input", config.input, offset, || sym)
                            .map(|_| ())
                    })
            },
        )?;

        Ok(())
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

    //#[test]
    //fn unvalid_order() {
    //    MockProver::run(K, &BracketCircuit::<2, Fq>::new([')', '(']), vec![])
    //        .unwrap()
    //        .verify()
    //        .unwrap_err();
    //}
}
