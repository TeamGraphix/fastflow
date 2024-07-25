//! Maximally-delayed Pauli flow algorithm.

use std::{
    iter,
    ops::{Deref, DerefMut},
};

use fixedbitset::FixedBitSet;
use hashbrown;
use log::Level;
use num_derive::FromPrimitive;
use num_enum::IntoPrimitive;
use num_traits::cast::FromPrimitive;
use pyo3::prelude::*;

use crate::{
    common::{self, Graph, InPlaceSetOp, Layer, Nodes, OrderedNodes},
    gf2_linalg::{self, GF2Solver},
};

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
/// Measurement planes or Pauli index.
enum PPlane {
    XY = 0,
    YZ = 1,
    ZX = 2,
    X = 3,
    Y = 4,
    Z = 5,
}

type InternalPPlanes = hashbrown::HashMap<usize, u8>;
type PPlanes = hashbrown::HashMap<usize, PPlane>;
type PFlow = hashbrown::HashMap<usize, Nodes>;

/// Checks the definition of Pauli flow.
fn check_definition(f: &PFlow, layer: &Layer, g: &Graph, pplanes: &PPlanes) -> anyhow::Result<()> {
    anyhow::ensure!(
        f.len() == pplanes.len(),
        "f and pplanes must have the same codomain"
    );
    for &i in f.keys() {
        let fi = &f[&i];
        let pi = pplanes[&i];
        for &fij in fi {
            match (i != fij, layer[i] <= layer[fij]) {
                (true, true) if !matches!(pplanes[&fij], PPlane::X | PPlane::Y) => {
                    let err = anyhow::anyhow!("layer check failed")
                        .context(format!("neither {i} == {fij} nor {i} -> {fij}: fi"));
                    return Err(err);
                }
                (false, false) => unreachable!("layer[i] == layer[i]"),
                _ => {}
            }
        }
        let odd_fi = common::odd_neighbors(g, fi);
        for &j in &odd_fi {
            match (i != j, layer[i] <= layer[j]) {
                (true, true) if !matches!(pplanes[&j], PPlane::Y | PPlane::Z) => {
                    let err = anyhow::anyhow!("layer check failed").context(format!(
                        "neither {i} == {j} nor {i} -> {j}: odd_neighbors(g, fi)"
                    ));
                    return Err(err);
                }
                (false, false) => unreachable!("layer[i] == layer[i]"),
                _ => {}
            }
        }
        for &j in fi.symmetric_difference(&odd_fi) {
            if pplanes.get(&j) == Some(&PPlane::Y) && i != j && layer[i] <= layer[j] {
                let err = anyhow::anyhow!("Y correction check failed")
                    .context(format!("{j} must be corrected by f({i}) xor Odd(f({i}))"));
                return Err(err);
            }
        }
        let in_info = (fi.contains(&i), odd_fi.contains(&i));
        match pi {
            PPlane::XY if in_info != (false, true) => {
                let err = anyhow::anyhow!("pplane check failed").context(format!(
                    "must satisfy ({i} in f({i}), {i} in Odd(f({i})) = (false, true): XY"
                ));
                return Err(err);
            }
            PPlane::YZ if in_info != (true, false) => {
                let err = anyhow::anyhow!("pplane check failed").context(format!(
                    "must satisfy ({i} in f({i}), {i} in Odd(f({i})) = (true, false): YZ"
                ));
                return Err(err);
            }
            PPlane::ZX if in_info != (true, true) => {
                let err = anyhow::anyhow!("pplane check failed").context(format!(
                    "must satisfy ({i} in f({i}), {i} in Odd(f({i})) = (true, true): ZX"
                ));
                return Err(err);
            }
            PPlane::X if !in_info.1 => {
                let err = anyhow::anyhow!("pplane check failed")
                    .context(format!("{i} must be in Odd(f({i})): X"));
                return Err(err);
            }
            PPlane::Y if !(in_info.0 ^ in_info.1) => {
                let err = anyhow::anyhow!("pplane check failed").context(format!(
                    "{i} must be in either f({i}) or Odd(f({i})), not both: Y"
                ));
                return Err(err);
            }
            PPlane::Z if !in_info.0 => {
                let err = anyhow::anyhow!("pplane check failed")
                    .context(format!("{i} must be in f({i}): Z"));
                return Err(err);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Initializes the upper block of working storage.
fn init_work_upper_co(
    work: &mut [FixedBitSet],
    g: &Graph,
    rowset: &OrderedNodes,
    colset: &OrderedNodes,
) {
    let colset2i = colset
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect::<hashbrown::HashMap<_, _>>();
    for (r, &v) in rowset.iter().enumerate() {
        let gv = &g[v];
        for &w in gv.iter() {
            if let Some(&c) = colset2i.get(&w) {
                work[r].insert(c);
            }
        }
    }
}

/// Initializes the lower block of working storage.
fn init_work_lower_co(
    work: &mut [FixedBitSet],
    g: &Graph,
    rowset: &OrderedNodes,
    colset: &OrderedNodes,
) {
    let colset2i = colset
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect::<hashbrown::HashMap<_, _>>();
    for (r, &v) in rowset.iter().enumerate() {
        // need to introduce self-loops
        if let Some(&c) = colset2i.get(&v) {
            work[r].insert(c);
        }
        let gv = &g[v];
        for &w in gv.iter() {
            if let Some(&c) = colset2i.get(&w) {
                work[r].insert(c);
            }
        }
    }
}

type BranchKind = u8;
const BRANCH_XY: BranchKind = 0;
const BRANCH_YZ: BranchKind = 1;
const BRANCH_ZX: BranchKind = 2;

/// Initializes the right-hand side of working storage for the upper block.
fn init_work_upper_rhs<const K: BranchKind>(
    work: &mut [FixedBitSet],
    u: usize,
    g: &Graph,
    rowset: &OrderedNodes,
    colset: &OrderedNodes,
) {
    const {
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_ZX);
    };
    debug_assert!(rowset.contains(&u));
    let rowset2i = rowset
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect::<hashbrown::HashMap<_, _>>();
    let c = colset.len();
    let gu = &g[u];
    if K != BRANCH_YZ {
        // = u
        work[rowset2i[&u]].insert(c);
    }
    if K == BRANCH_XY {
        return;
    }
    // Include u
    for &v in gu.iter() {
        if let Some(&r) = rowset2i.get(&v) {
            work[r].toggle(c);
        }
    }
}

/// Initializes the right-hand side of working storage for the lower block.
fn init_work_lower_rhs<const K: BranchKind>(
    work: &mut [FixedBitSet],
    u: usize,
    g: &Graph,
    rowset: &OrderedNodes,
    colset: &OrderedNodes,
) {
    const {
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_ZX);
    };
    let rowset2i = rowset
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .collect::<hashbrown::HashMap<_, _>>();
    let c = colset.len();
    let gu = &g[u];
    if K == BRANCH_XY {
        return;
    }
    for &v in gu.iter() {
        if let Some(&r) = rowset2i.get(&v) {
            work[r].toggle(c);
        }
    }
}

/// Initializes working storage for the given branch kind.
fn init_work<const K: BranchKind>(
    work: &mut [FixedBitSet],
    u: usize,
    g: &Graph,
    rowset_upper: &OrderedNodes,
    rowset_lower: &OrderedNodes,
    colset: &OrderedNodes,
) {
    const {
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_ZX);
    };
    let nrows_upper = rowset_upper.len();
    init_work_upper_co(&mut work[..nrows_upper], g, rowset_upper, colset);
    init_work_lower_co(&mut work[nrows_upper..], g, rowset_lower, colset);
    init_work_upper_rhs::<K>(&mut work[..nrows_upper], u, g, rowset_upper, colset);
    init_work_lower_rhs::<K>(&mut work[nrows_upper..], u, g, rowset_lower, colset);
}

/// Decodes the solution returned by `GF2Solver`.
fn decode_solution<const K: BranchKind>(u: usize, x: &FixedBitSet, i2v: &[usize]) -> Nodes {
    const {
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_ZX);
    };
    let mut fu = x.ones().map(|c| i2v[c]).collect::<Nodes>();
    if K != BRANCH_XY {
        fu.insert(u);
    }
    fu
}

