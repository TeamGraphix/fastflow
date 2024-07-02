//! Maximally-delayed generalized flow algorithm.

use crate::common::{self, Nodes, OrderedNodes};
use fixedbitset::FixedBitSet;
use hashbrown;
use num_traits::cast::FromPrimitive;
use pyo3::prelude::*;

use crate::{
    common::{Graph, InPlaceSetOp, Layer, Plane},
    gf2_linalg::GF2Solver,
};

// Introduced only for internal use
type InternalPlanes = hashbrown::HashMap<usize, u8>;
type Planes = hashbrown::HashMap<usize, Plane>;
type GFlow = hashbrown::HashMap<usize, Nodes>;

/// Checks if the domain of `f` is in V\O and the codomain is in V\I.
fn check_domain(f: &GFlow, vset: &Nodes, iset: &Nodes, oset: &Nodes) -> anyhow::Result<()> {
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
fn check_definition(f: &GFlow, layer: &Layer, g: &Graph, plane: &Planes) -> anyhow::Result<()> {
    anyhow::ensure!(
        f.len() == plane.len(),
        "f and plane must have the same codomain"
    );
    for &i in f.keys() {
        let fi = &f[&i];
        let pi = plane[&i];
        for &fij in fi {
            if i != fij && layer[i] <= layer[fij] {
                let err = anyhow::anyhow!("layer check failed")
                    .context(format!("neither {i} == {fij} nor {i} -> {fij}: fi"));
                return Err(err);
            }
        }
        let odd_fi = common::odd_neighbors(g, fi);
        for &j in &odd_fi {
            if i != j && layer[i] <= layer[j] {
                let err = anyhow::anyhow!("layer check failed").context(format!(
                    "neither {i} == {j} nor {i} -> {j}: odd_neighbors(g, fi)"
                ));
                return Err(err);
            }
        }
        let in_info = (fi.contains(&i), odd_fi.contains(&i));
        match pi {
            Plane::XY if in_info != (false, true) => {
                let err = anyhow::anyhow!("plane check failed")
                    .context(format!("must be {i} not in f({i}) and in Odd(f({i})): XY"));
                return Err(err);
            }
            Plane::YZ if in_info != (true, false) => {
                let err = anyhow::anyhow!("plane check failed")
                    .context(format!("must be {i} in f({i}) and not in Odd(f({i})): YZ"));
                return Err(err);
            }
            Plane::ZX if in_info != (true, true) => {
                let err = anyhow::anyhow!("plane check failed")
                    .context(format!("must be {i} in f({i}) and in Odd(f({i})): ZX"));
                return Err(err);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Initializes the working matrix.
fn init_work(
    work: &mut [FixedBitSet],
    g: &Graph,
    plane: &Planes,
    ocset: &OrderedNodes,
    omiset: &OrderedNodes,
) {
    let ncols = omiset.len();
    // Working memory for faster adjacency check
    let oc2i = ocset
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect::<hashbrown::HashMap<_, _>>();
    let omi2i = omiset
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect::<hashbrown::HashMap<_, _>>();
    // Encode node as one-hot vector
    for (i, &u) in ocset.iter().enumerate() {
        let gu = &g[u];
        // Initialize adjacency matrix
        let r = i;
        for &v in gu.iter() {
            if let Some(&c) = omi2i.get(&v) {
                work[r].insert(c);
            }
        }
        // Initialize rhs
        let ieq = i;
        let c = ncols + ieq;
        if let Plane::XY | Plane::ZX = plane[&u] {
            // = u
            work[ieq].insert(c);
        }
        if let Plane::XY = plane[&u] {
            continue;
        }
        // Include u
        for &v in gu.iter() {
            if let Some(&r) = oc2i.get(&v) {
                work[r].toggle(c);
            }
        }
    }
}

/// Finds the maximally-delayed generalized flow, if any.
///
/// # Arguments
///
/// - `g`: The adjacency list of the graph. Must be undirected and without self-loops.
/// - `iset`: The set of initial nodes.
/// - `oset`: The set of output nodes.
/// - `plane`: Measurement plane of each node in V\O.
///   - `0`: XY
///   - `1`: YZ
///   - `2`: ZX
///
/// # Note
///
/// - Node indices are assumed to be `0..g.len()`.
/// - Arguments are **NOT** verified.
#[pyfunction]
pub fn find(
    g: Graph,
    iset: Nodes,
    mut oset: Nodes,
    plane: InternalPlanes,
) -> Option<(GFlow, Layer)> {
    let plane = plane
        .into_iter()
        .map(|(k, v)| (k, Plane::from_u8(v).expect("plane is either 0, 1, or 2")))
        .collect::<Planes>();
    let n = g.len();
    let vset = (0..n).collect::<Nodes>();
    let mut cset = Nodes::new();
    // Need to use BTreeSet to get deterministic order
    let mut ocset = vset.difference(&oset).copied().collect::<OrderedNodes>();
    let mut omiset = oset.difference(&iset).copied().collect::<OrderedNodes>();
    let oset_orig = oset.clone();
    let mut f = GFlow::with_capacity(ocset.len());
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
        ncols = omiset.len();
        neqs = ocset.len();
        debug_assert!(work.len() >= nrows);
        work.truncate(nrows);
        // Decrease over time
        debug_assert!(work[0].len() >= ncols + neqs);
        common::zerofill(&mut work, ncols + neqs);
        init_work(&mut work, &g, &plane, &ocset, &omiset);
        let mut solver = GF2Solver::attach(work, neqs);
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
            let mut fu = x.ones().map(|c| tab[c]).collect::<Nodes>();
            if let Plane::YZ | Plane::ZX = plane[&u] {
                // Include u
                fu.insert(u);
            }
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
        check_definition(&f, &layer, &g, &plane).unwrap();
        // }
        Some((f, layer))
    } else {
        None
    }
}
