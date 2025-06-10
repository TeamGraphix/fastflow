//! Maximally-delayed generalized flow algorithm.

use std::iter;

use fixedbitset::FixedBitSet;
use hashbrown;
use pyo3::prelude::*;

use crate::{
    common::{
        FlowValidationError::{
            self, InconsistentFlowOrder, InconsistentFlowPlane, InvalidMeasurementSpec,
        },
        Graph, Layer, Nodes, OrderedNodes, FATAL_MSG,
    },
    internal::{
        gf2_linalg::GF2Solver,
        utils::{self, InPlaceSetDiff},
        validate,
    },
};

#[pyclass(eq, hash, frozen)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// Enum-like class for measurement planes.
pub enum Plane {
    XY,
    YZ,
    XZ,
}

type Planes = hashbrown::HashMap<usize, Plane>;
type GFlow = hashbrown::HashMap<usize, Nodes>;

/// Checks the definition of gflow.
///
/// 1. i -> g(i)
/// 2. j in Odd(g(i)) => i == j or i -> j
/// 3. i not in g(i) and in Odd(g(i)) if plane(i) == XY
/// 4. i in g(i) and in Odd(g(i)) if plane(i) == YZ
/// 5. i in g(i) and not in Odd(g(i)) if plane(i) == XZ
fn check_definition(
    f: &GFlow,
    layer: &Layer,
    g: &Graph,
    planes: &Planes,
) -> Result<(), FlowValidationError> {
    for &i in itertools::chain(f.keys(), planes.keys()) {
        if f.contains_key(&i) != planes.contains_key(&i) {
            Err(InvalidMeasurementSpec { node: i })?;
        }
    }
    for (&i, fi) in f {
        let pi = planes[&i];
        for &fij in fi {
            if i != fij && layer[i] <= layer[fij] {
                Err(InconsistentFlowOrder { nodes: (i, fij) })?;
            }
        }
        let odd_fi = utils::odd_neighbors(g, fi);
        for &j in &odd_fi {
            if i != j && layer[i] <= layer[j] {
                Err(InconsistentFlowOrder { nodes: (i, j) })?;
            }
        }
        let in_info = (fi.contains(&i), odd_fi.contains(&i));
        match pi {
            Plane::XY if in_info != (false, true) => {
                Err(InconsistentFlowPlane {
                    node: i,
                    plane: Plane::XY,
                })?;
            }
            Plane::YZ if in_info != (true, false) => {
                Err(InconsistentFlowPlane {
                    node: i,
                    plane: Plane::YZ,
                })?;
            }
            Plane::XZ if in_info != (true, true) => {
                Err(InconsistentFlowPlane {
                    node: i,
                    plane: Plane::XZ,
                })?;
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
    planes: &Planes,
    ocset: &OrderedNodes,
    omiset: &OrderedNodes,
) {
    let ncols = omiset.len();
    // Set-to-index maps
    let oc2i = utils::indexmap::<hashbrown::HashMap<_, _>>(ocset);
    let omi2i = utils::indexmap::<hashbrown::HashMap<_, _>>(omiset);
    // Encode node as one-hot vector
    for (i, &u) in ocset.iter().enumerate() {
        let gu = &g[u];
        // Initialize adjacency matrix
        let r = i;
        for &v in gu {
            if let Some(&c) = omi2i.get(&v) {
                work[r].insert(c);
            }
        }
        // Initialize rhs
        let ieq = i;
        let c = ncols + ieq;
        if let Plane::XY | Plane::XZ = planes[&u] {
            // = u
            work[ieq].insert(c);
        }
        if planes[&u] == Plane::XY {
            continue;
        }
        // Include u
        for &v in gu {
            if let Some(&r) = oc2i.get(&v) {
                work[r].toggle(c);
            }
        }
    }
}

/// Finds the maximally-delayed generalized flow.
///
/// # Arguments
///
/// - `g`: The adjacency list of the graph. Must be undirected and without self-loops.
/// - `iset`: The set of initial nodes.
/// - `oset`: The set of output nodes.
/// - `planes`: Measurement plane of each node in V\O.
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
pub fn find(g: Graph, iset: Nodes, oset: Nodes, planes: Planes) -> Option<(GFlow, Layer)> {
    let n = g.len();
    let vset = (0..n).collect::<Nodes>();
    let mut cset = Nodes::new();
    // Need to use BTreeSet to get deterministic order
    let mut ocset = vset.difference(&oset).copied().collect::<OrderedNodes>();
    let mut omiset = oset.difference(&iset).copied().collect::<OrderedNodes>();
    let mut f = GFlow::with_capacity(ocset.len());
    let mut layer = vec![0_usize; n];
    let mut nrows = ocset.len();
    let mut ncols = omiset.len();
    let mut neqs = ocset.len();
    let mut work = vec![FixedBitSet::with_capacity(ncols + neqs); nrows];
    for l in 1_usize.. {
        cset.clear();
        if ocset.is_empty() || omiset.is_empty() {
            break;
        }
        tracing::debug!("=====layer {l}=====");
        // Decrease over time
        nrows = ocset.len();
        ncols = omiset.len();
        neqs = ocset.len();
        debug_assert!(work.len() >= nrows);
        work.truncate(nrows);
        // Decrease over time
        debug_assert!(work[0].len() >= ncols + neqs);
        utils::zerofill(&mut work, ncols + neqs);
        tracing::debug!("rowset: {ocset:?}");
        tracing::debug!("colset: {omiset:?}");
        tracing::debug!("eqset : {ocset:?}");
        tracing::debug!(
            "planes: {:?}",
            ocset.iter().map(|&u| planes[&u]).collect::<Vec<_>>()
        );
        init_work(&mut work, &g, &planes, &ocset, &omiset);
        let mut solver = GF2Solver::attach(work.as_mut_slice(), neqs);
        let mut x = FixedBitSet::with_capacity(ncols);
        tracing::debug!("{solver:?}");
        for (ieq, &u) in ocset.iter().enumerate() {
            if !solver.solve_in_place(&mut x, ieq) {
                tracing::debug!("solution not found: {u}");
                continue;
            }
            cset.insert(u);
            // Decode solution
            let mut fu = omiset
                .iter()
                .enumerate()
                .filter_map(|(i, &v)| if x[i] { Some(v) } else { None })
                .collect::<Nodes>();
            if let Plane::YZ | Plane::XZ = planes[&u] {
                // Include u
                fu.insert(u);
            }
            tracing::debug!("f({u}) = {fu:?}");
            f.insert(u, fu);
            tracing::debug!("layer({u}) = {l}");
            layer[u] = l;
        }
        if cset.is_empty() {
            break;
        }
        ocset.difference_with(&cset);
        omiset.extend(cset.difference(&iset));
    }
    if ocset.is_empty() {
        tracing::debug!("gflow found");
        tracing::debug!("gflow: {f:?}");
        tracing::debug!("layer: {layer:?}");
        // TODO: Remove this block once stabilized
        {
            let f_flatiter = f
                .iter()
                .flat_map(|(i, fi)| Iterator::zip(iter::repeat(i), fi.iter()));
            validate::check_domain(f_flatiter, &vset, &iset, &oset).expect(FATAL_MSG);
            validate::check_initial(&layer, &oset, true).expect(FATAL_MSG);
            check_definition(&f, &layer, &g, &planes).expect(FATAL_MSG);
        }
        Some((f, layer))
    } else {
        tracing::debug!("gflow not found");
        None
    }
}

/// Validates generalized flow.
///
/// # Errors
///
/// - If `gflow` is invalid.
/// - If `gflow` is inconsistent with `g`.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
pub fn verify(
    gflow: (GFlow, Layer),
    g: Graph,
    iset: Nodes,
    oset: Nodes,
    planes: Planes,
) -> PyResult<()> {
    let (f, layer) = gflow;
    let n = g.len();
    let vset = (0..n).collect::<Nodes>();
    let f_flatiter = f
        .iter()
        .flat_map(|(i, fi)| Iterator::zip(iter::repeat(i), fi.iter()));
    validate::check_domain(f_flatiter, &vset, &iset, &oset)?;
    validate::check_initial(&layer, &oset, true)?;
    check_definition(&f, &layer, &g, &planes)?;
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
                &map! { 0: Plane::XY },
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
                    0: Plane::XY,
                    1: Plane::XY
                },
            ),
            Err(InconsistentFlowOrder { nodes: (0, 1) })
        );
        // Violate XY: 0 in f(0)
        assert_eq!(
            check_definition(
                &map! { 0: set!{0} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: Plane::XY },
            ),
            Err(InconsistentFlowPlane {
                node: 0,
                plane: Plane::XY
            })
        );
        // Violate YZ: 0 in Odd(f(0))
        assert_eq!(
            check_definition(
                &map! { 0: set!{1} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: Plane::YZ },
            ),
            Err(InconsistentFlowPlane {
                node: 0,
                plane: Plane::YZ
            })
        );
        // Violate XZ: 0 not in Odd(f(0)) and in f(0)
        assert_eq!(
            check_definition(
                &map! { 0: set!{0} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: Plane::XZ },
            ),
            Err(InconsistentFlowPlane {
                node: 0,
                plane: Plane::XZ
            })
        );
        // Violate XZ: 0 in Odd(f(0)) and not in f(0)
        assert_eq!(
            check_definition(
                &map! { 0: set!{1} },
                &vec![1, 0],
                &test_utils::graph(&[(0, 1)]),
                &map! { 0: Plane::XZ },
            ),
            Err(InconsistentFlowPlane {
                node: 0,
                plane: Plane::XZ
            })
        );
    }

    #[test_log::test]
    fn test_find_case0() {
        let TestCase { g, iset, oset } = test_utils::CASE0.clone();
        let planes = map! {};
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), planes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(layer, vec![0, 0]);
        verify((f, layer), g, iset, oset, planes).unwrap();
    }

    #[test_log::test]
    fn test_find_case1() {
        let TestCase { g, iset, oset } = test_utils::CASE1.clone();
        let planes = map! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XY,
            3: Plane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), planes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([1]));
        assert_eq!(f[&1], Nodes::from([2]));
        assert_eq!(f[&2], Nodes::from([3]));
        assert_eq!(f[&3], Nodes::from([4]));
        assert_eq!(layer, vec![4, 3, 2, 1, 0]);
        verify((f, layer), g, iset, oset, planes).unwrap();
    }

    #[test_log::test]
    fn test_find_case2() {
        let TestCase { g, iset, oset } = test_utils::CASE2.clone();
        let planes = map! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XY,
            3: Plane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), planes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([2]));
        assert_eq!(f[&1], Nodes::from([3]));
        assert_eq!(f[&2], Nodes::from([4]));
        assert_eq!(f[&3], Nodes::from([5]));
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
        verify((f, layer), g, iset, oset, planes).unwrap();
    }

    #[test_log::test]
    fn test_find_case3() {
        let TestCase { g, iset, oset } = test_utils::CASE3.clone();
        let planes = map! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), planes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([4, 5]));
        assert_eq!(f[&1], Nodes::from([3, 4, 5]));
        assert_eq!(f[&2], Nodes::from([3, 5]));
        assert_eq!(layer, vec![1, 1, 1, 0, 0, 0]);
        verify((f, layer), g, iset, oset, planes).unwrap();
    }

    #[test_log::test]
    fn test_find_case4() {
        let TestCase { g, iset, oset } = test_utils::CASE4.clone();
        let planes = map! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XZ,
            3: Plane::YZ
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone(), planes.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], Nodes::from([2]));
        assert_eq!(f[&1], Nodes::from([5]));
        assert_eq!(f[&2], Nodes::from([2, 4]));
        assert_eq!(f[&3], Nodes::from([3]));
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
        verify((f, layer), g, iset, oset, planes).unwrap();
    }

    #[test_log::test]
    fn test_find_case5() {
        let TestCase { g, iset, oset } = test_utils::CASE5.clone();
        let planes = map! {
            0: Plane::XY,
            1: Plane::XY
        };
        assert!(find(g, iset, oset, planes).is_none());
    }

    #[test_log::test]
    fn test_find_case6() {
        let TestCase { g, iset, oset } = test_utils::CASE6.clone();
        let planes = map! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XY,
            3: Plane::XY
        };
        assert!(find(g, iset, oset, planes).is_none());
    }

    #[test_log::test]
    fn test_find_case7() {
        let TestCase { g, iset, oset } = test_utils::CASE7.clone();
        let planes = map! {
            0: Plane::YZ,
            1: Plane::XZ,
            2: Plane::XY,
            3: Plane::YZ
        };
        assert!(find(g, iset, oset, planes).is_none());
    }

    #[test_log::test]
    fn test_find_case8() {
        let TestCase { g, iset, oset } = test_utils::CASE8.clone();
        let planes = map! {
            0: Plane::YZ,
            1: Plane::XZ,
            2: Plane::XY
        };
        assert!(find(g, iset, oset, planes).is_none());
    }
}