/// Filters and collects nodes that match the given pattern.
macro_rules! matching_nodes {
    ($src:expr, $p:pat) => {
        $src.iter()
            .filter_map(|(k, &v)| if let $p = v { Some(k) } else { None })
            .copied()
            .collect::<Nodes>()
    };
}

#[derive(Debug)]
/// RAII guard for inserting a node into a set.
///
/// Inserts `u` on construction and reverts on drop.
struct ScopedInclude<'a> {
    target: &'a mut OrderedNodes,
    u: Option<usize>,
}

impl<'a> ScopedInclude<'a> {
    pub fn new(target: &'a mut OrderedNodes, u: usize) -> Self {
        let u = if target.insert(u) { Some(u) } else { None };
        Self { target, u }
    }
}

impl Deref for ScopedInclude<'_> {
    type Target = OrderedNodes;

    fn deref(&self) -> &Self::Target {
        self.target
    }
}

impl DerefMut for ScopedInclude<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.target
    }
}

impl Drop for ScopedInclude<'_> {
    fn drop(&mut self) {
        if let Some(u) = self.u {
            self.target.remove(&u);
        }
    }
}

#[derive(Debug)]
/// RAII guard for deleting a node from a set.
///
/// Removes `u` on construction and reverts on drop.
struct ScopedExclude<'a> {
    target: &'a mut OrderedNodes,
    u: Option<usize>,
}

