use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::common::{Flow, InPlaceSetOp, Layer};

#[pyfunction]
pub fn find(
    g: Vec<HashSet<usize>>,
    iset: HashSet<usize>,
    mut oset: HashSet<usize>,
) -> Option<(Flow, Layer)> {
    let n = g.len();
    let vset = (0..n).collect::<HashSet<_>>();
    let mut cset = &oset - &iset;
    let icset = &vset - &iset;
    let ocset = &vset - &oset;
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
    if oset == vset {
        Some((f, layer))
    } else {
        None
    }
}
