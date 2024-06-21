use anyhow;
use fixedbitset::FixedBitSet;

type GF2Matrix = Vec<FixedBitSet>;

#[derive(Debug, Clone)]
pub struct GF2Solver {
    rows: usize,
    cols: usize,
    neqs: usize,
    rank: Option<usize>,
    perm: Vec<usize>,
    work: GF2Matrix,
}

impl GF2Solver {
    // Left for convenience
    #[allow(dead_code)]
    pub fn new_from(co: &GF2Matrix, rhs: &[FixedBitSet]) -> anyhow::Result<Self> {
        let rows = co.len();
        if rows == 0 {
            return Err(anyhow::anyhow!("co is empty"));
        }
        let neqs = rhs.len();
        if neqs == 0 {
            return Err(anyhow::anyhow!("rhs is empty"));
        }
        if rhs.iter().any(|rhsi| rhsi.len() != rows) {
            return Err(anyhow::anyhow!("rhs size mismatch"));
        }
        let cols = co[0].len();
        if co.iter().any(|coi| coi.len() != cols) {
            return Err(anyhow::anyhow!("co is jagged"));
        }
        if cols == 0 {
            return Err(anyhow::anyhow!("zero-length columns"));
        }
        let mut work = vec![FixedBitSet::with_capacity(cols + neqs); rows];
        for (r, row) in co.iter().enumerate() {
            for c in row.ones() {
                work[r].insert(c);
            }
        }
        for (mut c, rhsc) in rhs.iter().enumerate() {
            c += cols;
            for r in rhsc.ones() {
                work[r].insert(c);
            }
        }
        Ok(Self {
            rows,
            cols,
            neqs,
            rank: None,
            perm: (0..cols).collect(),
            work,
        })
    }

    pub fn attach(work: GF2Matrix, neqs: usize) -> anyhow::Result<Self> {
        if neqs == 0 {
            return Err(anyhow::anyhow!("neqs is zero"));
        }
        let rows = work.len();
        if rows == 0 {
            return Err(anyhow::anyhow!("work is empty"));
        }
        let width = work[0].len();
        if work.iter().any(|worki| worki.len() != width) {
            return Err(anyhow::anyhow!("work is jagged"));
        }
        if width == 0 {
            return Err(anyhow::anyhow!("zero-length columns"));
        }
        if width <= neqs {
            return Err(anyhow::anyhow!("neqs mismatch"));
        }
        let cols = width - neqs;
        Ok(Self {
            rows,
            cols,
            neqs,
            rank: None,
            perm: (0..cols).collect(),
            work,
        })
    }

    pub fn detach(self) -> GF2Matrix {
        self.work
    }

    fn move_pivot_impl(&mut self, i: usize, r: usize, c: usize) {
        self.work.swap(i, r);
        if i == c {
            return;
        }
        for r in 0..self.rows {
            let bi = self.work[r][i];
            let bc = self.work[r][c];
            self.work[r].set(i, bc);
            self.work[r].set(c, bi);
        }
        self.perm.swap(i, c);
    }

    fn move_pivot(&mut self, i: usize) -> bool {
        for c in i..self.cols {
            for r in i..self.rows {
                if self.work[r][c] {
                    self.move_pivot_impl(i, r, c);
                    return true;
                }
            }
        }
        false
    }

    fn eliminate_lower(&mut self) {
        debug_assert!(self.rank.is_none());
        let rmax = self.rows.min(self.cols);
        for i in 0..rmax {
            // No remaining `1`
            if !self.move_pivot(i) {
                self.rank = Some(i);
                return;
            }
            for r in i + 1..self.rows {
                if !self.work[r][i] {
                    continue;
                }
                // MEMO: i < r
                // Need to borrow twice at a time
                let (ss, s2) = self.work.split_at_mut(r);
                let (_, s1) = ss.split_at(i);
                let src = &s1[0];
                let dst = &mut s2[0];
                // MEMO: Rooms for optimization
                //  Redundant operations on the area already cleared
                *dst ^= src;
            }
        }
        self.rank = Some(rmax);
    }