impl<'a> ScopedExclude<'a> {
    pub fn new(target: &'a mut OrderedNodes, u: usize) -> Self {
        let u = if target.remove(&u) { Some(u) } else { None };
        Self { target, u }
    }
}

impl Deref for ScopedExclude<'_> {
    type Target = OrderedNodes;

    fn deref(&self) -> &Self::Target {
        self.target
    }
}

impl DerefMut for ScopedExclude<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.target
    }
}

impl Drop for ScopedExclude<'_> {
    fn drop(&mut self) {
        if let Some(u) = self.u {
            self.target.insert(u);
        }
    }
}

/// Logs the working storage.
///
/// For debugging purposes only.
fn log_work(work_upper: &[FixedBitSet], work_lower: &[FixedBitSet]) {
    if !log::log_enabled!(Level::Debug) {
        return;
    }
    let ncols = work_upper[0].len() - 1;
    log::debug!("work (upper):");
    for row in gf2_linalg::format_work(work_upper, ncols) {
        log::debug!("  {}", row);
    }
    log::debug!("work (lower):");
    for row in gf2_linalg::format_work(work_lower, ncols) {
        log::debug!("  {}", row);
    }
}

/// Finds the maximally-delayed Pauli flow.
///
/// # Arguments
///
/// - `g`: The adjacency list of the graph. Must be undirected and without self-loops.
/// - `iset`: The set of initial nodes.
/// - `oset`: The set of output nodes.
/// - `pplanes`: Measurement plane of each node in `&vset - &oset`.
///   - `0`: XY
///   - `1`: YZ
///   - `2`: ZX
///   - `3`: X
///   - `4`: Y
///   - `5`: Z
///
/// # Note
///
/// - Node indices are assumed to be `0..g.len()`.
/// - Arguments are **NOT** verified.
#[pyfunction]
pub fn find(
    g: Graph,
    iset: Nodes,
    oset: Nodes,
    pplanes: InternalPPlanes,
) -> Option<(PFlow, Layer)> {
    log::debug!("pflow::find");
    let pplanes = pplanes
        .into_iter()
        .map(|(k, v)| (k, PPlane::from_u8(v).expect("pplane is in 0..6")))
        .collect::<PPlanes>();
    let yset = matching_nodes!(pplanes, PPlane::Y);
    let xyset = matching_nodes!(pplanes, PPlane::X | PPlane::Y);
    let yzset = matching_nodes!(pplanes, PPlane::Y | PPlane::Z);
    debug_assert!(yset.is_disjoint(&oset));
    debug_assert!(xyset.is_disjoint(&oset));
    debug_assert!(yzset.is_disjoint(&oset));
    let n = g.len();
    let vset = (0..n).collect::<Nodes>();
    let mut cset = Nodes::new();
    let mut ocset = vset.difference(&oset).copied().collect::<Nodes>();
    let mut rowset_upper = vset.difference(&yzset).copied().collect::<OrderedNodes>();
    let mut rowset_lower = yset.iter().copied().collect::<OrderedNodes>();
    let mut colset = xyset.difference(&iset).copied().collect::<OrderedNodes>();
    let mut f = PFlow::with_capacity(ocset.len());
    let mut layer = vec![0_usize; n];
    let mut work = vec![FixedBitSet::new(); rowset_upper.len() + rowset_lower.len()];
    let mut i2v = Vec::new();
    for l in 0_usize.. {
        log::debug!("=====layer {l}=====");
        cset.clear();
        for &u in &ocset {
            let rowset_upper = ScopedInclude::new(&mut rowset_upper, u);
            let rowset_lower = ScopedExclude::new(&mut rowset_lower, u);
            let colset = ScopedExclude::new(&mut colset, u);
            let nrows_upper = rowset_upper.len();
            let nrows_lower = rowset_lower.len();
            let ncols = colset.len();
            if nrows_upper + nrows_lower == 0 || ncols == 0 {
                continue;
            }
            let ppu = pplanes[&u];
            log::debug!("====checking {u} ({ppu:?})====");
            log::debug!("rowset_upper: {:?}", &*rowset_upper);
            log::debug!("rowset_lower: {:?}", &*rowset_lower);
            log::debug!("colset      : {:?}", &*colset);
            // No monotonicity guarantees
            work.resize_with(nrows_upper + nrows_lower, || {
                FixedBitSet::with_capacity(ncols + 1)
            });
            i2v.clear();
            i2v.extend(colset.iter().copied());
            let mut x = FixedBitSet::with_capacity(ncols);
            let mut done = false;
            if !done && matches!(ppu, PPlane::XY | PPlane::X | PPlane::Y) {
                log::debug!("===XY branch===");
                x.clear();
                common::zerofill(&mut work, ncols + 1);
                init_work::<BRANCH_XY>(&mut work, u, &g, &rowset_upper, &rowset_lower, &colset);
                log_work(&work[..nrows_upper], &work[nrows_upper..]);
                let mut solver = GF2Solver::attach(work, 1);
                if solver.solve_in_place(&mut x, 0) {
                    log::debug!("solution found for {u} (XY)");
                    f.insert(u, decode_solution::<BRANCH_XY>(u, &x, &i2v));
                    done = true;
                } else {
                    log::debug!("solution not found: {u} (XY)");
                }
                work = solver.detach();
            }
            if !done && matches!(ppu, PPlane::YZ | PPlane::Y | PPlane::Z) {
                log::debug!("===YZ branch===");
                x.clear();
                common::zerofill(&mut work, ncols + 1);
                init_work::<BRANCH_YZ>(&mut work, u, &g, &rowset_upper, &rowset_lower, &colset);
                log_work(&work[..nrows_upper], &work[nrows_upper..]);
                let mut solver = GF2Solver::attach(work, 1);
                if solver.solve_in_place(&mut x, 0) {
                    log::debug!("solution found for {u} (YZ)");
                    f.insert(u, decode_solution::<BRANCH_YZ>(u, &x, &i2v));
                    done = true;
                } else {
                    log::debug!("solution not found: {u} (YZ)");
                }
                work = solver.detach();
            }
            if !done && matches!(ppu, PPlane::ZX | PPlane::Z | PPlane::X) {
                log::debug!("===ZX branch===");
                x.clear();
                common::zerofill(&mut work, ncols + 1);
                init_work::<BRANCH_ZX>(&mut work, u, &g, &rowset_upper, &rowset_lower, &colset);
                log_work(&work[..nrows_upper], &work[nrows_upper..]);
                let mut solver = GF2Solver::attach(work, 1);
                if solver.solve_in_place(&mut x, 0) {
                    log::debug!("solution found for {u} (ZX)");
                    f.insert(u, decode_solution::<BRANCH_ZX>(u, &x, &i2v));
                    done = true;
                } else {
                    log::debug!("solution not found: {u} (ZX)");
                }
                work = solver.detach();
            }
            if done {
                log::debug!("f({}) = {:?}", u, &f[&u]);
                log::debug!("layer({u}) = {l}");
                layer[u] = l;
                cset.insert(u);
            } else {
                log::debug!("solution not found: {u} (all branches)");
            }
        }
        if l == 0 {
            rowset_upper.difference_with(&oset);
            rowset_lower.difference_with(&oset);
            colset.union_with(oset.difference(&iset));
        } else if cset.is_empty() {
            break;
        }
        ocset.difference_with(&cset);
        rowset_upper.difference_with(&cset);
        rowset_lower.difference_with(&cset);
        colset.union_with(cset.difference(&iset));
    }
    if ocset.is_empty() {
        log::debug!("pflow found");
        log::debug!("pflow: {f:?}");
        log::debug!("layer: {layer:?}");
        // TODO: Uncomment once ready
        // if cfg!(debug_assertions) {
        let f_flatiter = f
            .iter()
            .flat_map(|(i, fi)| Iterator::zip(iter::repeat(i), fi.iter()));
        common::check_domain(f_flatiter, &vset, &iset, &oset).unwrap();
        common::check_initial(&layer, &oset, false).unwrap();
        check_definition(&f, &layer, &g, &pplanes).unwrap();
        // }
        Some((f, layer))
    } else {
        log::debug!("pflow not found");
        None
    }
}

