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
        oset_work.clear();
        cset_work.clear();
        for &v in &cset {
            let checkv = &check[v];
            if checkv.len() != 1 {
                continue;
            }
            // Get the only element
            let u = *checkv.iter().next().expect("one element here");
            f.insert(u, v);
            // MEMO: Typo in Mahlla's PP (2007)
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
        // TODO: Uncomment once ready
        // if cfg!(debug_assertions) {
        common::check_domain(f.iter(), &vset, &iset, &oset_orig).unwrap();
        common::check_initial(&layer, &oset_orig).unwrap();
        check_definition(&f, &layer, &g).unwrap();
        // }
        Some((f, layer))
    } else {
        None
    }
}
