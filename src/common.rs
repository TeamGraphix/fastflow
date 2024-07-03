//! Common functionalities for the flow/gflow algorithms.

use anyhow;
use fixedbitset::FixedBitSet;
use hashbrown;
use std::{collections::BTreeSet, hash::Hash, ops::Deref};

/// Set of nodes.
pub type Nodes = hashbrown::HashSet<usize>;
/// Ordered set of nodes.
pub type OrderedNodes = BTreeSet<usize>;
/// Undirected graph represented as an adjacency list.
pub type Graph = Vec<Nodes>;
/// Numbered representation of the associated partial order of flow/gflow.
pub type Layer = Vec<usize>;

/// Checks if `layer[u] == 0` iff `u` is in `oset`.
pub fn check_initial(layer: &Layer, oset: &Nodes) -> anyhow::Result<()> {
    for (u, &lu) in layer.iter().enumerate() {
        // u in O => layer[u] == 0
        // u not in O => layer[u] != 0
        let valid = oset.contains(&u) ^ (lu != 0);
        if !valid {
            let err = anyhow::anyhow!("initial check failed")
                .context(format!("cannot be maximally-delayed due to {u}"));
            return Err(err);
        }
    }
    Ok(())
}

/// Checks if the domain of `f` is in V\O and the codomain is in V\I.
///
/// # Note
///
/// - Accepts `impl Iterator<Item = (&usize, &usize)>` to reuse it for flow, gflow, and pflow.
pub fn check_domain<'a, 'b>(
    f_flatiter: impl Iterator<Item = (&'a usize, &'b usize)>,
    vset: &Nodes,
    iset: &Nodes,
    oset: &Nodes,
) -> anyhow::Result<()> {
    let icset = vset - iset;
    let ocset = vset - oset;
    for (&i, &fi) in f_flatiter {
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

/// Computes the odd neighbors of the vertices in `kset`.
///
/// # Note
///
/// - Naively implemented.
pub fn odd_neighbors(g: &Graph, kset: &Nodes) -> Nodes {
    if kset.iter().any(|&ki| ki >= g.len()) {
        panic!("kset out of range");
    }
    let mut work = kset.clone();
    work.extend(kset.iter().flat_map(|&ki| g[ki].iter().copied()));
    work.retain(|&u| kset.intersection(&g[u]).count() % 2 == 1);
    work
}

/// Resizes `mat` to `mat.len()` x `ncols` and fills with zeros.
pub fn zerofill(mat: &mut [FixedBitSet], ncols: usize) {
    let src = FixedBitSet::with_capacity(ncols);
    mat.iter_mut().for_each(|x| {
        x.clone_from(&src);
    });
}

/// Helper trait for in-place set operations.
pub trait InPlaceSetOp<T: Clone> {
    /// Extends self with the elements from `other`.
    fn union_with<U>(&mut self, other: impl Iterator<Item = U>)
    where
        U: Deref<Target = T>;

    /// Drops the elements from `other` from self.
    fn difference_with<U>(&mut self, other: impl Iterator<Item = U>)
    where
        U: Deref<Target = T>;
}

impl<T> InPlaceSetOp<T> for hashbrown::HashSet<T>
where
    T: Eq + Clone + Hash,
{
    fn union_with<U>(&mut self, other: impl Iterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        self.extend(other.map(|x| x.deref().clone()));
    }

    fn difference_with<U>(&mut self, other: impl Iterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        other.for_each(|x| {
            self.remove(x.deref());
        });
    }
}

impl<T> InPlaceSetOp<T> for BTreeSet<T>
where
    T: Eq + Clone + Ord,
{
    fn union_with<U>(&mut self, other: impl Iterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        self.extend(other.map(|x| x.deref().clone()));
    }

    fn difference_with<U>(&mut self, other: impl Iterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        other.for_each(|x| {
            self.remove(x.deref());
        });
    }
}