#[cfg(test)]
mod tests {
    use test_log;

    use super::*;
    use crate::test_utils::{self, TestCase};

    #[test_log::test]
    fn test_find_case0() {
        let TestCase { g, iset, oset } = test_utils::CASE0.clone();
        let pplanes = measurements! {};
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, pplanes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(layer, vec![0, 0]);
    }

    #[test_log::test]
    fn test_find_case1() {
        let TestCase { g, iset, oset } = test_utils::CASE1.clone();
        let pplanes = measurements! {
            0: PPlane::XY,
            1: PPlane::XY,
            2: PPlane::XY,
            3: PPlane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, pplanes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodes![1]);
        assert_eq!(f[&1], nodes![2]);
        assert_eq!(f[&2], nodes![3]);
        assert_eq!(f[&3], nodes![4]);
        assert_eq!(layer, vec![4, 3, 2, 1, 0]);
    }

    #[test_log::test]
    fn test_find_case2() {
        let TestCase { g, iset, oset } = test_utils::CASE2.clone();
        let pplanes = measurements! {
            0: PPlane::XY,
            1: PPlane::XY,
            2: PPlane::XY,
            3: PPlane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, pplanes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodes![2]);
        assert_eq!(f[&1], nodes![3]);
        assert_eq!(f[&2], nodes![4]);
        assert_eq!(f[&3], nodes![5]);
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
    }

