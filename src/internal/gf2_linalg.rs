//! GF(2) linear solver for gflow algorithm.

use std::{
    collections::BTreeMap,
    fmt::{self, Debug, Formatter},
    ops::DerefMut,
};

use fixedbitset::FixedBitSet;
use itertools::Itertools;

/// Solver for GF(2) linear equations.
#[derive(PartialEq, Eq)]
pub struct GF2Solver<W: DerefMut<Target = [FixedBitSet]>> {
    /// Number of rows in the coefficient matrix.
    rows: usize,
    /// Number of columns in the coefficient matrix.
    cols: usize,
    /// Number of independent equations solved at once.
    neqs: usize,
    /// Rank of the coefficient matrix. Available after elimination.
    rank: Option<usize>,
    /// Permutation of columns.
    perm: Vec<usize>,
    /// Working storage for the Gauss-Jordan elimination.
    /// Compatible with both owned and borrowed storage.
    work: W,
}

impl<W: DerefMut<Target = [FixedBitSet]>> GF2Solver<W> {
    /// Checks the arguments of `attach`.
    fn attach_check(work: &[FixedBitSet], neqs: usize) {
        assert!(neqs > 0, "neqs is zero");
        let rows = work.len();
        assert!(rows > 0, "work is empty");
        let Ok(width) = work.iter().map(FixedBitSet::len).all_equal_value() else {
            panic!("work is jagged");
        };
        assert!(width > 0, "zero-length columns");
        assert!(width > neqs, "neqs too large");
    }

    /// Attaches to the existing working storage.
    ///
    /// This method is designed for reusing the working storage.
    ///
    /// # Arguments
    ///
    /// - `work`: Working storage for the Gauss-Jordan elimination.
    /// - `neqs`: Number of equations.
    ///
    /// # Panics
    ///
    /// - If `neqs` is zero.
    /// - If `work` is empty (no rows).
    /// - If `work` is jagged, i.e., the number of columns is not uniform.
    /// - If `work[...]` is empty (no columns).
    /// - If `neqs` is so large that there is no room for the coefficient matrix.
    pub fn attach(work: W, neqs: usize) -> Self {
        Self::attach_check(&work, neqs);
        let rows = work.len();
        let width = work[0].len();
        let cols = width - neqs;
        Self {
            rows,
            cols,
            neqs,
            rank: None,
            perm: (0..cols).collect(),
            work,
        }
    }

    /// Moves `(r, c)` to `(i, i)` and updates the permutation.
    fn move_pivot_impl(&mut self, i: usize, r: usize, c: usize) {
        self.work.swap(i, r);
        if i == c {
            return;
        }
        for row in &mut self.work[..self.rows] {
            let bi = row[i];
            let bc = row[c];
            row.set(i, bc);
            row.set(c, bi);
        }
        self.perm.swap(i, c);
    }

    /// Finds the first `1` and move it to `(i, i)`.
    fn move_pivot(&mut self, i: usize) -> bool {
        for c in i..self.cols {
            for (offset, row) in self.work[i..self.rows].iter().enumerate() {
                if row[c] {
                    let r = offset + i;
                    self.move_pivot_impl(i, r, c);
                    return true;
                }
            }
        }
        false
    }

    /// Eliminates the lower triangular part of `work`.
    ///
    /// May panic if the rank is already known.
    fn eliminate_lower(&mut self) {
        debug_assert!(self.rank.is_none());
        let rmax = self.rows.min(self.cols);
        for i in 0..rmax {
            // No remaining `1`
            if !self.move_pivot(i) {
                debug_assert!(!self.work[i][i]);
                self.rank = Some(i);
                return;
            }
            debug_assert!(self.work[i][i]);
            for r in i + 1..self.rows {
                if !self.work[r][i] {
                    continue;
                }
                let [src, dst] = self.work.get_disjoint_mut([i, r]).expect("i < r");
                // MEMO: Rooms for optimization
                //  Redundant operations on the area already cleared
                debug_assert_eq!(src.count_ones(..i), 0);
                dst.symmetric_difference_with(src);
            }
        }
        self.rank = Some(rmax);
    }

    /// Validates the result after the lower elimination.
    fn validate_afterlower(&self) -> bool {
        let rank = self.rank.expect("rank already known here");
        for c in 0..rank {
            for r in c..rank {
                let expected = r == c;
                if self.work[r][c] != expected {
                    return false;
                }
            }
        }
        self.work[rank..self.rows]
            .iter()
            .all(|row| row.count_ones(..self.cols) == 0)
    }

