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

    pub fn solve_in_place(&mut self, out: &mut FixedBitSet, ieq: usize) -> Option<()> {
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
                    return None;
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
        Some(())
    }

    // Left for easier testing
    #[allow(dead_code)]
    pub fn solve(&mut self, ieq: usize) -> Option<FixedBitSet> {
        let mut out = FixedBitSet::with_capacity(self.cols);
        self.solve_in_place(&mut out, ieq)?;
        Some(out)
    }
}

