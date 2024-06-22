use fixedbitset::FixedBitSet;
use pyo3::prelude::*;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    common::{GFlow, InPlaceSetOp, Layer},
    gf2_linalg::GF2Solver,
};

#[pyfunction]
pub fn find(
    g: Vec<HashSet<usize>>,
    iset: HashSet<usize>,
    mut oset: HashSet<usize>,
) -> Option<(GFlow, Layer)> {
    let n = g.len();
    let vset = (0..n).collect::<HashSet<_>>();
    let mut cset = HashSet::new();
    // Need to use BTreeSet to get deterministic order
    let mut ocset = vset.difference(&oset).copied().collect::<BTreeSet<_>>();
    let mut omiset = oset.difference(&iset).copied().collect::<BTreeSet<_>>();
    // omivec[i] = i'th node in O\I after sorting
    let mut omivec = Vec::new();
    let mut f = HashMap::with_capacity(ocset.len());
    let mut layer = vec![0_usize; n];
    let mut nrows = ocset.len();
    let mut ncols = omiset.len();
    let mut neqs = ocset.len();
    // Reuse working memory
    let mut work = vec![FixedBitSet::with_capacity(ncols + neqs); nrows];
    let mut x = FixedBitSet::with_capacity(ncols);
    for l in 1_usize.. {
        cset.clear();
        nrows = ocset.len();
        work.truncate(nrows);
        ncols = omiset.len();
        neqs = ocset.len();
        // MEMO: Need to break before attaching
        if nrows == 0 || ncols == 0 || neqs == 0 {
            break;
        }
        work.iter_mut().for_each(|x| {
            // No allocation
            debug_assert!(x.len() >= ncols + neqs);
            x.grow(ncols + neqs);
            x.clear();
        });
        // Encode node as one-hot vector
        for (r, &u) in ocset.iter().enumerate() {
            // Initialize rhs
            work[r].insert(ncols + r);
            for (c, &v) in omiset.iter().enumerate() {
                // Initialize adjacency matrix
                if g[u].contains(&v) {
                    work[r].insert(c);
                }
            }
        }
        let mut solver = GF2Solver::attach(work, neqs).unwrap();
        omivec.clear();
        omiset.iter().for_each(|&u| omivec.push(u));
        for (ieq, &u) in ocset.iter().enumerate() {
            if !solver.solve_in_place(&mut x, ieq) {
                continue;
            }
            cset.insert(u);
            // Decode solution
            let fu = HashSet::from_iter(x.ones().map(|i| omivec[i]));
            f.insert(u, fu);
            layer[u] = l;
        }
        if cset.is_empty() {
            break;
        }
        oset.union_with(cset.iter());
        ocset.difference_with(cset.iter());
        omiset.union_with(cset.iter());
        work = solver.detach();
    }
    if oset == vset {
        Some((f, layer))
    } else {
        None
    }
}