    /// Eliminates the upper triangular part of `work`.
    ///
    /// This method should be called after `eliminate_lower`.
    /// May panic if the rank is not yet known.
    fn eliminate_upper(&mut self) {
        debug_assert!(self.rank.is_some());
        let rank = self.rank.expect("rank already known here");
        for i in (0..rank).rev() {
            debug_assert!(self.work[i][i]);
            for r in 0..i {
                if !self.work[r][i] {
                    continue;
                }
                let [src, dst] = self.work.get_disjoint_mut([i, r]).expect("r < i");
                debug_assert_eq!(src.count_ones(..i), 0);
                dst.symmetric_difference_with(src);
            }
        }
    }

    /// Validates the result after the upper elimination.
    ///
    /// Fails if `eliminate_lower` is not called yet.
    fn validate_afterupper(&self) -> bool {
        let rank = self.rank.expect("rank already known here");
        for c in 0..rank {
            for r in 0..rank {
                let expected = r == c;
                if self.work[r][c] != expected {
                    return false;
                }
            }
        }
        self.work[rank..self.rows]
            .iter()
            .all(|row| row.count_ones(..self.cols) == 0)
    }

    /// Eliminates the lower and upper triangular parts of `work`.
    ///
    /// Guaranteed to be no-op if already eliminated.
    fn eliminate(&mut self) {
        // Already eliminated
        if self.rank.is_some() {
            return;
        }
        self.eliminate_lower();
        debug_assert!(self.validate_afterlower());
        self.eliminate_upper();
        debug_assert!(self.validate_afterupper());
    }

    /// Solves the equation indexed by `ieq` and writes the result to `out`.
    ///
    /// Gaussian elimination is performed only if not done yet.
    ///
    /// # Arguments
    ///
    /// - `out`: Output bitset. Needs to have consistent size.
    /// - `ieq`: Index of the equation to solve.
    ///
    /// # Returns
    ///
    /// `true` if the equation is solvable, `false` otherwise.
    ///
    /// # Panics
    ///
    /// - If `out.len() != self.cols`.
    /// - If `ieq` is out of range.
    pub fn solve_in_place(&mut self, out: &mut FixedBitSet, ieq: usize) -> bool {
        // Eliminate if not done yet
        assert!(
            out.len() == self.cols,
            "output size mismatch: {:} != {:}",
            out.len(),
            self.cols
        );
        self.eliminate();
        assert!(
            ieq < self.neqs,
            "equation index out of range: {:} >= {:}",
            ieq,
            self.neqs
        );
        let rank = self.rank.expect("rank already known here");
        let c = self.cols + ieq;
        // Overdetermined
        if rank < self.rows {
            // = 1 in the zeroed area
            if self.work[rank..self.rows].iter().any(|row| row[c]) {
                return false;
            }
        }
        // One of the possible solutions (eagerly use `0`)
        out.clear();
        for (i, row) in self.work.iter().enumerate() {
            if row[c] {
                out.insert(self.perm[i]);
            }
        }
        true
    }
}

impl<W: DerefMut<Target = [FixedBitSet]>> Debug for GF2Solver<W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut ret = f.debug_struct("GF2Solver");
        ret.field("rows", &self.rows)
            .field("cols", &self.cols)
            .field("neqs", &self.neqs)
            .field("rank", &self.rank)
            .field("perm", &self.perm);
        let mut work = BTreeMap::new();
        for (r, row) in self.work.iter().enumerate() {
            let mut s = String::with_capacity(self.cols);
            for c in 0..self.cols {
                s.push(if row[c] { '1' } else { '0' });
            }
            work.insert(r, s);
        }
        ret.field("co", &work);
        let mut work = BTreeMap::new();
        for (r, row) in self.work.iter().enumerate() {
            let mut s = String::with_capacity(self.neqs);
            for ieq in 0..self.neqs {
                let c = self.cols + ieq;
                s.push(if row[c] { '1' } else { '0' });
            }
            work.insert(r, s);
        }
        ret.field("rhs", &work);
        ret.finish()
    }
}

#[cfg(test)]
mod tests {
    use rand::{self, Rng};
    use rstest::rstest;
    use rstest_reuse::{apply, template};

    use super::*;

