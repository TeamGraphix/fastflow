//! Maximally-delayed generalized flow algorithm.

use crate::common::{self, Nodes, OrderedNodes};
use crate::{
    common::{Graph, InPlaceSetOp, Layer},
    gf2_linalg::GF2Solver,
};
use crate::{gf2_linalg, validate};
use fixedbitset::FixedBitSet;
use hashbrown;
use log::Level;
use num_derive::FromPrimitive;
use num_enum::IntoPrimitive;
use num_traits::cast::FromPrimitive;
use pyo3::prelude::*;
use std::iter;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
/// Measurement plane.
enum Plane {
    XY = 0,
    YZ = 1,
    ZX = 2,
}

type InternalPlanes = hashbrown::HashMap<usize, u8>;
type Planes = hashbrown::HashMap<usize, Plane>;
type GFlow = hashbrown::HashMap<usize, Nodes>;

/// Checks the definition of gflow.
///
/// 1. i -> g(i)
/// 2. j in odd_neighbors(g(i)) => i == j or i -> j
/// 3. i not in g(i) and in odd_neighbors(g(i)) if plane(i) == XY
/// 4. i in g(i) and in odd_neighbors(g(i)) if plane(i) == YZ
/// 5. i in g(i) and not in odd_neighbors(g(i)) if plane(i) == ZX
fn check_definition(f: &GFlow, layer: &Layer, g: &Graph, planes: &Planes) -> anyhow::Result<()> {
    anyhow::ensure!(
        f.len() == planes.len(),
        "f and planes must have the same codomain"
    );
    for &i in f.keys() {
        let fi = &f[&i];
        let pi = planes[&i];
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
                let err = anyhow::anyhow!("plane check failed").context(format!(
                    "must satisfy ({i} in f({i}), {i} in Odd(f({i})) = (false, true): XY"
                ));
                return Err(err);
            }
            Plane::YZ if in_info != (true, false) => {
                let err = anyhow::anyhow!("plane check failed").context(format!(
                    "must satisfy ({i} in f({i}), {i} in Odd(f({i})) = (true, false): YZ"
                ));
                return Err(err);
            }
            Plane::ZX if in_info != (true, true) => {
                let err = anyhow::anyhow!("plane check failed").context(format!(
                    "must satisfy ({i} in f({i}), {i} in Odd(f({i})) = (true, true): ZX"
                ));
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
    planes: &Planes,
    ocset: &OrderedNodes,
    omiset: &OrderedNodes,
) {
    let ncols = omiset.len();
    // Set-to-index maps
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
        if let Plane::XY | Plane::ZX = planes[&u] {
            // = u
            work[ieq].insert(c);
        }
        if let Plane::XY = planes[&u] {
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

/// Finds the maximally-delayed generalized flow.
///
/// # Arguments
///
/// - `g`: The adjacency list of the graph. Must be undirected and without self-loops.
/// - `iset`: The set of initial nodes.
/// - `oset`: The set of output nodes.
/// - `planes`: Measurement plane of each node in V\O.
///   - `0`: XY
///   - `1`: YZ
///   - `2`: ZX
///
/// # Note
///
/// - Node indices are assumed to be `0..g.len()`.
/// - Arguments are **NOT** verified.
#[pyfunction]
pub fn find(g: Graph, iset: Nodes, oset: Nodes, planes: InternalPlanes) -> Option<(GFlow, Layer)> {
    log::debug!("gflow::find");
    validate::check_graph(&g, &iset, &oset).unwrap();
    let planes = planes
        .into_iter()
        .map(|(k, v)| (k, Plane::from_u8(v).expect("plane is either 0, 1, or 2")))
        .collect::<Planes>();
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
    let mut i2v = Vec::new();
    for l in 1_usize.. {
        cset.clear();
        if ocset.is_empty() || omiset.is_empty() {
            break;
        }
        log::debug!("=====layer {l}=====");
        // Decrease over time
        nrows = ocset.len();
        ncols = omiset.len();
        neqs = ocset.len();
        debug_assert!(work.len() >= nrows);
        work.truncate(nrows);
        // Decrease over time
        debug_assert!(work[0].len() >= ncols + neqs);
        common::zerofill(&mut work, ncols + neqs);
        log::debug!("rowset: {ocset:?}");
        log::debug!("colset: {omiset:?}");
        log::debug!("eqset : {ocset:?}");
        log::debug!(
            "planes: {:?}",
            ocset.iter().map(|&u| planes[&u]).collect::<Vec<_>>()
        );
        init_work(&mut work, &g, &planes, &ocset, &omiset);
        if log::log_enabled!(Level::Debug) {
            log::debug!("work:");
            for row in gf2_linalg::format_work(&work, omiset.len()) {
                log::debug!("  {}", row);
            }
        }
        let mut solver = GF2Solver::attach(work, neqs);
        let mut x = FixedBitSet::with_capacity(ncols);
        // i2v[i] = node assigned to one-hot vector x[i]
        i2v.clear();
        i2v.extend(omiset.iter().copied());
        for (ieq, &u) in ocset.iter().enumerate() {
            if !solver.solve_in_place(&mut x, ieq) {
                log::debug!("solution not found: {u}");
                continue;
            }
            cset.insert(u);
            // Decode solution
            let mut fu = x.ones().map(|c| i2v[c]).collect::<Nodes>();
            if let Plane::YZ | Plane::ZX = planes[&u] {
                // Include u
                fu.insert(u);
            }
            log::debug!("f({u}) = {fu:?}");
            f.insert(u, fu);
            log::debug!("layer({u}) = {l}");
            layer[u] = l;
        }
        if cset.is_empty() {
            break;
        }
        ocset.difference_with(&cset);
        omiset.union_with(cset.difference(&iset));
        work = solver.detach();
    }
    if ocset.is_empty() {
        log::debug!("gflow found");
        log::debug!("gflow: {f:?}");
        log::debug!("layer: {layer:?}");
        // TODO: Uncomment once ready
        // if cfg!(debug_assertions) {
        let f_flatiter = f
            .iter()
            .flat_map(|(i, fi)| Iterator::zip(iter::repeat(i), fi.iter()));
        common::check_domain(f_flatiter, &vset, &iset, &oset).unwrap();
        common::check_initial(&layer, &oset, true).unwrap();
        check_definition(&f, &layer, &g, &planes).unwrap();
        // }
        Some((f, layer))
    } else {
        log::debug!("gflow not found");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodeset;
    use crate::test_utils::{self, TestCase};
    use test_log;

    macro_rules! planes {
    ($($u:literal: $v:expr),*) => {
        ::hashbrown::HashMap::from_iter([$(($u, ($v).into())),*].iter().copied())
    };
}

    #[test_log::test]
    fn test_find_case0() {
        let TestCase { g, iset, oset } = test_utils::CASE0.get_or_init(test_utils::case0).clone();
        let planes = planes! {};
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, planes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(layer, vec![0, 0]);
    }

    #[test_log::test]
    fn test_find_case1() {
        let TestCase { g, iset, oset } = test_utils::CASE1.get_or_init(test_utils::case1).clone();
        let planes = planes! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XY,
            3: Plane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, planes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodeset![1]);
        assert_eq!(f[&1], nodeset![2]);
        assert_eq!(f[&2], nodeset![3]);
        assert_eq!(f[&3], nodeset![4]);
        assert_eq!(layer, vec![4, 3, 2, 1, 0]);
    }

    #[test_log::test]
    fn test_find_case2() {
        let TestCase { g, iset, oset } = test_utils::CASE2.get_or_init(test_utils::case2).clone();
        let planes = planes! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XY,
            3: Plane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, planes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodeset![2]);
        assert_eq!(f[&1], nodeset![3]);
        assert_eq!(f[&2], nodeset![4]);
        assert_eq!(f[&3], nodeset![5]);
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
    }

    #[test_log::test]
    fn test_find_case3() {
        let TestCase { g, iset, oset } = test_utils::CASE3.get_or_init(test_utils::case3).clone();
        let planes = planes! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XY
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, planes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodeset![4, 5]);
        assert_eq!(f[&1], nodeset![3, 4, 5]);
        assert_eq!(f[&2], nodeset![3, 5]);
        assert_eq!(layer, vec![1, 1, 1, 0, 0, 0]);
    }

    #[test_log::test]
    fn test_find_case4() {
        let TestCase { g, iset, oset } = test_utils::CASE4.get_or_init(test_utils::case4).clone();
        let planes = planes! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::ZX,
            3: Plane::YZ
        };
        let flen = g.len() - oset.len();
        let (f, layer) = find(g, iset, oset, planes).unwrap();
        assert_eq!(f.len(), flen);
        assert_eq!(f[&0], nodeset![2]);
        assert_eq!(f[&1], nodeset![5]);
        assert_eq!(f[&2], nodeset![2, 4]);
        assert_eq!(f[&3], nodeset![3]);
        assert_eq!(layer, vec![2, 2, 1, 1, 0, 0]);
    }

    #[test_log::test]
    fn test_find_case5() {
        let TestCase { g, iset, oset } = test_utils::CASE5.get_or_init(test_utils::case5).clone();
        let planes = planes! {
            0: Plane::XY,
            1: Plane::XY
        };
        assert!(find(g, iset, oset, planes).is_none());
    }

    #[test_log::test]
    fn test_find_case6() {
        let TestCase { g, iset, oset } = test_utils::CASE6.get_or_init(test_utils::case6).clone();
        let planes = planes! {
            0: Plane::XY,
            1: Plane::XY,
            2: Plane::XY,
            3: Plane::XY
        };
        assert!(find(g, iset, oset, planes).is_none());
    }

    #[test_log::test]
    fn test_find_case7() {
        let TestCase { g, iset, oset } = test_utils::CASE7.get_or_init(test_utils::case7).clone();
        let planes = planes! {
            0: Plane::YZ,
            1: Plane::ZX,
            2: Plane::XY,
            3: Plane::YZ
        };
        assert!(find(g, iset, oset, planes).is_none());
    }

    #[test_log::test]
    fn test_find_case8() {
        let TestCase { g, iset, oset } = test_utils::CASE8.get_or_init(test_utils::case8).clone();
        let planes = planes! {
            0: Plane::YZ,
            1: Plane::ZX,
            2: Plane::XY
        };
        assert!(find(g, iset, oset, planes).is_none());
    }
}