    #[test_log::test]
    fn test_find_case3() {
        let TestCase { g, iset, oset } = test_utils::CASE3.clone();
        let pplanes = measurements! {
            0: PPlane::XY,
            1: PPlane::XY,
            2: PPlane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, pplanes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodes![4, 5]);
        assert_eq!(f[&1], nodes![3, 4, 5]);
        assert_eq!(f[&2], nodes![3, 5]);
        assert_eq!(layer, vec![1, 1, 1, 0, 0, 0]);
    }

    #[test_log::test]
    fn test_find_case4() {
        let TestCase { g, iset, oset } = test_utils::CASE4.clone();
        let pplanes = measurements! {
            0: PPlane::XY,
            1: PPlane::XY,
            2: PPlane::ZX,
            3: PPlane::YZ
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, pplanes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodes![2]);
        assert_eq!(f[&1], nodes![5]);
        assert_eq!(f[&2], nodes![2, 4]);
        assert_eq!(f[&3], nodes![3]);
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
    }

    #[test_log::test]
    fn test_find_case5() {
        let TestCase { g, iset, oset } = test_utils::CASE5.clone();
        let pplanes = measurements! {
            0: PPlane::XY,
            1: PPlane::XY
        };
        assert!(find(g, iset, oset, pplanes).is_none());
    }

    #[test_log::test]
    fn test_find_case6() {
        let TestCase { g, iset, oset } = test_utils::CASE6.clone();
        let pplanes = measurements! {
            0: PPlane::XY,
            1: PPlane::X,
            2: PPlane::XY,
            3: PPlane::X
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, pplanes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodes![1]);
        assert_eq!(f[&1], nodes![4]);
        assert_eq!(f[&2], nodes![3]);
        assert_eq!(f[&3], nodes![2, 4]);
        assert_eq!(layer, vec![1, 1, 0, 1, 0]);
    }

    #[test_log::test]
    fn test_find_case7() {
        let TestCase { g, iset, oset } = test_utils::CASE7.clone();
        let pplanes = measurements! {
            0: PPlane::Z,
            1: PPlane::Z,
            2: PPlane::Y,
            3: PPlane::Y
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, pplanes).unwrap();
        assert_eq!(f.len(), flen);
        // Graphix
        // assert_eq!(f[&0], nodes![0, 1]);
        assert_eq!(f[&0], nodes![0]);
        assert_eq!(f[&1], nodes![1]);
        assert_eq!(f[&2], nodes![2]);
        assert_eq!(f[&3], nodes![4]);
        assert_eq!(layer, vec![1, 0, 0, 1, 0]);
    }

    #[test_log::test]
    fn test_find_case8() {
        let TestCase { g, iset, oset } = test_utils::CASE8.clone();
        let pplanes = measurements! {
            0: PPlane::Z,
            1: PPlane::ZX,
            2: PPlane::Y
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, pplanes).unwrap();
        assert_eq!(f.len(), flen);
        // Graphix
        // assert_eq!(f[&0], nodes![0, 3, 4]);
        assert_eq!(f[&0], nodes![0, 2, 4]);
        assert_eq!(f[&1], nodes![1, 2]);
        assert_eq!(f[&2], nodes![4]);
        assert_eq!(layer, vec![1, 1, 1, 0, 0]);
    }
}