    #[test]
    fn test_attach() {
        let mut work = vec![
            // 1000111
            FixedBitSet::with_capacity_and_blocks(7, vec![0b111_0001]),
            // 0100011
            FixedBitSet::with_capacity_and_blocks(7, vec![0b110_0010]),
            // 0010001
            FixedBitSet::with_capacity_and_blocks(7, vec![0b100_0100]),
        ];
        let sol = GF2Solver::attach(work.as_mut_slice(), 3);
        assert_eq!(sol.rows, 3);
        assert_eq!(sol.cols, 4);
        assert_eq!(sol.neqs, 3);
        assert_eq!(sol.rank, None);
        assert_eq!(sol.perm, &[0, 1, 2, 3]);
        assert_eq!(format!("{:}", sol.work[0]), "1000111");
        assert_eq!(format!("{:}", sol.work[1]), "0100011");
        assert_eq!(format!("{:}", sol.work[2]), "0010001");
        // Call Debug
        let ex = format!("{sol:?}");
        assert!(!ex.is_empty());
    }

    /// Helper function to create a solver storage from the coefficient matrix and the right-hand side.
    fn new_from(co: &[FixedBitSet], rhs: &[FixedBitSet]) -> Vec<FixedBitSet> {
        let rows = co.len();
        assert!(rows > 0);
        let neqs = rhs.len();
        assert!(neqs > 0);
        assert_eq!(rhs.iter().map(FixedBitSet::len).all_equal_value(), Ok(rows));
        let Ok(cols) = co.iter().map(FixedBitSet::len).all_equal_value() else {
            panic!("co is jagged");
        };
        assert!(cols > 0);
        let mut work = vec![FixedBitSet::with_capacity(cols + neqs); rows];
        for (r, row) in co.iter().enumerate() {
            for c in row.ones() {
                work[r].insert(c);
            }
        }
        for (ieq, rhsc) in rhs.iter().enumerate() {
            let c = cols + ieq;
            for r in rhsc.ones() {
                work[r].insert(c);
            }
        }
        work
    }

    fn compute_lhs(co: &[FixedBitSet], x: &FixedBitSet) -> FixedBitSet {
        let mut lhs = FixedBitSet::with_capacity(co.len());
        for (r, row) in co.iter().enumerate() {
            let mut sum = false;
            for c in row.ones() {
                sum ^= x[c];
            }
            lhs.set(r, sum);
        }
        lhs
    }

