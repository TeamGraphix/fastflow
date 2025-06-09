//! GF(2) linear solver interface for Python.

use fixedbitset::FixedBitSet;
use numpy::ndarray::{Array1, ArrayView2};

use crate::internal::gf2_linalg::GF2Solver;

pub struct Solver {
    sol: GF2Solver<Vec<FixedBitSet>>,
}

impl Solver {
    /// Create a `Solver` from equation.
    ///
    /// # Arguments
    ///
    /// - `a`: Coefficient matrix.
    /// - `b`: Right-hand side matrix.
    ///
    /// # Panics
    ///
    /// Panics if the numbers of variables in `a` and `b` do not match.
    #[allow(clippy::must_use_candidate)]
    pub fn from_eq(a: &ArrayView2<bool>, b: &ArrayView2<bool>) -> Self {
        let (rows, cols) = a.dim();
        let (rows_, neqs) = b.dim();
        let width = cols + neqs;
        assert_eq!(rows, rows_, "Inconsistent number of rows.");
        let mut work = vec![FixedBitSet::with_capacity(width); rows];
        for r in 0..rows {
            for c in 0..cols {
                work[r].set(c, a[[r, c]]);
            }
            for ieq in 0..neqs {
                work[r].set(cols + ieq, b[[r, ieq]]);
            }
        }
        Self {
            sol: GF2Solver::attach(work, neqs),
        }
    }

    /// Solve the equation indexed by `ieq`.
    pub fn solve(&mut self, ieq: usize) -> Option<Array1<bool>> {
        self.sol
            .solve(ieq)
            .map(|x| Array1::from_iter((0..x.len()).map(|i| x[i])))
    }
}

#[cfg(test)]
mod tests {
    use numpy::ndarray::array;

    use super::*;

    #[test]
    fn test_solver() {
        let a = array![[true, true], [false, true]];
        let b = array![[false], [true]];
        let mut solver = Solver::from_eq(&a.view(), &b.view());
        assert_eq!(solver.solve(0), Some(array![true, true]));
    }
}
