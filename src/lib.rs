#![allow(clippy::just_underscores_and_digits)]
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
/// - gate1: (acc + 1) * (1 - (acc + 1) * inv(acc + 1)) = 0
/// - gate2: 1 - (acc + 1) * inv(acc + 1) = 0
///
/// Alternative approaches include using a lookup table for possible accumulator values but this is less optimal.
use std::{iter, marker::PhantomData};

use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    pasta::group::ff::PrimeField,
    plonk::{Advice, Circuit, Column, ConstraintSystem, Expression, Selector, TableColumn},
    poly::Rotation,
};

const MAX_LEN: usize = 10;

pub struct BracketCircuit<F: PrimeField, const MAX_LEN: usize> {
    input: [char; MAX_LEN],
    _p: PhantomData<F>,
}

impl<F: PrimeField, const MAX_LEN: usize> BracketCircuit<F, MAX_LEN> {
    pub fn try_new(input: impl AsRef<str>) -> Option<Self> {
        Some(Self {
            input: input.as_ref().chars().collect::<Vec<_>>().try_into().ok()?,
            _p: PhantomData,
        })
    }
}

// Stores the configuration of the table (columns) that the circuit needs
#[derive(Clone)]
pub struct Config {
    s_accumulation: Selector,
    s_is_zero: Selector,
    s_not_min_one: Selector,

    input: Column<Advice>,
    previous_result: Column<Advice>,
    result: Column<Advice>,

    invert_result: Column<Advice>,
    table: TableColumn,
}

impl<F: PrimeField, const L: usize> Circuit<F> for BracketCircuit<F, L> {
    type Config = Config;

    // Not important at this stage
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        todo!("Not needed at this stage.")
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let config = Config {
            s_accumulation: meta.selector(),
            s_is_zero: meta.selector(),
            s_not_min_one: meta.selector(),

            result: meta.advice_column(),
            input: meta.advice_column(),
            previous_result: meta.advice_column(),
            invert_result: meta.advice_column(),
            table: meta.lookup_table_column(),
        };
        meta.enable_equality(config.result);
        meta.enable_equality(config.previous_result);

        meta.create_gate("accumulation", |meta| {
            let _81 = Expression::Constant(F::from(81));
            let _3281 = Expression::Constant(F::from(3281));
            let _inv_1640 = Expression::Constant(F::from(1640).invert().unwrap());

            let s_accumulation = meta.query_selector(config.s_accumulation);
            let s_is_zero = meta.query_selector(config.s_is_zero);

            let input = meta.query_advice(config.input, Rotation::cur());
            let result = meta.query_advice(config.result, Rotation::cur());
            let previous_result = meta.query_advice(config.previous_result, Rotation::cur());

            let function = -(input.clone() * (_81 * input.clone() - _3281) * _inv_1640);

            vec![
                s_accumulation * (previous_result.clone() + function - result.clone()),
                s_is_zero * result,
            ]
        });

        meta.create_gate("neg check for accum", |meta| {
            let _1 = Expression::Constant(F::from(1));

            let s = meta.query_selector(config.s_not_min_one);

            let r_plus_1 = meta.query_advice(config.result, Rotation::cur()) + _1.clone();
            let inv_r_plus_1 = meta.query_advice(config.invert_result, Rotation::cur());

            let gate1 = r_plus_1.clone() * (_1.clone() - r_plus_1.clone() * inv_r_plus_1.clone());
            let gate2 = _1 - r_plus_1 * inv_r_plus_1;

            vec![s.clone() * gate1, s * gate2]
        });

        meta.lookup(|meta| {
            vec![(
                meta.query_advice(config.input, Rotation::cur()),
                config.table,
            )]
        });

        config
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        let _81 = Value::known(F::from(81));
        let _2 = Value::known(F::from(2));
        let _3281 = Value::known(F::from(3281));
        let _inv_1640 = Value::known(F::from(1640).invert().unwrap());

        layouter.assign_region(
            || "input",
            |mut region| {
                self.input
                    .iter()
                    .map(|sym| Value::known(F::from(*sym as u64)))
                    .enumerate()
                    .try_fold(None, |prev: Option<AssignedCell<F, F>>, (offset, value)| {
                        config.s_accumulation.enable(&mut region, offset)?;

                        region.assign_advice(|| "input", config.input, offset, || value)?;

                        let mut acc_value = -(value * ((_81 * value) - _3281) * _inv_1640);

                        if let Some(previous_row_cell) = prev {
                            let assigned_prev_current_row = region.assign_advice(
                                || "previous result",
                                config.previous_result,
                                offset,
                                || previous_row_cell.value().copied(),
                            )?;
                            region.constrain_equal(
                                assigned_prev_current_row.cell(),
                                previous_row_cell.cell(),
                            )?;

                            acc_value = previous_row_cell.value().copied() + acc_value;
                        } else {
                            region.assign_advice(
                                || "accumulator",
                                config.previous_result,
                                offset,
                                || Value::known(F::ZERO),
                            )?;
                        }

                        let accum = region.assign_advice(
                            || "accumulator",
                            config.result,
                            offset,
                            || acc_value,
                        )?;

                        config.s_not_min_one.enable(&mut region, offset)?;
                        region.assign_advice(
                            || "inverted accumulator",
                            config.invert_result,
                            offset,
                            || acc_value.map(|v| (v + F::ONE).invert().unwrap_or_else(|| F::ZERO)),
                        )?;

                        Result::<_, halo2_proofs::plonk::Error>::Ok(Some(accum))
                    })?;

                config.s_is_zero.enable(&mut region, L - 1)?;

                Ok(())
            },
        )?;

        layouter.assign_table(
            || "input_check",
            |mut table| {
                table.assign_cell(|| "empty", config.table, 0, || Value::known(F::from(0)))?;
                table.assign_cell(|| "(", config.table, 1, || Value::known(F::from(40)))?;

                table.assign_cell(|| ")", config.table, 2, || Value::known(F::from(41)))
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    use super::*;

    #[test]
    fn valid() {
        MockProver::run(
            10,
            &BracketCircuit::<Fp, 10>::try_new("(()(())())").unwrap(),
            vec![],
        )
        .unwrap()
        .verify()
        .unwrap();
    }

    #[test]
    fn simple_valid() {
        MockProver::run(10, &BracketCircuit::<Fp, 2>::try_new("()").unwrap(), vec![])
            .unwrap()
            .verify()
            .unwrap();
    }

    #[test]
    fn unvalid_order() {
        MockProver::run(10, &BracketCircuit::<Fp, 2>::try_new(")(").unwrap(), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }

    #[test]
    fn unvalid_solo_symbol_open() {
        MockProver::run(10, &BracketCircuit::<Fp, 1>::try_new("(").unwrap(), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }

    #[test]
    fn unvalid_solo_symbol_close() {
        MockProver::run(10, &BracketCircuit::<Fp, 1>::try_new(")").unwrap(), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }

    #[test]
    fn wrong_symbol() {
        MockProver::run(10, &BracketCircuit::<Fp, 1>::try_new("*").unwrap(), vec![])
            .unwrap()
            .verify()
            .unwrap_err();
    }
}
