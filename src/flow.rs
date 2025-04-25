//! Maximally-delayed causal flow algorithm.

use hashbrown;
use pyo3::prelude::*;

use crate::{
    common::{
        FlowValidationError::{self, InconsistentFlowOrder},
        Graph, Layer, Nodes, FATAL_MSG,
    },
    internal::{utils::InPlaceSetDiff, validate},
};

type Flow = hashbrown::HashMap<usize, usize>;

/// Checks the definition of causal flow.
///
/// 1. i -> f(i)
/// 2. j in neighbors(f(i)) => i == j or i -> j
/// 3. i in neighbors(f(i))
fn check_definition(f: &Flow, layer: &Layer, g: &Graph) -> Result<(), FlowValidationError> {
    for (&i, &fi) in f {
        if layer[i] <= layer[fi] {
            Err(InconsistentFlowOrder { nodes: (i, fi) })?;
        }
        for &j in &g[fi] {
            if i != j && layer[i] <= layer[j] {
                Err(InconsistentFlowOrder { nodes: (i, j) })?;
            }
        }
        if !(g[fi].contains(&i) && g[i].contains(&fi)) {
            Err(InconsistentFlowOrder { nodes: (i, fi) })?;
        }
    }
    Ok(())
}

/// Finds the maximally-delayed causal flow.
///
/// # Arguments
///
/// - `g`: The adjacency list of the graph. Must be undirected and without self-loops.
/// - `iset`: The set of initial nodes.
/// - `oset`: The set of output nodes.
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
pub fn find(g: Graph, iset: Nodes, mut oset: Nodes) -> Option<(Flow, Layer)> {
    let n = g.len();
    let vset = (0..n).collect::<Nodes>();
    let mut cset = &oset - &iset;
    let icset = &vset - &iset;
    let ocset = &vset - &oset;
    let oset_orig = oset.clone();
    let mut f = Flow::with_capacity(ocset.len());
    let mut layer = vec![0_usize; n];
    // check[v] = g[v] & (vset - oset)
    let mut check = g.iter().map(|x| x & &ocset).collect::<Vec<_>>();
    let mut oset_work = Nodes::new();
    let mut cset_work = Nodes::new();
    for l in 1_usize.. {
        tracing::debug!("=====layer {l}=====");
        oset_work.clear();
        cset_work.clear();
        for &v in &cset {
            let checkv = &check[v];
            if checkv.len() != 1 {
                continue;
            }
            let u = *checkv.iter().next().expect("one element here");
            tracing::debug!("f({u}) = {v}");
            f.insert(u, v);
            tracing::debug!("layer({u}) = {l}");
            layer[u] = l;
            oset_work.insert(u);
            cset_work.insert(v);
        }
        if oset_work.is_empty() {
            break;
        }
        // For all u check[u] -= oset_work
        for &v in &oset_work {
            g[v].iter().for_each(|&u| {
                check[u].remove(&v);
            });
        }
        oset.extend(&oset_work);
        cset.difference_with(&cset_work);
        cset.extend(oset_work.intersection(&icset));
    }
    if oset == vset {
        tracing::debug!("flow found");
        tracing::debug!("flow : {f:?}");
        tracing::debug!("layer: {layer:?}");
        // TODO: Remove this block once stabilized
        {
            validate::check_domain(f.iter(), &vset, &iset, &oset_orig).expect(FATAL_MSG);
            validate::check_initial(&layer, &oset_orig, true).expect(FATAL_MSG);
            check_definition(&f, &layer, &g).expect(FATAL_MSG);
        }
        Some((f, layer))
    } else {
        tracing::debug!("flow not found");
        None
    }
}

/// Validates flow.
///
/// # Errors
///
/// - If `flow` is invalid.
/// - If `flow` is inconsistent with `g`.
#[pyfunction]
#[allow(clippy::needless_pass_by_value)]
pub fn verify(flow: (Flow, Layer), g: Graph, iset: Nodes, oset: Nodes) -> PyResult<()> {
    let (f, layer) = flow;
    let n = g.len();
    let vset = (0..n).collect::<Nodes>();
    validate::check_domain(f.iter(), &vset, &iset, &oset)?;
    validate::check_initial(&layer, &oset, true)?;
    check_definition(&f, &layer, &g)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use test_log;

    use super::*;
    use crate::internal::test_utils::{self, TestCase};

    #[test_log::test]
    fn test_check_definition_ng() {
        // Violate 0 -> f(0) = 1
        assert_eq!(
            check_definition(&map! { 0: 1 }, &vec![0, 0], &test_utils::graph(&[(0, 1)])),
            Err(InconsistentFlowOrder { nodes: (0, 1) })
        );
        // Violate 1 in nb(f(0)) = nb(2) => 0 == 1 or 0 -> 1
        assert_eq!(
            check_definition(
                &map! { 0: 2 },
                &vec![1, 1, 0],
                &test_utils::graph(&[(0, 1), (1, 2)])
            ),
            Err(InconsistentFlowOrder { nodes: (0, 1) })
        );
        // Violate 0 in nb(f(0)) = nb(2)
        assert_eq!(
            check_definition(
                &map! { 0: 2 },
                &vec![2, 1, 0],
                &test_utils::graph(&[(0, 1), (1, 2)])
            ),
            Err(InconsistentFlowOrder { nodes: (0, 2) })
        );
    }

    #[test_log::test]
    fn test_find_case0() {
        let TestCase { g, iset, oset } = test_utils::CASE0.clone();
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(layer, vec![0, 0]);
        verify((f, layer), g, iset, oset).unwrap();
    }

    #[test_log::test]
    fn test_find_case1() {
        let TestCase { g, iset, oset } = test_utils::CASE1.clone();
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], 1);
        assert_eq!(f[&1], 2);
        assert_eq!(f[&2], 3);
        assert_eq!(f[&3], 4);
        assert_eq!(layer, vec![4, 3, 2, 1, 0]);
        verify((f, layer), g, iset, oset).unwrap();
    }

    #[test_log::test]
    fn test_find_case2() {
        let TestCase { g, iset, oset } = test_utils::CASE2.clone();
        let flen = g.len() - oset.len();
        let (f, layer) = find(g.clone(), iset.clone(), oset.clone()).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], 2);
        assert_eq!(f[&1], 3);
        assert_eq!(f[&2], 4);
        assert_eq!(f[&3], 5);
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
        verify((f, layer), g, iset, oset).unwrap();
    }

    #[test_log::test]
    fn test_find_case3() {
        let TestCase { g, iset, oset } = test_utils::CASE3.clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case4() {
        let TestCase { g, iset, oset } = test_utils::CASE4.clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case5() {
        let TestCase { g, iset, oset } = test_utils::CASE5.clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case6() {
        let TestCase { g, iset, oset } = test_utils::CASE6.clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case7() {
        let TestCase { g, iset, oset } = test_utils::CASE7.clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case8() {
        let TestCase { g, iset, oset } = test_utils::CASE8.clone();
        assert!(find(g, iset, oset).is_none());
    }
}
