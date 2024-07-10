//! Maximally-delayed causal flow algorithm.

use hashbrown;
use pyo3::prelude::*;

use crate::{
    common::{self, Graph, InPlaceSetOp, Layer, Nodes},
    validate,
};

type Flow = hashbrown::HashMap<usize, usize>;

/// Checks if the properties of the causal flow are satisfied.
fn check_definition(f: &Flow, layer: &Layer, g: &Graph) -> anyhow::Result<()> {
    for (&i, &fi) in f.iter() {
        if layer[i] <= layer[fi] {
            let err = anyhow::anyhow!("layer check failed").context(format!("must be {i} -> {fi}"));
            return Err(err);
        }
        for &j in &g[fi] {
            if i != j && layer[i] <= layer[j] {
                let err = anyhow::anyhow!("layer check failed")
                    .context(format!("neither {i} == {j} nor {i} -> {j}"));
                return Err(err);
            }
        }
        if !(g[fi].contains(&i) && g[i].contains(&fi)) {
            let err = anyhow::anyhow!("graph check failed")
                .context(format!("{i} and {fi} not connected"));
            return Err(err);
        }
    }
    Ok(())
}

/// Finds the maximally-delayed causal flow, if any.
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
pub fn find(g: Graph, iset: Nodes, mut oset: Nodes) -> Option<(Flow, Layer)> {
    log::debug!("flow::find");
    validate::check_graph(&g, &iset, &oset).unwrap();
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
    // Reuse working memory
    let mut oset_work = Nodes::new();
    let mut cset_work = Nodes::new();
    for l in 1_usize.. {
        log::debug!("=====layer {l}=====");
        oset_work.clear();
        cset_work.clear();
        for &v in &cset {
            let checkv = &check[v];
            if checkv.len() != 1 {
                continue;
            }
            // Get the only element
            let u = *checkv.iter().next().expect("one element here");
            log::debug!("f({u}) = {v}");
            f.insert(u, v);
            // MEMO: Typo in Mahlla's PP (2007)
            log::debug!("layer({u}) = {l}");
            layer[u] = l;
            oset_work.insert(u);
            cset_work.insert(v);
        }
        if oset_work.is_empty() {
            break;
        }
        // For all u check[u] -= oset_work
        for &v in &oset_work {
            // check[u] contains v only if v in g[u]
            //  => u must be in g[v]
            g[v].iter().for_each(|&u| {
                check[u].remove(&v);
            });
        }
        oset.union_with(&oset_work);
        cset.difference_with(&cset_work);
        cset.union_with(oset_work.intersection(&icset));
    }
    if oset == vset {
        log::debug!("flow found");
        log::debug!("flow : {f:?}");
        log::debug!("layer: {layer:?}");
        // TODO: Uncomment once ready
        // if cfg!(debug_assertions) {
        common::check_domain(f.iter(), &vset, &iset, &oset_orig).unwrap();
        common::check_initial(&layer, &oset_orig, true).unwrap();
        check_definition(&f, &layer, &g).unwrap();
        // }
        Some((f, layer))
    } else {
        log::debug!("flow not found");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{self, TestCase};
    use test_log;

    #[test_log::test]
    fn test_find_case0() {
        let TestCase { g, iset, oset } = test_utils::CASE0.get_or_init(test_utils::case0).clone();
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(layer, vec![0, 0]);
    }

    #[test_log::test]
    fn test_find_case1() {
        let TestCase { g, iset, oset } = test_utils::CASE1.get_or_init(test_utils::case1).clone();
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], 1);
        assert_eq!(f[&1], 2);
        assert_eq!(f[&2], 3);
        assert_eq!(f[&3], 4);
        assert_eq!(layer, vec![4, 3, 2, 1, 0]);
    }

    #[test_log::test]
    fn test_find_case2() {
        let TestCase { g, iset, oset } = test_utils::CASE2.get_or_init(test_utils::case2).clone();
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], 2);
        assert_eq!(f[&1], 3);
        assert_eq!(f[&2], 4);
        assert_eq!(f[&3], 5);
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
    }

    #[test_log::test]
    fn test_find_case3() {
        let TestCase { g, iset, oset } = test_utils::CASE3.get_or_init(test_utils::case3).clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case4() {
        let TestCase { g, iset, oset } = test_utils::CASE4.get_or_init(test_utils::case4).clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case5() {
        let TestCase { g, iset, oset } = test_utils::CASE5.get_or_init(test_utils::case5).clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case6() {
        let TestCase { g, iset, oset } = test_utils::CASE6.get_or_init(test_utils::case6).clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case7() {
        let TestCase { g, iset, oset } = test_utils::CASE7.get_or_init(test_utils::case7).clone();
        assert!(find(g, iset, oset).is_none());
    }

    #[test_log::test]
    fn test_find_case8() {
        let TestCase { g, iset, oset } = test_utils::CASE8.get_or_init(test_utils::case8).clone();
        assert!(find(g, iset, oset).is_none());
    }
}
