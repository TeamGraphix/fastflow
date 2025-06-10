//! Maximally-delayed Pauli flow algorithm.

use std::iter;

use fixedbitset::FixedBitSet;
use hashbrown;
use pyo3::prelude::*;

use crate::{
    common::{
        FlowValidationError::{
            self, InconsistentFlowOrder, InconsistentFlowPPlane, InvalidMeasurementSpec,
        },
        Graph, Layer, Nodes, OrderedNodes, FATAL_MSG,
    },
    internal::{
        gf2_linalg::GF2Solver,
        utils::{self, InPlaceSetDiff, ScopedExclude, ScopedInclude},
        validate,
    },
};

#[pyclass(eq, hash, frozen)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// Enum-like class for measurement planes or Pauli measurements.
pub enum PPlane {
    XY,
    YZ,
    XZ,
    X,
    Y,
    Z,
}

type PPlanes = hashbrown::HashMap<usize, PPlane>;
type PFlow = hashbrown::HashMap<usize, Nodes>;

/// Checks the definition of Pauli flow.
fn check_definition(
    f: &PFlow,
    layer: &Layer,
    g: &Graph,
    pplanes: &PPlanes,
) -> Result<(), FlowValidationError> {
    for &i in itertools::chain(f.keys(), pplanes.keys()) {
        if f.contains_key(&i) != pplanes.contains_key(&i) {
            Err(InvalidMeasurementSpec { node: i })?;
        }
    }
    for (&i, fi) in f {
        let pi = pplanes[&i];
        for &fij in fi {
            match (i != fij, layer[i] <= layer[fij]) {
                (true, true) if !matches!(pplanes.get(&fij), Some(&PPlane::X | &PPlane::Y)) => {
                    Err(InconsistentFlowOrder { nodes: (i, fij) })?;
                }
                (false, false) => unreachable!("layer[i] == layer[i]"),
                _ => {}
            }
        }
        let odd_fi = utils::odd_neighbors(g, fi);
        for &j in &odd_fi {
            match (i != j, layer[i] <= layer[j]) {
                (true, true) if !matches!(pplanes.get(&j), Some(&PPlane::Y | &PPlane::Z)) => {
                    Err(InconsistentFlowOrder { nodes: (i, j) })?;
                }
                (false, false) => unreachable!("layer[i] == layer[i]"),
                _ => {}
            }
        }
        for &j in fi.symmetric_difference(&odd_fi) {
            if pplanes.get(&j) == Some(&PPlane::Y) && i != j && layer[i] <= layer[j] {
                Err(InconsistentFlowPPlane {
                    node: i,
                    pplane: PPlane::Y,
                })?;
            }
        }
        let in_info = (fi.contains(&i), odd_fi.contains(&i));
        match pi {
            PPlane::XY if in_info != (false, true) => {
                Err(InconsistentFlowPPlane {
                    node: i,
                    pplane: PPlane::XY,
                })?;
            }
            PPlane::YZ if in_info != (true, false) => {
                Err(InconsistentFlowPPlane {
                    node: i,
                    pplane: PPlane::YZ,
                })?;
            }
            PPlane::XZ if in_info != (true, true) => {
                Err(InconsistentFlowPPlane {
                    node: i,
                    pplane: PPlane::XZ,
                })?;
            }
            PPlane::X if !in_info.1 => {
                Err(InconsistentFlowPPlane {
                    node: i,
                    pplane: PPlane::X,
                })?;
            }
            PPlane::Y if !(in_info.0 ^ in_info.1) => {
                Err(InconsistentFlowPPlane {
                    node: i,
                    pplane: PPlane::Y,
                })?;
            }
            PPlane::Z if !in_info.0 => {
                Err(InconsistentFlowPPlane {
                    node: i,
                    pplane: PPlane::Z,
                })?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Sellects nodes from `src` with `pred`.
fn matching_nodes(src: &PPlanes, mut pred: impl FnMut(&PPlane) -> bool) -> Nodes {
    src.iter()
        .filter_map(|(&k, v)| if pred(v) { Some(k) } else { None })
        .collect()
}

/// Initializes the upper block of working storage.
fn init_work_upper_co(
    work: &mut [FixedBitSet],
    g: &Graph,
    rowset: &OrderedNodes,
    colset: &OrderedNodes,
) {
    let colset2i = utils::indexmap::<hashbrown::HashMap<_, _>>(colset);
    for (r, &v) in rowset.iter().enumerate() {
        let gv = &g[v];
        for &w in gv {
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
    let colset2i = utils::indexmap::<hashbrown::HashMap<_, _>>(colset);
    for (r, &v) in rowset.iter().enumerate() {
        // need to introduce self-loops
        if let Some(&c) = colset2i.get(&v) {
            work[r].insert(c);
        }
        let gv = &g[v];
        for &w in gv {
            if let Some(&c) = colset2i.get(&w) {
                work[r].insert(c);
            }
        }
    }
}

type BranchKind = u8;
const BRANCH_XY: BranchKind = 0;
const BRANCH_YZ: BranchKind = 1;
const BRANCH_XZ: BranchKind = 2;

/// Initializes the right-hand side of working storage for the upper block.
fn init_work_upper_rhs<const K: BranchKind>(
    work: &mut [FixedBitSet],
    u: usize,
    g: &Graph,
    rowset: &OrderedNodes,
    colset: &OrderedNodes,
) {
    const {
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_XZ);
    };
    debug_assert!(rowset.contains(&u));
    let rowset2i = utils::indexmap::<hashbrown::HashMap<_, _>>(rowset);
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
    for &v in gu {
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
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_XZ);
    };
    let rowset2i = utils::indexmap::<hashbrown::HashMap<_, _>>(rowset);
    let c = colset.len();
    let gu = &g[u];
    if K == BRANCH_XY {
        return;
    }
    for &v in gu {
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
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_XZ);
    };
    let nrows_upper = rowset_upper.len();
    init_work_upper_co(&mut work[..nrows_upper], g, rowset_upper, colset);
    init_work_lower_co(&mut work[nrows_upper..], g, rowset_lower, colset);
    init_work_upper_rhs::<K>(&mut work[..nrows_upper], u, g, rowset_upper, colset);
    init_work_lower_rhs::<K>(&mut work[nrows_upper..], u, g, rowset_lower, colset);
}

/// Decodes the solution returned by `GF2Solver`.
fn decode_solution<const K: BranchKind>(u: usize, x: &FixedBitSet, colset: &OrderedNodes) -> Nodes {
    const {
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_XZ);
    };
    let mut fu = colset
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| if x[i] { Some(v) } else { None })
        .collect::<Nodes>();
    if K != BRANCH_XY {
        fu.insert(u);
    }
    fu
}

#[derive(Debug)]
struct PFlowContext<'a> {
    work: &'a mut Vec<FixedBitSet>,
    g: &'a Graph,
    u: usize,
    rowset_upper: &'a OrderedNodes,
    rowset_lower: &'a OrderedNodes,
    colset: &'a OrderedNodes,
    x: &'a mut FixedBitSet,
    f: &'a mut PFlow,
}

