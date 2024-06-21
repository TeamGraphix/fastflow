use fixedbitset::FixedBitSet;
use pyo3::prelude::*;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    common::{GFlow, Layer},
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
    // Need to use BTreeSet to get deterministic order
    let mut ocset = vset.difference(&oset).copied().collect::<BTreeSet<_>>();
    let mut cset = HashSet::new();
    let mut omiset = oset.difference(&iset).copied().collect::<BTreeSet<_>>();
    let mut f = HashMap::with_capacity(ocset.len());
    let mut layer = vec![0_usize; n];
    let mut nrows;
    let mut ncols = omiset.len();
    let mut neqs = ocset.len();
    let mut work = Vec::new();
    let mut x = FixedBitSet::with_capacity(ncols);
    work.resize_with(ocset.len(), || {
        // ncols + neqs monotonically decreases
        FixedBitSet::with_capacity(ncols + neqs)
    });
    // Used to decode x
    let mut omivec = Vec::new();
    for l in 1_usize.. {
        cset.clear();
        nrows = ocset.len();
        work.truncate(nrows);
        ncols = omiset.len();
        neqs = ocset.len();
        if nrows == 0 || ncols == 0 || neqs == 0 {
            break;
        }
        work.iter_mut().for_each(|x| {
            // No allocation
            debug_assert!(x.len() >= ncols + neqs);
            x.grow(ncols + neqs);
            x.clear();
        });
        for (r, &u) in ocset.iter().enumerate() {
            work[r].insert(ncols + r);
            for (c, &v) in omiset.iter().enumerate() {
                if g[u].contains(&v) {
                    work[r].insert(c);
                }
            }
        }
        let mut solver = GF2Solver::attach(work, neqs).unwrap();
        omivec.clear();
        omiset.iter().for_each(|&u| omivec.push(u));
        for (ieq, &u) in ocset.iter().enumerate() {
            if let None = solver.solve_in_place(&mut x, ieq) {
                continue;
            }
            cset.insert(u);
            let fu = HashSet::from_iter(x.ones().map(|i| omivec[i]));
            f.insert(u, fu);
            layer[u] = l;
        }
        if cset.is_empty() {
            break;
        }
        for &u in cset.iter() {
            oset.insert(u);
            ocset.remove(&u);
            omiset.insert(u);
        }
        work = solver.detach();
    }
    if oset == vset {
        return Some((f, layer));
    } else {
        return None;
    }
}
