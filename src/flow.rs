use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::common::{Flow, Layer};

#[pyfunction]
pub fn find(
    g: Vec<HashSet<usize>>,
    iset: HashSet<usize>,
    mut oset: HashSet<usize>,
) -> Option<(Flow, Layer)> {
    let n = g.len();
    let mut f = HashMap::new();
    let mut layer = vec![0_usize; n];
    let vset = (0..n).collect::<HashSet<_>>();
    let mut cset = oset.difference(&iset).copied().collect::<HashSet<_>>();
    let icset = vset.difference(&iset).copied().collect::<HashSet<_>>();
    let ocset = vset.difference(&oset).copied().collect::<HashSet<_>>();
    // check[v] = g[v] & (vset - oset)
    let mut check = g
        .iter()
        .map(|x| x.intersection(&ocset).copied().collect::<HashSet<_>>())
        .collect::<Vec<_>>();
    let mut oset_work = HashSet::new();
    let mut cset_work = HashSet::new();
    f.reserve(ocset.len());
    for l in 1_usize.. {
        oset_work.clear();
        cset_work.clear();
        for &v in &cset {
            if check[v].len() != 1 {
                continue;
            }
            // Get the only element
            let u = *check[v].iter().next().expect("One element here.");
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
        // MEMO: May be refactored later using trait
        // oset |= oset_work
        oset.extend(oset_work.iter().copied());
        // cset -= cset_work
        cset_work.iter().for_each(|&v| {
            cset.remove(&v);
        });
        // cset |= oset_work & icset
        cset.extend(oset_work.intersection(&icset).copied());
    }
    if oset == vset {
        Some((f, layer))
    } else {
        None
    }
}
