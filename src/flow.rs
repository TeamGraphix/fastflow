use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::common::{self, Graph, InPlaceSetOp, Layer};

type Flow = HashMap<usize, usize>;

#[cfg(debug_assertions)]
fn check_domain(
    f: &Flow,
    vset: &HashSet<usize>,
    iset: &HashSet<usize>,
    oset: &HashSet<usize>,
) -> anyhow::Result<()> {
    let icset = vset - iset;
    let ocset = vset - oset;
    for (&i, &fi) in f.iter() {
        if !ocset.contains(&i) {
            let err = anyhow::anyhow!("domain check failed").context(format!("{i} not in V\\O"));
            return Err(err);
        }
        if !icset.contains(&fi) {
            let err = anyhow::anyhow!("domain check failed").context(format!("{fi} not in V\\I"));
            return Err(err);
        }
    }
    Ok(())
}

#[cfg(debug_assertions)]
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

#[pyfunction]
pub fn find(g: Graph, iset: HashSet<usize>, mut oset: HashSet<usize>) -> Option<(Flow, Layer)> {
    let n = g.len();
    let vset = (0..n).collect::<HashSet<_>>();
    let mut cset = &oset - &iset;
    let icset = &vset - &iset;
    let ocset = &vset - &oset;
    let oset_orig = oset.clone();
    let mut f = HashMap::with_capacity(ocset.len());
    let mut layer = vec![0_usize; n];
    // check[v] = g[v] & (vset - oset)
    let mut check = g.iter().map(|x| x & &ocset).collect::<Vec<_>>();
    // Reuse working memory
    let mut oset_work = HashSet::new();
    let mut cset_work = HashSet::new();
    for l in 1_usize.. {
        oset_work.clear();
        cset_work.clear();
        for &v in &cset {
            if check[v].len() != 1 {
                continue;
            }
            // Get the only element
            let u = *check[v].iter().next().expect("one element here");
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
        oset.union_with(oset_work.iter());
        cset.difference_with(cset_work.iter());
        cset.union_with(oset_work.intersection(&icset));
    }
    debug_assert_eq!(check_domain(&f, &vset, &iset, &oset_orig).unwrap(), ());
    if oset == vset {
        debug_assert_eq!(common::check_initial(&layer, &oset_orig).unwrap(), ());
        debug_assert_eq!(check_definition(&f, &layer, &g).unwrap(), ());
        Some((f, layer))
    } else {
        None
    }
}