    #[cfg(test)]
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
        for i in rank..self.rows {
            if self.work[i].count_ones(..self.cols) != 0 {
                return false;
            }
        }
        true
    }

    fn eliminate_upper(&mut self) {
        debug_assert!(self.rank.is_some());
        let rank = self.rank.expect("rank already known here");
        for i in (0..rank).rev() {
            for r in 0..i {
                if !self.work[r][i] {
                    continue;
                }
                // MEMO: r < i
                let (ss, s2) = self.work.split_at_mut(i);
                let (_, s1) = ss.split_at_mut(r);
                let src = &s2[0];
                let dst = &mut s1[0];
                *dst ^= src;
            }
        }
    }

    #[cfg(test)]
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
        for i in rank..self.rows {
            if self.work[i].count_ones(..self.cols) != 0 {
                return false;
            }
        }
        true
    }

    fn eliminate(&mut self) {
        // Already eliminated
        if self.rank.is_some() {
            return;
        }
        self.eliminate_lower();
        self.eliminate_upper();
    }

    pub fn solve_in_place(&mut self, out: &mut FixedBitSet, ieq: usize) -> bool {
        // Eliminate if not done yet
        self.eliminate();
        if ieq >= self.neqs {
            panic!("equation index out of range");
        }
        let rank = self.rank.expect("rank already known here");
        let c = self.cols + ieq;
        // Overdetermined
        if rank < self.rows {
            for r in rank..self.rows {
                // = 1 in the zeroed area
                if self.work[r][c] {
                    return false;
                }
            }
        }
        // One of the possible solutions (eagerly use `0`)
        out.grow(self.cols);
        out.clear();
        for i in 0..rank {
            if self.work[i][c] {
                out.insert(self.perm[i]);
            }
        }
        true
    }

    // Left for easier testing
    #[allow(dead_code)]
    pub fn solve(&mut self, ieq: usize) -> Option<FixedBitSet> {
        let mut out = FixedBitSet::with_capacity(self.cols);
        match self.solve_in_place(&mut out, ieq) {
            false => None,
            true => Some(out),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use rstest::rstest;
    use rstest_reuse::{apply, template};

    #[test]
    fn test_new_from() {
        let co = vec![
            // 1000
            FixedBitSet::with_capacity_and_blocks(4, vec![0b0001]),
            // 0100
            FixedBitSet::with_capacity_and_blocks(4, vec![0b0010]),
            // 0010
            FixedBitSet::with_capacity_and_blocks(4, vec![0b0100]),
        ];
        let rhs = vec![
            // 100
            FixedBitSet::with_capacity_and_blocks(3, vec![0b001]),
            // 110
            FixedBitSet::with_capacity_and_blocks(3, vec![0b011]),
            // 111
            FixedBitSet::with_capacity_and_blocks(3, vec![0b111]),
        ];
        let sol = GF2Solver::new_from(&co, &rhs).unwrap();
        assert_eq!(sol.rows, 3);
        assert_eq!(sol.cols, 4);
        assert_eq!(sol.neqs, 3);
        assert_eq!(sol.rank, None);
        assert_eq!(sol.perm, &[0, 1, 2, 3]);
        assert_eq!(format!("{:}", sol.work[0]), "1000111");
        assert_eq!(format!("{:}", sol.work[1]), "0100011");
        assert_eq!(format!("{:}", sol.work[2]), "0010001");
    }

    #[test]
    fn test_attach() {
        let work = vec![
            // 1000111
            FixedBitSet::with_capacity_and_blocks(7, vec![0b1110001]),
            // 0100011
            FixedBitSet::with_capacity_and_blocks(7, vec![0b1100010]),
            // 0010001
            FixedBitSet::with_capacity_and_blocks(7, vec![0b1000100]),
        ];
        let sol = GF2Solver::attach(work, 3).unwrap();
        assert_eq!(sol.rows, 3);
        assert_eq!(sol.cols, 4);
        assert_eq!(sol.neqs, 3);
        assert_eq!(sol.rank, None);
        assert_eq!(sol.perm, &[0, 1, 2, 3]);
        assert_eq!(format!("{:}", sol.work[0]), "1000111");
        assert_eq!(format!("{:}", sol.work[1]), "0100011");
        assert_eq!(format!("{:}", sol.work[2]), "0010001");
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

    fn rand_co(rows: usize, cols: usize, p: f64) -> GF2Matrix {
        assert!(0.0 <= p && p <= 1.0);
        let mut rng = thread_rng();
        let mut co = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = FixedBitSet::with_capacity(cols);
            for c in 0..cols {
                if rng.gen::<f64>() < p {
                    row.insert(c);
                }
            }
            co.push(row);
        }
        co
    }

    fn rand_rhs(rows: usize, p: f64) -> FixedBitSet {
        assert!(0.0 <= p && p <= 1.0);
        let mut rng = thread_rng();
        let mut rhs = FixedBitSet::with_capacity(rows);
        for r in 0..rows {
            if rng.gen::<f64>() < p {
                rhs.insert(r);
            }
        }
        rhs
    }

    const REP: usize = 1000;

    #[template]
    #[rstest]
    fn template_tests(
        #[values(1, 2, 7, 12)] rows: usize,
        #[values(1, 2, 7, 12)] cols: usize,
        #[values(1, 2, 7, 12)] neqs: usize,
    ) {
    }

    #[apply(template_tests)]
    fn test_eliminate_lower_random(rows: usize, cols: usize, neqs: usize) {
        let mut rng = thread_rng();
        for _ in 0..REP {
            // Random p
            let p1 = rng.gen::<f64>();
            let p2 = rng.gen::<f64>();
            let co = rand_co(rows, cols, p1);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, p2));
            let mut sol = GF2Solver::new_from(&co, &rhs).unwrap();
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
            let mut sol = GF2Solver::new_from(&co, &rhs).unwrap();
            sol.eliminate_lower();
            assert!(sol.validate_afterlower());
        }
    }

    #[apply(template_tests)]
    fn test_eliminate_upper_random(rows: usize, cols: usize, neqs: usize) {
        let mut rng = thread_rng();
        for _ in 0..REP {
            // Random p
            let p1 = rng.gen::<f64>();
            let p2 = rng.gen::<f64>();
            let co = rand_co(rows, cols, p1);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, p2));
            let mut sol = GF2Solver::new_from(&co, &rhs).unwrap();
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
            let mut sol = GF2Solver::new_from(&co, &rhs).unwrap();
            sol.eliminate();
            assert!(sol.validate_afterupper());
        }
    }

    #[apply(template_tests)]
    fn test_solve_random(rows: usize, cols: usize, neqs: usize) {
        let mut rng = thread_rng();
        for _ in 0..REP {
            // Random p
            let p1 = rng.gen::<f64>();
            let p2 = rng.gen::<f64>();
            let co = rand_co(rows, cols, p1);
            let mut rhs = Vec::with_capacity(neqs);
            rhs.resize_with(neqs, || rand_rhs(rows, p2));
            let mut sol = GF2Solver::new_from(&co, &rhs).unwrap();
            sol.eliminate();
            for ieq in 0..neqs {
                let x = sol.solve(ieq);
                if x.is_none() {
                    assert!(sol.rank.unwrap() < sol.rows);
                    continue;
                }
                for i in sol.rank.unwrap()..sol.rows {
                    assert!(sol.work[i].count_ones(..sol.cols) == 0);
                    assert!(!sol.work[i][cols + ieq]);
                }
                let b = compute_lhs(&co, &x.unwrap());
                assert_eq!(b, rhs[ieq]);
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
            let mut sol = GF2Solver::new_from(&co, &rhs).unwrap();
            sol.eliminate();
            for ieq in 0..neqs {
                let x = sol.solve(ieq);
                if x.is_none() {
                    assert!(sol.rank.unwrap() < sol.rows);
                    continue;
                }
                for i in sol.rank.unwrap()..sol.rows {
                    assert!(sol.work[i].count_ones(..sol.cols) == 0);
                    assert!(!sol.work[i][cols + ieq]);
                }
                let b = compute_lhs(&co, &x.unwrap());
                assert_eq!(b, rhs[ieq]);
            }
        }
    }
}
