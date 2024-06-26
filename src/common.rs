//! Common functionalities for the flow/gflow algorithms.

use anyhow;
use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
    ops::Deref,
};

/// Undirected graph represented as an adjacency list.
pub type Graph = Vec<HashSet<usize>>;
/// Numbered representation of the associated partial order of flow/gflow.
pub type Layer = Vec<usize>;

/// Checks if `layer[u] == 0` iff `u` is in `oset`.
pub fn check_initial(layer: &Layer, oset: &HashSet<usize>) -> anyhow::Result<()> {
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

/// Computes the odd neighbors of the vertices in `kset`.
pub fn odd_neighbors(g: &Graph, kset: &HashSet<usize>) -> HashSet<usize> {
    if kset.iter().any(|&ki| ki >= g.len()) {
        panic!("kset out of range");
    }
    let mut src = kset.clone();
    src.extend(kset.iter().flat_map(|&ki| g[ki].iter().copied()));
    let mut res = HashSet::new();
    for u in src {
        if kset.intersection(&g[u]).count() % 2 == 1 {
            res.insert(u);
        }
    }
    res
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

impl<T> InPlaceSetOp<T> for HashSet<T>
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Plane {
    XY,
    YZ,
    ZX,
}

impl TryFrom<u8> for Plane {
    type Error = anyhow::Error;

    fn try_from(p: u8) -> anyhow::Result<Self> {
        match p {
            0 => Ok(Plane::XY),
            1 => Ok(Plane::YZ),
            2 => Ok(Plane::ZX),
            _ => Err(anyhow::anyhow!("invalid plane index").context(p)),
        }
    }
}