    fn rand_co(rows: usize, cols: usize, p: f64) -> Vec<FixedBitSet> {
        assert!((0.0..=1.0).contains(&p));
        let mut rng = rand::rng();
        let mut co = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = FixedBitSet::with_capacity(cols);
            for c in 0..cols {
                if rng.random::<f64>() < p {
                    row.insert(c);
                }
            }
            co.push(row);
        }
        co
    }

    fn rand_rhs(rows: usize, p: f64) -> FixedBitSet {
        assert!((0.0..=1.0).contains(&p));
        let mut rng = rand::rng();
        let mut rhs = FixedBitSet::with_capacity(rows);
        for r in 0..rows {
            if rng.random::<f64>() < p {
                rhs.insert(r);
            }
        }
        rhs
    }

    #[test]
    #[should_panic = "neqs is zero"]
    fn test_attach_noeq() {
        GF2Solver::attach([].as_mut_slice(), 0);
    }

    #[test]
    #[should_panic = "work is empty"]
    fn test_attach_empty_rows() {
        GF2Solver::attach([].as_mut_slice(), 1);
    }

    #[test]
    #[should_panic = "zero-length columns"]
    fn test_attach_empty_cols() {
        let mut work = vec![FixedBitSet::with_capacity(0); 3];
        GF2Solver::attach(work.as_mut_slice(), 1);
    }

    #[test]
    #[should_panic = "work is jagged"]
    fn test_attach_empty_jagged() {
        let mut work = vec![FixedBitSet::with_capacity(3), FixedBitSet::with_capacity(4)];
        GF2Solver::attach(work.as_mut_slice(), 1);
    }

    #[test]
    #[should_panic = "neqs too large"]
    fn test_attach_neqs_large() {
        let mut work = vec![FixedBitSet::with_capacity(3); 3];
        GF2Solver::attach(work.as_mut_slice(), 4);
    }

    #[test]
    #[should_panic = "output size mismatch:"]
    fn test_solve_invalid_size() {
        let mut work = vec![
            // 1000111
            FixedBitSet::with_capacity_and_blocks(7, vec![0b111_0001]),
            // 0100011
            FixedBitSet::with_capacity_and_blocks(7, vec![0b110_0010]),
            // 0010001
            FixedBitSet::with_capacity_and_blocks(7, vec![0b100_0100]),
        ];
        let mut sol = GF2Solver::attach(work.as_mut_slice(), 3);
        let mut out = FixedBitSet::with_capacity(5);
        sol.solve_in_place(&mut out, 0);
    }

    #[test]
    #[should_panic = "equation index out of range:"]
    fn test_solve_invalid_index() {
        let mut work = vec![
            // 1000111
            FixedBitSet::with_capacity_and_blocks(7, vec![0b111_0001]),
            // 0100011
            FixedBitSet::with_capacity_and_blocks(7, vec![0b110_0010]),
            // 0010001
            FixedBitSet::with_capacity_and_blocks(7, vec![0b100_0100]),
        ];
        let mut sol = GF2Solver::attach(work.as_mut_slice(), 3);
        let mut out = FixedBitSet::with_capacity(4);
        sol.solve_in_place(&mut out, 9);
    }

    #[test]
    fn test_idempotent() {
        let mut work = vec![
            // 101
            FixedBitSet::with_capacity_and_blocks(3, vec![0b101]),
            // 110
            FixedBitSet::with_capacity_and_blocks(3, vec![0b110]),
        ];
        let mut sol = GF2Solver::attach(work.as_mut_slice(), 1);
        assert_eq!(sol.rank, None);
        let dummy_rank = 99;
        // Set invalid value
        sol.rank = Some(dummy_rank);
        // No-op
        sol.eliminate();
        assert_eq!(sol.rank, Some(dummy_rank));
    }

    #[test]
    fn test_validate_afterlower_ng() {
        let mut work = vec![
            // 001
            FixedBitSet::with_capacity_and_blocks(3, vec![0b001]),
            // 010
            FixedBitSet::with_capacity_and_blocks(3, vec![0b011]),
        ];
        let mut sol = GF2Solver::attach(work.as_mut_slice(), 1);
        // Exact value
        sol.rank = Some(2);
        assert!(!sol.validate_afterlower());
        assert!(!sol.validate_afterupper());
    }

    #[test]
    fn test_validate_afterupper_ng() {
        let mut work = vec![
            // 011
            FixedBitSet::with_capacity_and_blocks(3, vec![0b011]),
            // 010
            FixedBitSet::with_capacity_and_blocks(3, vec![0b010]),
        ];
        let mut sol = GF2Solver::attach(work.as_mut_slice(), 1);
        // Exact value
        sol.rank = Some(2);
        assert!(sol.validate_afterlower());
        assert!(!sol.validate_afterupper());
    }

    #[test]
    fn test_move_pivot_impl() {
        for r in 0..3 {
            for c in 0..3 {
                let mut work = vec![FixedBitSet::with_capacity_and_blocks(4, vec![0b0000]); 3];
                work[r].insert(c);
                let mut sol = GF2Solver::attach(work.as_mut_slice(), 1);
                sol.move_pivot_impl(0, r, c);
                assert!(sol.work[0][0]);
            }
        }
    }

    #[cfg(not(miri))]
    const REP: usize = 1000;
    #[cfg(miri)]
    const REP: usize = 1;

    #[cfg(not(miri))]
    #[template]
    #[rstest]
    fn template_tests(
        #[values(1, 2, 7, 12, 23)] rows: usize,
        #[values(1, 2, 7, 12, 23)] cols: usize,
        #[values(1, 2, 7, 12)] neqs: usize,
    ) {
    }

    #[cfg(miri)]
    #[template]
    #[rstest]
    fn template_tests(
        #[values(1, 2, 7)] rows: usize,
        #[values(1, 2, 7)] cols: usize,
        #[values(1, 2)] neqs: usize,
    ) {
    }

    #[test]
    fn test_solve_simple() {
        let mut work = vec![
            // 1001
            FixedBitSet::with_capacity_and_blocks(4, vec![0b1001]),
            // 0101
            FixedBitSet::with_capacity_and_blocks(4, vec![0b1010]),
            // 0000
            FixedBitSet::with_capacity_and_blocks(4, vec![0b0000]),
        ];
        let mut sol = GF2Solver::attach(work.as_mut_slice(), 1);
        let mut x = FixedBitSet::with_capacity(3);
        assert_eq!(sol.rank, None);
        assert!(sol.solve_in_place(&mut x, 0));
        assert_eq!(sol.rank, Some(2));
        let x_orig = x.clone();
        assert!(sol.solve_in_place(&mut x, 0));
        assert_eq!(x, x_orig);
        assert!(x[0]);
        assert!(x[1]);
        assert!(!x[2]);
    }

    #[apply(template_tests)]
    fn test_eliminate_lower_random(rows: usize, cols: usize, neqs: usize) {
        let mut rng = rand::rng();
        for _ in 0..REP {
            // Random p
            let p1 = rng.random::<f64>();
            let p2 = rng.random::<f64>();
            let co = rand_co(rows, cols, p1);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, p2));
            let mut work = new_from(&co, &rhs);
            let mut sol = GF2Solver::attach(work.as_mut_slice(), neqs);
            sol.eliminate_lower();
            assert!(sol.validate_afterlower());
        }
    }

    #[apply(template_tests)]
    fn test_eliminate_lower_special(rows: usize, cols: usize, neqs: usize) {
        for p in [0.0, 1.0] {
            // Special p
            let co = rand_co(rows, cols, p);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, 0.5));
            let mut work = new_from(&co, &rhs);
            let mut sol = GF2Solver::attach(work.as_mut_slice(), neqs);
            sol.eliminate_lower();
            assert!(sol.validate_afterlower());
        }
    }

    #[apply(template_tests)]
    fn test_eliminate_upper_random(rows: usize, cols: usize, neqs: usize) {
        let mut rng = rand::rng();
        for _ in 0..REP {
            // Random p
            let p1 = rng.random::<f64>();
            let p2 = rng.random::<f64>();
            let co = rand_co(rows, cols, p1);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, p2));
            let mut work = new_from(&co, &rhs);
            let mut sol = GF2Solver::attach(work.as_mut_slice(), neqs);
            sol.eliminate();
            assert!(sol.validate_afterupper());
        }
    }

    #[apply(template_tests)]
    fn test_eliminate_upper_special(rows: usize, cols: usize, neqs: usize) {
        for p in [0.0, 1.0] {
            // Special p
            let co = rand_co(rows, cols, p);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, 0.5));
            let mut work = new_from(&co, &rhs);
            let mut sol = GF2Solver::attach(work.as_mut_slice(), neqs);
            sol.eliminate();
            assert!(sol.validate_afterupper());
        }
    }

    #[apply(template_tests)]
    fn test_solve_random(rows: usize, cols: usize, neqs: usize) {
        let mut rng = rand::rng();
        for _ in 0..REP {
            // Random p
            let p1 = rng.random::<f64>();
            let p2 = rng.random::<f64>();
            let co = rand_co(rows, cols, p1);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, p2));
            let mut work = new_from(&co, &rhs);
            let mut sol = GF2Solver::attach(work.as_mut_slice(), neqs);
            sol.eliminate();
            for (ieq, rhsi) in rhs.iter().enumerate() {
                let mut x = FixedBitSet::with_capacity(cols);
                if !sol.solve_in_place(&mut x, ieq) {
                    assert!(sol.rank.unwrap() < sol.rows);
                    continue;
                }
                for row in &sol.work[sol.rank.unwrap()..sol.rows] {
                    assert!(row.count_ones(..sol.cols) == 0);
                    assert!(!row[cols + ieq]);
                }
                let b = compute_lhs(&co, &x);
                assert_eq!(&b, rhsi);
                let rank = sol.rank.unwrap();
                for i_ in rank..sol.cols {
                    let i = sol.perm[i_];
                    assert!(!x[i]);
                }
            }
        }
    }

    #[apply(template_tests)]
    fn test_solve_special(rows: usize, cols: usize, neqs: usize) {
        for (p1, p2) in [(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)] {
            // Special p
            let co = rand_co(rows, cols, p1);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, p2));
            let mut work = new_from(&co, &rhs);
            let mut sol = GF2Solver::attach(work.as_mut_slice(), neqs);
            sol.eliminate();
            for (ieq, rhsi) in rhs.iter().enumerate() {
                let mut x = FixedBitSet::with_capacity(cols);
                if !sol.solve_in_place(&mut x, ieq) {
                    assert!(sol.rank.unwrap() < sol.rows);
                    continue;
                }
                for row in &sol.work[sol.rank.unwrap()..sol.rows] {
                    assert!(row.count_ones(..sol.cols) == 0);
                    assert!(!row[cols + ieq]);
                }
                let b = compute_lhs(&co, &x);
                assert_eq!(&b, rhsi);
                let rank = sol.rank.unwrap();
                for i_ in rank..sol.cols {
                    let i = sol.perm[i_];
                    assert!(!x[i]);
                }
            }
        }
    }
}
