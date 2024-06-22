//! Common functionalities for the flow/gflow algorithms.

use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
};

/// Undirected graph represented as an adjacency list.
pub type Graph = Vec<HashSet<usize>>;
/// Numbered representation of the associated partial order of flow/gflow.
pub type Layer = Vec<usize>;

/// Check if `layer[u] == 0` iff `u` is in `oset`.
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

/// Compute the odd neighbors of the vertices in `kset`.
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
    /// Extend self with the elements from `other`.
    fn union_with<'a>(&'a mut self, other: impl Iterator<Item = &'a T>)
    where
        // Need to ensure that we can create &'a T from T
        T: 'a;

    /// Drop the elements from `other` from self.
    fn difference_with<'a>(&'a mut self, other: impl Iterator<Item = &'a T>)
    where
        T: 'a;
}

impl<T> InPlaceSetOp<T> for HashSet<T>
where
    T: Eq + Clone + Hash,
{
    fn union_with<'a>(&mut self, other: impl Iterator<Item = &'a T>)
    where
        T: 'a,
    {
        self.extend(other.cloned());
    }

    fn difference_with<'a>(&mut self, other: impl Iterator<Item = &'a T>)
    where
        T: 'a,
    {
        other.for_each(|x| {
            self.remove(x);
        });
    }
}

impl<T> InPlaceSetOp<T> for BTreeSet<T>
where
    T: Eq + Clone + Ord,
{
    fn union_with<'a>(&mut self, other: impl Iterator<Item = &'a T>)
    where
        T: 'a,
    {
        self.extend(other.cloned());
    }

    fn difference_with<'a>(&mut self, other: impl Iterator<Item = &'a T>)
    where
        T: 'a,
    {
        other.for_each(|x| {
            self.remove(x);
        });
    }
}
