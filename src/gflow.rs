//! Maximally-delayed generalized flow algorithm.

use fixedbitset::FixedBitSet;
use pyo3::prelude::*;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    common::{self, Graph, InPlaceSetOp, Layer},
    gf2_linalg::GF2Solver,
};

type GFlow = HashMap<usize, HashSet<usize>>;

/// Checks if the domain of `f` is in V\O and the codomain is in V\I.
fn check_domain(
    f: &GFlow,
    vset: &HashSet<usize>,
    iset: &HashSet<usize>,
    oset: &HashSet<usize>,
) -> anyhow::Result<()> {
    let icset = vset - iset;
    let ocset = vset - oset;
    for &i in f.keys() {
        if !ocset.contains(&i) {
            let err = anyhow::anyhow!("domain check failed").context(format!("{i} not in V\\O"));
            return Err(err);
        }
    }
    for &fij in f.values().flatten() {
        if !icset.contains(&fij) {
            let err = anyhow::anyhow!("domain check failed").context(format!("{fij} not in V\\I"));
            return Err(err);
        }
    }
    Ok(())
}

/// Checks if the properties of the generalized flow are satisfied.
fn check_definition(f: &GFlow, layer: &Layer, g: &Graph) -> anyhow::Result<()> {
    for (&i, fi) in f.iter() {
        for &fij in fi {
            if layer[i] <= layer[fij] {
                let err =
                    anyhow::anyhow!("layer check failed").context(format!("must be {i} -> {fij}"));
                return Err(err);
            }
        }
        let odd_fi = common::odd_neighbors(g, fi);
        for &j in &odd_fi {
            if i != j && layer[i] <= layer[j] {
                let err = anyhow::anyhow!("layer check failed")
                    .context(format!("neither {i} == {j} nor {i} -> {j}"));
                return Err(err);
            }
        }
        if !odd_fi.contains(&i) {
            let err = anyhow::anyhow!("graph check failed")
                .context(format!("{i} and Odd(f({i})) not connected"));
            return Err(err);
        }
    }
    Ok(())
}

/// Resizes `mat` to `mat.len()` x `ncols` and fills with zeros.
fn zerofill(mat: &mut [FixedBitSet], ncols: usize) {
    let src = FixedBitSet::with_capacity(ncols);
    mat.iter_mut().for_each(|x| {
        x.clone_from(&src);
    });
}

/// Finds the maximally-delayed generalized flow, if any.
///
/// # Arguments
///
/// - `g`: The adjacency list of the graph. Must be undirected and without self-loops.
/// - `iset`: The set of initial nodes. Must be consistent with `g`.
/// - `oset`: The set of output nodes. Must be consistent with `g`.
///
/// # Note
///
/// - Node indices are assumed to be `0..g.len()`.
/// - Arguments are **NOT** verified.
#[pyfunction]
pub fn find(g: Graph, iset: HashSet<usize>, mut oset: HashSet<usize>) -> Option<(GFlow, Layer)> {
    let n = g.len();
    let vset = (0..n).collect::<HashSet<_>>();
    let mut cset = HashSet::new();
    // Need to use BTreeSet to get deterministic order
    let mut ocset = vset.difference(&oset).copied().collect::<BTreeSet<_>>();
    let mut omiset = oset.difference(&iset).copied().collect::<BTreeSet<_>>();
    let oset_orig = oset.clone();
    let mut f = HashMap::with_capacity(ocset.len());
    let mut layer = vec![0_usize; n];
    let mut nrows = ocset.len();
    let mut ncols = omiset.len();
    let mut neqs = ocset.len();
    // Reuse working memory
    let mut work = vec![FixedBitSet::with_capacity(ncols + neqs); nrows];
    let mut tab = Vec::new();
    for l in 1_usize.. {
        cset.clear();
        if ocset.is_empty() || omiset.is_empty() {
            break;
        }
        // Decrease over time
        nrows = ocset.len();
        // Increase over time
        ncols = omiset.len();
        // Decrease over time
        neqs = ocset.len();
        debug_assert!(work.len() >= nrows);
        work.truncate(nrows);
        zerofill(&mut work, ncols + neqs);
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
        let mut x = FixedBitSet::with_capacity(ncols);
        // tab[i] = node index assigned to one-hot vector x[i]
        tab.clear();
        tab.extend(omiset.iter().copied());
        for (ieq, &u) in ocset.iter().enumerate() {
            if !solver.solve_in_place(&mut x, ieq) {
                continue;
            }
            cset.insert(u);
            // Decode solution
            let fu = x.ones().map(|c| tab[c]).collect();
            f.insert(u, fu);
            layer[u] = l;
        }
        if cset.is_empty() {
            break;
        }
        oset.union_with(cset.iter());
        ocset.difference_with(cset.iter());
        omiset.union_with(cset.difference(&iset));
        work = solver.detach();
    }
    if oset == vset {
        // TODO: Uncomment once ready
        // if cfg!(debug_assertions) {
        check_domain(&f, &vset, &iset, &oset_orig).unwrap();
        common::check_initial(&layer, &oset_orig).unwrap();
        check_definition(&f, &layer, &g).unwrap();
        // }
        Some((f, layer))
    } else {
        None
    }
}
