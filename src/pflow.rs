//! Maximally-delayed Pauli flow algorithm.

use crate::common::InPlaceSetOp;
use crate::common::{self, Graph, Layer, Nodes, OrderedNodes};
use crate::gf2_linalg::GF2Solver;
use fixedbitset::FixedBitSet;
use hashbrown;
use num_derive::FromPrimitive;
use num_enum::IntoPrimitive;
use num_traits::cast::FromPrimitive;
use pyo3::prelude::*;
use std::iter;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum PPlane {
    XY = 0,
    YZ = 1,
    ZX = 2,
    X = 3,
    Y = 4,
    Z = 5,
}

// Introduced only for internal use
type InternalPPlanes = hashbrown::HashMap<usize, u8>;
type PPlanes = hashbrown::HashMap<usize, PPlane>;
type PFlow = hashbrown::HashMap<usize, Nodes>;

fn check_initial_pflow(layer: &Layer, oset: &Nodes) -> anyhow::Result<()> {
    for &u in oset {
        if layer[u] != 0 {
            let err = anyhow::anyhow!("initial check failed")
                .context(format!("cannot be maximally-delayed due to {u}"));
            return Err(err);
        }
    }
    Ok(())
}

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
        // Diagonal elements included
        work[r].insert(r);
        let gv = &g[v];
        for &w in gv.iter() {
            if let Some(&c) = colset2i.get(&w) {
                work[r].insert(c);
            }
        }
    }
}

fn clear_work_rhs(work: &mut [FixedBitSet]) {
    for row in work.iter_mut() {
        let width = row.len();
        row.remove_range(width - 1..width);
    }
}

type BranchKind = u8;
const BRANCH_XY: BranchKind = 0;
const BRANCH_YZ: BranchKind = 1;
const BRANCH_ZX: BranchKind = 2;