/// Implements the branch-specific part of the algorithm.
#[tracing::instrument]
fn find_impl<const K: BranchKind>(ctx: &mut PFlowContext) -> bool {
    const {
        assert!(K == BRANCH_XY || K == BRANCH_YZ || K == BRANCH_XZ);
    };
    let u = ctx.u;
    ctx.x.clear();
    let ncols = ctx.colset.len();
    utils::zerofill(ctx.work, ncols + 1);
    init_work::<K>(
        ctx.work,
        u,
        ctx.g,
        ctx.rowset_upper,
        ctx.rowset_lower,
        ctx.colset,
    );
    let mut solver = GF2Solver::attach(ctx.work.as_mut_slice(), 1);
    tracing::debug!("{solver:?}");
    if solver.solve_in_place(ctx.x, 0) {
        tracing::debug!("solution found for {u}");
        ctx.f.insert(u, decode_solution::<K>(u, ctx.x, ctx.colset));
        true
    } else {
        tracing::debug!("solution not found: {u}");
        false
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
///
/// # Panics
///
/// If inputs/outputs do not pass the validation.
///
/// # Note
///
/// - Node indices are assumed to be `0..g.len()`.
/// - Arguments are **NOT** verified.
#[pyfunction]
#[tracing::instrument]
#[allow(clippy::needless_pass_by_value)]
pub fn find(g: Graph, iset: Nodes, oset: Nodes, pplanes: PPlanes) -> Option<(PFlow, Layer)> {
    let yset = matching_nodes(&pplanes, |pp| matches!(pp, PPlane::Y));
    let xyset = matching_nodes(&pplanes, |pp| matches!(pp, PPlane::X | PPlane::Y));
    let yzset = matching_nodes(&pplanes, |pp| matches!(pp, PPlane::Y | PPlane::Z));
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
    for l in 0_usize.. {
        tracing::debug!("=====layer {l}=====");
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
            tracing::debug!("====checking {u} ({ppu:?})====");
            tracing::debug!("rowset_upper: {:?}", &*rowset_upper);
            tracing::debug!("rowset_lower: {:?}", &*rowset_lower);
            tracing::debug!("colset      : {:?}", &*colset);
            // No monotonicity guarantees
            work.resize_with(nrows_upper + nrows_lower, || {
                FixedBitSet::with_capacity(ncols + 1)
            });
            let mut x = FixedBitSet::with_capacity(ncols);
            let mut done = false;
            let mut ctx = PFlowContext {
                work: &mut work,
                g: &g,
                u,
                rowset_upper: &rowset_upper,
                rowset_lower: &rowset_lower,
                colset: &colset,
                x: &mut x,
                f: &mut f,
            };
            if !done && matches!(ppu, PPlane::XY | PPlane::X | PPlane::Y) {
                tracing::debug!("===XY branch===");
                done |= find_impl::<BRANCH_XY>(&mut ctx);
            }
            if !done && matches!(ppu, PPlane::YZ | PPlane::Y | PPlane::Z) {
                tracing::debug!("===YZ branch===");
                done |= find_impl::<BRANCH_YZ>(&mut ctx);
            }
            if !done && matches!(ppu, PPlane::XZ | PPlane::Z | PPlane::X) {
                tracing::debug!("===XZ branch===");
                done |= find_impl::<BRANCH_XZ>(&mut ctx);
            }
            if done {
                tracing::debug!("f({}) = {:?}", u, &f[&u]);
                tracing::debug!("layer({u}) = {l}");
                layer[u] = l;
                cset.insert(u);
            } else {
                tracing::debug!("solution not found: {u} (all branches)");
            }
        }
        if l == 0 {
            rowset_upper.difference_with(&oset);
            rowset_lower.difference_with(&oset);
            colset.extend(oset.difference(&iset));
        } else if cset.is_empty() {
            break;
        }
        ocset.difference_with(&cset);
        rowset_upper.difference_with(&cset);
        rowset_lower.difference_with(&cset);
        colset.extend(cset.difference(&iset));
    }
    if ocset.is_empty() {
        tracing::debug!("pflow found");
        tracing::debug!("pflow: {f:?}");
        tracing::debug!("layer: {layer:?}");
        // TODO: Remove this block once stabilized
        {
            let f_flatiter = f
                .iter()
                .flat_map(|(i, fi)| Iterator::zip(iter::repeat(i), fi.iter()));
            validate::check_domain(f_flatiter, &vset, &iset, &oset).expect(FATAL_MSG);
            validate::check_initial(&layer, &oset, false).expect(FATAL_MSG);
            check_definition(&f, &layer, &g, &pplanes).expect(FATAL_MSG);
        }
        Some((f, layer))
    } else {
        tracing::debug!("pflow not found");
        None
    }
}

/// Validates Pauli flow.
///
/// # Errors
///
/// - If `pflow` is invalid.
/// - If `pflow` is inconsistent with `g`.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
pub fn verify(
    pflow: (PFlow, Layer),
    g: Graph,
    iset: Nodes,
    oset: Nodes,
    pplanes: PPlanes,
) -> PyResult<()> {
    let (f, layer) = pflow;
    let n = g.len();
    let vset = (0..n).collect::<Nodes>();
    let f_flatiter = f
        .iter()
        .flat_map(|(i, fi)| Iterator::zip(iter::repeat(i), fi.iter()));
    validate::check_domain(f_flatiter, &vset, &iset, &oset)?;
    validate::check_initial(&layer, &oset, false)?;
    check_definition(&f, &layer, &g, &pplanes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use test_log;

    use super::*;
    use crate::internal::test_utils::{self, TestCase};

    #[test_log::test]
    fn test_check_definition_ng() {
        // Missing Plane specification
        assert_eq!(
            check_definition(
                &map! { 0: set!{1} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! {},
            ),
            Err(InvalidMeasurementSpec { node: 0 })
        );
        // Violate 0 -> f(0) = 1
        assert_eq!(
            check_definition(
                &map! { 0: set!{1} },
                &vec![0, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: PPlane::XY },
            ),
            Err(InconsistentFlowOrder { nodes: (0, 1) })
        );
        // Violate 1 in nb(f(0)) = nb(2) => 0 == 1 or 0 -> 1
        assert_eq!(
            check_definition(
                &map! { 0: set!{2}, 1: set!{2} },
                &vec![1, 1, 0],
                &test_utils::graph(&[(0, 1), (1, 2)]),
                &map! {
                    0: PPlane::XY,
                    1: PPlane::XY
                },
            ),
            Err(InconsistentFlowOrder { nodes: (0, 1) })
        );
        // Violate Y: 0 != 1 and not 0 -> 1 and 1 in f(0) ^ Odd(f(0))
        assert_eq!(
            check_definition(
                &map! { 0: set!{1}, 1: set!{2} },
                &vec![1, 1, 0],
                &test_utils::graph(&[(0, 1), (1, 2)]),
                &map! { 0: PPlane::XY, 1: PPlane::Y },
            ),
            Err(InconsistentFlowPPlane {
                node: 0,
                pplane: PPlane::Y,
            })
        );
        // Violate XY: 0 in f(0)
        assert_eq!(
            check_definition(
                &map! { 0: set!{0} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: PPlane::XY },
            ),
            Err(InconsistentFlowPPlane {
                node: 0,
                pplane: PPlane::XY
            })
        );
        // Violate YZ: 0 in Odd(f(0))
        assert_eq!(
            check_definition(
                &map! { 0: set!{1} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: PPlane::YZ },
            ),
            Err(InconsistentFlowPPlane {
                node: 0,
                pplane: PPlane::YZ
            })
        );
        // Violate XZ: 0 not in Odd(f(0)) and in f(0)
        assert_eq!(
            check_definition(
                &map! { 0: set!{0} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: PPlane::XZ },
            ),
            Err(InconsistentFlowPPlane {
                node: 0,
                pplane: PPlane::XZ
            })
        );
        // Violate XZ: 0 in Odd(f(0)) and not in f(0)
        assert_eq!(
            check_definition(
                &map! { 0: set!{1} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: PPlane::XZ },
            ),
            Err(InconsistentFlowPPlane {
                node: 0,
                pplane: PPlane::XZ
            })
        );
        // Violate X: 0 not in Odd(f(0))
        assert_eq!(
            check_definition(
                &map! { 0: set!{0} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: PPlane::X },
            ),
            Err(InconsistentFlowPPlane {
                node: 0,
                pplane: PPlane::X
            })
        );
        // Violate Z: 0 not in f(0)
        assert_eq!(
            check_definition(
                &map! { 0: set!{1} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: PPlane::Z },
            ),
            Err(InconsistentFlowPPlane {
                node: 0,
                pplane: PPlane::Z
            })
        );
        // Violate Y: 0 in f(0) and 0 in Odd(f(0))
        assert_eq!(
            check_definition(
                &map! { 0: set!{0, 1} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: PPlane::Y },
            ),
            Err(InconsistentFlowPPlane {
                node: 0,
                pplane: PPlane::Y
            })
        );
    }

    #[test_log::test]
    fn test_find_case0() {
        let TestCase { g, iset, oset } = test_utils::CASE0.clone();
        let pplanes = map! {};
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), pplanes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(layer, vec![0, 0]);
        verify((f, layer), g, iset, oset, pplanes).unwrap();
    }

    #[test_log::test]
    fn test_find_case1() {
        let TestCase { g, iset, oset } = test_utils::CASE1.clone();
        let pplanes = map! {
            0: PPlane::XY,
            1: PPlane::XY,
            2: PPlane::XY,
            3: PPlane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), pplanes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([1]));
        assert_eq!(f[&1], Nodes::from([2]));
        assert_eq!(f[&2], Nodes::from([3]));
        assert_eq!(f[&3], Nodes::from([4]));
        assert_eq!(layer, vec![4, 3, 2, 1, 0]);
        verify((f, layer), g, iset, oset, pplanes).unwrap();
    }

    #[test_log::test]
    fn test_find_case2() {
        let TestCase { g, iset, oset } = test_utils::CASE2.clone();
        let pplanes = map! {
            0: PPlane::XY,
            1: PPlane::XY,
            2: PPlane::XY,
            3: PPlane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), pplanes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([2]));
        assert_eq!(f[&1], Nodes::from([3]));
        assert_eq!(f[&2], Nodes::from([4]));
        assert_eq!(f[&3], Nodes::from([5]));
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
        verify((f, layer), g, iset, oset, pplanes).unwrap();
    }

    #[test_log::test]
    fn test_find_case3() {
        let TestCase { g, iset, oset } = test_utils::CASE3.clone();
        let pplanes = map! {
            0: PPlane::XY,
            1: PPlane::XY,
            2: PPlane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), pplanes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([4, 5]));
        assert_eq!(f[&1], Nodes::from([3, 4, 5]));
        assert_eq!(f[&2], Nodes::from([3, 5]));
        assert_eq!(layer, vec![1, 1, 1, 0, 0, 0]);
        verify((f, layer), g, iset, oset, pplanes).unwrap();
    }

    #[test_log::test]
    fn test_find_case4() {
        let TestCase { g, iset, oset } = test_utils::CASE4.clone();
        let pplanes = map! {
            0: PPlane::XY,
            1: PPlane::XY,
            2: PPlane::XZ,
            3: PPlane::YZ
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), pplanes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([2]));
        assert_eq!(f[&1], Nodes::from([5]));
        assert_eq!(f[&2], Nodes::from([2, 4]));
        assert_eq!(f[&3], Nodes::from([3]));
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
        verify((f, layer), g, iset, oset, pplanes).unwrap();
    }

    #[test_log::test]
    fn test_find_case5() {
        let TestCase { g, iset, oset } = test_utils::CASE5.clone();
        let pplanes = map! {
            0: PPlane::XY,
            1: PPlane::XY
        };
        assert!(find(g, iset, oset, pplanes).is_none());
    }

    #[test_log::test]
    fn test_find_case6() {
        let TestCase { g, iset, oset } = test_utils::CASE6.clone();
        let pplanes = map! {
            0: PPlane::XY,
            1: PPlane::X,
            2: PPlane::XY,
            3: PPlane::X
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), pplanes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([1]));
        assert_eq!(f[&1], Nodes::from([4]));
        assert_eq!(f[&2], Nodes::from([3]));
        assert_eq!(f[&3], Nodes::from([2, 4]));
        assert_eq!(layer, vec![1, 1, 0, 1, 0]);
        verify((f, layer), g, iset, oset, pplanes).unwrap();
    }

    #[test_log::test]
    fn test_find_case7() {
        let TestCase { g, iset, oset } = test_utils::CASE7.clone();
        let pplanes = map! {
            0: PPlane::Z,
            1: PPlane::Z,
            2: PPlane::Y,
            3: PPlane::Y
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), pplanes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        // Graphix
        // assert_eq!(f[&0], Nodes::from([0, 1]));
        assert_eq!(f[&0], Nodes::from([0]));
        assert_eq!(f[&1], Nodes::from([1]));
        assert_eq!(f[&2], Nodes::from([2]));
        assert_eq!(f[&3], Nodes::from([4]));
        assert_eq!(layer, vec![1, 0, 0, 1, 0]);
        verify((f, layer), g, iset, oset, pplanes).unwrap();
    }

    #[test_log::test]
    fn test_find_case8() {
        let TestCase { g, iset, oset } = test_utils::CASE8.clone();
        let pplanes = map! {
            0: PPlane::Z,
            1: PPlane::XZ,
            2: PPlane::Y
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), pplanes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        // Graphix
        // assert_eq!(f[&0], Nodes::from([0, 3, 4]));
        assert_eq!(f[&0], Nodes::from([0, 2, 4]));
        assert_eq!(f[&1], Nodes::from([1, 2]));
        assert_eq!(f[&2], Nodes::from([4]));
        assert_eq!(layer, vec![1, 1, 1, 0, 0]);
        verify((f, layer), g, iset, oset, pplanes).unwrap();
    }
}
