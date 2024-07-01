/// # Balanced Bracket Verification Circuit
///
/// This Rust code and its zk-SNARK adaptation verify if a string of brackets `('` and `')'` is balanced.
/// The goal is to prove this property without revealing the string itself.
///
/// ## Algorithm
///
/// The core Rust function checks if brackets are balanced:
///
/// ```rust
/// fn is_valid_brackets(s: &str) -> bool {
///     s.chars()
///         .try_fold(0u32, |acc, c| match c {
///             '(' => Some(acc + 1),
///             ')' => acc.checked_sub(1),
///             other => panic!("not allowed symbol: {other}"),
///         })
///         .eq(&Some(0))
/// }
///
/// fn main() {
///     assert!(is_valid_brackets("()"));
///     assert!(!is_valid_brackets(")("));
/// }
/// ```
///
/// ## Mathematical Representation
///
/// 1. Validate the input contains only the allowed ASCII codes (40 for `(` and 41 for `)`).
/// 2. Use a function `f(x)` that maps 40 to +1 and 41 to -1.
///
///    Example: `f(x) = 81 - 2x`
///
/// 3. Sum the values and ensure `Sum(f(x)) == 0`.
/// 4. Ensure the subtraction in the circuit mirrors the `checked_sub` behavior to prevent invalid cases.
///
/// ## zk-SNARK Circuit Implementation
///
/// 1. **Lookup Table**: Ensure the character is in a set of valid ASCII codes.
/// 2. **Polynomic Transformation**: Use functions that map ASCII values to +1 or -1.
/// 3. **Sum Calculation**: Accumulate the sum in columns and verify it equals zero.
/// 4. **Field Characteristics Handling**: Since negative numbers aren't directly representable,
///    use specific polynomial constraints to emulate this behavior.
///
/// ## Constraints
///
/// The circuit design includes:
/// - lookup0: input in [40, 41]
/// - gate0: (prev_acc + (81 - 2 * input)) - acc = 0
/// - gate1: acc * (1 - acc * inv_acc) = 0
/// - gate2: 1 - acc * inv_acc = 0
///
/// Alternative approaches include using a lookup table for possible accumulator values but this is less optimal.
use std::marker::PhantomData;

use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    pasta::group::ff::PrimeField,
    plonk::{Advice, Circuit, Column, ConstraintSystem, TableColumn},
    poly::Rotation,
};

pub struct BracketCircuit<F: PrimeField> {
    input: String,
    _p: PhantomData<F>,
}

impl<F: PrimeField> BracketCircuit<F> {
    pub fn new(input: impl AsRef<str>) -> Self {
        Self {
            input: input.as_ref().to_string(),
            _p: PhantomData,
        }
    }
}

// Stores the configuration of the table (columns) that the circuit needs
#[derive(Clone)]
pub struct Config {
    input: Column<Advice>,
    accum: Column<Advice>,
    inv_accum: Column<Advice>,
    table: TableColumn,
}

impl<F: PrimeField> Circuit<F> for BracketCircuit<F> {
    type Config = Config;

    // Not important at this stage
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        todo!("Not needed at this stage.")
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let config = Config {
            input: meta.advice_column(),
            accum: meta.advice_column(),
            inv_accum: meta.advice_column(),
            table: meta.lookup_table_column(),
        };

        meta.lookup(|meta| {
            let input = meta.query_advice(config.input, Rotation::cur());
            vec![(input, config.table)]
        });

        config
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        let _input = layouter.assign_region(
            || "input",
            |mut region| {
                self.input
                    .chars()
                    .enumerate()
                    .map(|(offset, sym)| {
                        region.assign_advice(
                            || "input",
                            config.input,
                            offset,
                            || Value::known(F::from(sym as u64)),
                        )
                    })
                    .collect::<Result<Box<[_]>, _>>()
            },
        )?;

        layouter.assign_table(
            || "input_check",
            |mut table| {
                table.assign_cell(|| "", config.table, 0, || Value::known(F::from(0)))?;
                table.assign_cell(|| "(", config.table, 1, || Value::known(F::from(40)))?;

                table.assign_cell(|| ")", config.table, 2, || Value::known(F::from(41)))
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{dev::MockProver, pasta::Fq};

    use super::*;

    #[test]
    fn valid() {
        MockProver::run(10, &BracketCircuit::<Fq>::new("()"), vec![])
            .unwrap()
            .verify()
            .unwrap();
    }

    #[test]
    fn unvali_order() {
        MockProver::run(10, &BracketCircuit::<Fq>::new(")("), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }

    #[test]
    fn solo_symbol() {
        MockProver::run(10, &BracketCircuit::<Fq>::new("("), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }

    #[test]
    fn wrong_symbol() {
        MockProver::run(10, &BracketCircuit::<Fq>::new("*"), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }
}