/// Initializes the right-hand side of the work matrix for the upper part.
///
/// # Note
///
/// - `K` specifies the branch kind.
///   - `0`: `XY` branch.
///   - `1`: `YZ` branch.
///   - `2`: `ZX` branch.
fn init_work_upper_rhs<const K: BranchKind>(
    work: &mut [FixedBitSet],
    u: usize,
    g: &Graph,
    rowset: &OrderedNodes,
    colset: &OrderedNodes,
) {
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

fn init_work_lower_rhs<const K: BranchKind>(
    work: &mut [FixedBitSet],
    u: usize,
    g: &Graph,
    rowset: &OrderedNodes,
    colset: &OrderedNodes,
) {
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

fn decode_solution<const K: BranchKind>(u: usize, x: &FixedBitSet, tab: &[usize]) -> Nodes {
    let mut fu = x.ones().map(|c| tab[c]).collect::<Nodes>();
    if K != BRANCH_XY {
        fu.insert(u);
    }
    fu
}

macro_rules! matching_nodes {
    ($src:expr, $p:pat) => {
        $src.iter()
            .filter_map(|(k, &v)| if let $p = v { Some(k) } else { None })
            .copied()
            .collect::<Nodes>()
    };
}

#[pyfunction]
pub fn find(g: Graph, iset: Nodes, oset: Nodes, pplane: InternalPPlanes) -> Option<(PFlow, Layer)> {
    let pplane = pplane
        .into_iter()
        .map(|(k, v)| (k, PPlane::from_u8(v).expect("pplane is in 0..6")))
        .collect::<PPlanes>();
    let yset = matching_nodes!(pplane, PPlane::Y);
    let xyset = matching_nodes!(pplane, PPlane::X | PPlane::Y);
    let yzset = matching_nodes!(pplane, PPlane::Y | PPlane::Z);
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
    // Working memory
    let mut work = vec![FixedBitSet::new(); rowset_upper.len() + rowset_lower.len()];
    let mut tab = Vec::new();
    for l in 0_usize.. {
        cset.clear();
        let mut cleanup = None;
        for &u in &ocset {
            // Perform cleanup
            if let Some((uprev, p0, p1, p2)) = cleanup {
                if p0 {
                    rowset_upper.remove(&uprev);
                }
                if p1 {
                    rowset_lower.insert(uprev);
                }
                if p2 {
                    debug_assert!(!iset.contains(&uprev));
                    colset.insert(uprev);
                }
            }
            // Exclude u and prepare for cleanup
            cleanup = Some((
                u,
                rowset_upper.insert(u),
                rowset_lower.remove(&u),
                colset.remove(&u),
            ));
            let nrows_upper = rowset_upper.len();
            let nrows_lower = rowset_lower.len();
            let ncols = colset.len();
            if nrows_upper + nrows_lower == 0 || ncols == 0 {
                continue;
            }
            // No monotonicity guarantees
            work.resize_with(nrows_upper + nrows_lower, || {
                FixedBitSet::with_capacity(ncols + 1)
            });
            common::zerofill(&mut work, ncols + 1);
            init_work_upper_co(&mut work[..nrows_upper], &g, &rowset_upper, &colset);
            init_work_lower_co(&mut work[nrows_upper..], &g, &rowset_lower, &colset);
            tab.clear();
            tab.extend(colset.iter().copied());
            let mut x = FixedBitSet::with_capacity(ncols);
            let ppu = pplane[&u];
            let mut done = false;
            // TODO: Use macro later
            if !done && matches!(ppu, PPlane::XY | PPlane::X | PPlane::Y) {
                x.clear();
                clear_work_rhs(&mut work);
                init_work_upper_rhs::<BRANCH_XY>(
                    &mut work[..nrows_upper],
                    u,
                    &g,
                    &rowset_upper,
                    &colset,
                );
                init_work_lower_rhs::<BRANCH_XY>(
                    &mut work[nrows_upper..],
                    u,
                    &g,
                    &rowset_lower,
                    &colset,
                );
                let mut solver = GF2Solver::attach(work, 1);
                if solver.solve_in_place(&mut x, 0) {
                    f.insert(u, decode_solution::<BRANCH_XY>(u, &x, &tab));
                    done = true;
                }
                work = solver.detach();
            }
            if !done && matches!(ppu, PPlane::YZ | PPlane::Y | PPlane::Z) {
                x.clear();
                clear_work_rhs(&mut work);
                init_work_upper_rhs::<BRANCH_YZ>(
                    &mut work[..nrows_upper],
                    u,
                    &g,
                    &rowset_upper,
                    &colset,
                );
                init_work_lower_rhs::<BRANCH_YZ>(
                    &mut work[nrows_upper..],
                    u,
                    &g,
                    &rowset_lower,
                    &colset,
                );
                let mut solver = GF2Solver::attach(work, 1);
                if solver.solve_in_place(&mut x, 0) {
                    f.insert(u, decode_solution::<BRANCH_YZ>(u, &x, &tab));
                    done = true;
                }
                work = solver.detach();
            }
            if !done && matches!(ppu, PPlane::ZX | PPlane::Z | PPlane::X) {
                x.clear();
                clear_work_rhs(&mut work);
                init_work_upper_rhs::<BRANCH_ZX>(
                    &mut work[..nrows_upper],
                    u,
                    &g,
                    &rowset_upper,
                    &colset,
                );
                init_work_lower_rhs::<BRANCH_ZX>(
                    &mut work[nrows_upper..],
                    u,
                    &g,
                    &rowset_lower,
                    &colset,
                );
                let mut solver = GF2Solver::attach(work, 1);
                if solver.solve_in_place(&mut x, 0) {
                    f.insert(u, decode_solution::<BRANCH_ZX>(u, &x, &tab));
                    done = true;
                }
                work = solver.detach();
            }
            if done {
                layer[u] = l;
                cset.insert(u);
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
        // TODO: Uncomment once ready
        // if cfg!(debug_assertions) {
        let f_flatiter = f
            .iter()
            .flat_map(|(i, fi)| Iterator::zip(iter::repeat(i), fi.iter()));
        common::check_domain(f_flatiter, &vset, &iset, &oset).unwrap();
        check_initial_pflow(&layer, &oset).unwrap();
        // }
        Some((f, layer))
    } else {
        None
    }
}

