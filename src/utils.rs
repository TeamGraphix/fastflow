//! Utilities.

use std::{
    collections::BTreeSet,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use fixedbitset::FixedBitSet;

use crate::common::{Graph, Nodes, OrderedNodes};

/// Computes the odd neighbors of the vertices in `kset`.
///
/// # Note
///
/// - Naive implementation only for post-verification.
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
    fn union_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>;

    /// Drops the elements from `other` from self.
    fn difference_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>;
}

impl<T> InPlaceSetOp<T> for hashbrown::HashSet<T>
where
    T: Eq + Clone + Hash,
{
    fn union_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        self.extend(other.into_iter().map(|x| x.deref().clone()));
    }

    fn difference_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        other.into_iter().for_each(|x| {
            self.remove(x.deref());
        });
    }
}

impl<T> InPlaceSetOp<T> for BTreeSet<T>
where
    T: Eq + Clone + Ord,
{
    fn union_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        self.extend(other.into_iter().map(|x| x.deref().clone()));
    }

    fn difference_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        other.into_iter().for_each(|x| {
            self.remove(x.deref());
        });
    }
}

#[derive(Debug)]
/// RAII guard for inserting a node into a set.
///
/// Inserts `u` on construction and reverts on drop.
pub struct ScopedInclude<'a> {
    target: &'a mut OrderedNodes,
    u: Option<usize>,
}

impl<'a> ScopedInclude<'a> {
    pub fn new(target: &'a mut OrderedNodes, u: usize) -> Self {
        let u = if target.insert(u) { Some(u) } else { None };
        Self { target, u }
    }
}

impl Deref for ScopedInclude<'_> {
    type Target = OrderedNodes;

    fn deref(&self) -> &Self::Target {
        self.target
    }
}

impl DerefMut for ScopedInclude<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.target
    }
}

impl Drop for ScopedInclude<'_> {
    fn drop(&mut self) {
        if let Some(u) = self.u {
            self.target.remove(&u);
        }
    }
}

#[derive(Debug)]
/// RAII guard for deleting a node from a set.
///
/// Removes `u` on construction and reverts on drop.
pub struct ScopedExclude<'a> {
    target: &'a mut OrderedNodes,
    u: Option<usize>,
}

impl<'a> ScopedExclude<'a> {
    pub fn new(target: &'a mut OrderedNodes, u: usize) -> Self {
        let u = if target.remove(&u) { Some(u) } else { None };
        Self { target, u }
    }
}

impl Deref for ScopedExclude<'_> {
    type Target = OrderedNodes;

    fn deref(&self) -> &Self::Target {
        self.target
    }
}

impl DerefMut for ScopedExclude<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.target
    }
}

impl Drop for ScopedExclude<'_> {
    fn drop(&mut self) {
        if let Some(u) = self.u {
            self.target.insert(u);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{TestCase, CASE3};

    #[test]
    fn test_odd_neighbors() {
        let TestCase { g, .. } = &*CASE3;
        for i in 0..g.len() {
            assert_eq!(odd_neighbors(g, &nodes![i]), g[i]);
        }
        assert_eq!(odd_neighbors(g, &nodes![0, 3]), nodes![0, 1, 3, 5]);
        assert_eq!(odd_neighbors(g, &nodes![1, 4]), nodes![1, 2, 3, 4, 5]);
        assert_eq!(odd_neighbors(g, &nodes![2, 5]), nodes![0, 1, 2, 4, 5]);
        assert_eq!(odd_neighbors(g, &nodes![0, 1, 2]), nodes![5]);
        assert_eq!(odd_neighbors(g, &nodes![3, 4, 5]), nodes![1]);
        assert_eq!(odd_neighbors(g, &nodes![0, 1, 2, 3, 4, 5]), nodes![1, 5]);
    }
}
