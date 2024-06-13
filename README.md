
> [!IMPORTANT]
> This repository is under active development as part of a workshop on zero-knowledge proofs.

## Arithmetic & Implementation
### **Arithmetization**

Usually, a programmer works with imperative code. However, to write a circuit , you first need to convert it into a specific mathematical
description. What kind depends on the constraint system we work with. In the context
of halo2, we use [PLONKish](https://zcash.github.io/halo2/concepts/arithmetization.html).

Within this framework, we have a matrix over a field (each matrix element is a finite field element over a prime number.

#### 📍Columns of Three Types
- Fixed - immutable constants
- Instance - inputs to the circuit (provided externally)
- Advice - values depending on inputs

#### 📍Column Relationships (Gates)
Example: "5th column plus 4th column minus 9th column should equal 0"

#### 📍Allowed Values for Relationships (Lookup)
Example: "5th column plus 4th column minus 9th column should be something from 1st
column"

#### 📍Equalities Between Cells (Copy-Constraint)
Simply put, this is the only type of relationship not between columns but between
cells.

All tools, except the last, are applied in the first stage of circuit configuration. Naturally, the fewer columns, gates, and lookup arguments, the better, though their number varies from project to project. Our [Sirius project](https://github.com/snarkify/sirius/blob/main/src/main_gate.rs) has one gate, while [keccak from Scroll](https://github.com/scroll-tech/zkevm-circuits/tree/0a2da7f1b8f716375d35135a3bb8f436f40491f9/keccak256) has more than a thousand. Some on-circuit virtual machines are built entirely using lookup tables without a single gate. You can scale horizontally by adding columns or vertically by minimizing the number of relationships over columns, affecting various parameters of the final solution. Therefore, arithmetization is a cornerstone topic in circuit
development.

Your ultimate task is to express the target algorithm as a set of relationships using the tools described above. In simpler terms, each row of the matrix represents a piece of computation, and the computations are defined by the columns.

Next time, I will formulate the task and write how to arithmetize it as an example. The code follows shortly after!

### **Formulating Arithmetic Task**

Let's articulate: We have an input string with characters `(` and `)`, encoded in ASCII codes: 40, 41. We need to verify that the string is balanced, i.e., that all brackets are matched.

Task: Write a circuit that proves we have such a string without revealing it.

Explanation: This may sound pointless, but now change the task to verifying a password's cryptographic strength, and the practical application becomes evident.

### **Algorithm**

```rust
fn is_valid_brackets(s: &str) -> bool {
    s.chars()
        .try_fold(0u32, |acc, c| match c {
            '(' => Some(acc + 1),
            ')' => acc.checked_sub(1),
            other => panic!("not allowed symbol: {other}"),
        })
        .eq(&Some(0))
}

fn main() {
    assert!(is_valid_brackets("()"));
    assert!(!is_valid_brackets(")("));
}
```

### **Solution in Mathematics**

1. Validate that the input consists only of allowable ASCII codes.
2. A function `f(x)` that maps 40 & 41 to 1 & -1 respectively.
3. Sum all values and check if `Sum(f(x)) == 0`.
4. Implement checked subtraction on-circuit to invalidate the )( case.

Before reading further, you might want to think of solutions for this part on your
own.

### **Implementation**

1. Lookup Table: Simply copy the input signal and verify it belongs to a table filled with just two constant values.
  
2. unsure how to solve this with a polynomial? Ask GPT! For instance: "Form a polynomial F(x) that maps 40 to 1 and 41 to -1." The result: `f(x) = 81 - 2x`

3. Here’s the tricky part: We can use many columns and perform all computation in one row, or use fewer columns and spread the computation across rows. Optimal tuning of column and row count holds many nuances, but generally, it's better to scale vertically than horizontally. For now, accept this as doctrine.

Example Column Configuration:

| a1      | a2                    |
| ------- | --------------------- |
| x_i     | prev_val              |
| x_{i+1} | prev_val + 81 - 2*x_i |

This accumulates the sum of all input values in the accumulator column. At the end, compare to zero.

4. Here lies the crux! In finite fields, we have no negative numbers (i.e., -1 is represented as p-1, where p is field characteristic). Additionally, comparison operations like "greater/less" are non-trivial.

Solutions? Concepts like is_zero field checking come into play, as demonstrated [here](https://github.com/icemelon/halo2-examples/blob/master/src/is_zero.rs).

### **Halo2 Implementation**

We now have all the math needed for the circuit. Create the project:

```sh
cargo init --lib halo2-workshop; cd halo2-workshop; cargo add halo2_proofs
```

In `lib.rs`, create a simple project:

```rust
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
```
[step-1-code](https://github.com/cyphersnake/halo2-workshop/blob/step-1/src/lib.rs)

---
This concludes the first part of our zero-knowledge proof workshop. Stay tuned for
more updates!
