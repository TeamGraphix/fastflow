//! Utilities.

use std::{
    collections::BTreeSet,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use fixedbitset::FixedBitSet;

use crate::common::{Graph, Nodes, OrderedNodes};

/// Computes the odd neighbors of the nodes in `kset`.
///
/// # Note
///
/// - Naive implementation only for post-verification.
pub fn odd_neighbors(g: &Graph, kset: &Nodes) -> Nodes {
    assert!(kset.iter().all(|&ki| ki < g.len()), "kset out of range");
    let mut work = kset.clone();
    work.extend(kset.iter().flat_map(|&ki| g[ki].iter().copied()));
    work.retain(|&u| kset.intersection(&g[u]).count() % 2 == 1);
    work
}

/// Resizes `mat` to `mat.len()` x `ncols` and fills with zeros.
pub fn zerofill(mat: &mut [FixedBitSet], ncols: usize) {
    let src = FixedBitSet::with_capacity(ncols);
    for x in mat.iter_mut() {
        x.clone_from(&src);
    }
}

/// Helper trait for in-place set operations.
pub trait InPlaceSetDiff<T> {
    /// Drops the elements from `other` from self.
    fn difference_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>;
}

impl<T> InPlaceSetDiff<T> for hashbrown::HashSet<T>
where
    T: Eq + Hash,
{
    fn difference_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        other.into_iter().for_each(|x| {
            self.remove(&*x);
        });
    }
}

impl<T> InPlaceSetDiff<T> for BTreeSet<T>
where
    T: Eq + Ord,
{
    fn difference_with<U>(&mut self, other: impl IntoIterator<Item = U>)
    where
        U: Deref<Target = T>,
    {
        other.into_iter().for_each(|x| {
            self.remove(&*x);
        });
    }
}

/// Computes element-to-index mapping for an ordered set.
pub fn indexmap<T: FromIterator<(usize, usize)>>(set: &OrderedNodes) -> T {
    set.iter().enumerate().map(|(i, &x)| (x, i)).collect()
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
    use std::collections::BTreeMap;

    use super::*;
    use crate::internal::test_utils::{TestCase, CASE3};

    #[test]
    fn test_odd_neighbors() {
        let TestCase { g, .. } = &*CASE3;
        for i in 0..g.len() {
            assert_eq!(odd_neighbors(g, &Nodes::from([i])), g[i]);
        }
        assert_eq!(
            odd_neighbors(g, &Nodes::from([0, 3])),
            Nodes::from([0, 1, 3, 5])
        );
        assert_eq!(
            odd_neighbors(g, &Nodes::from([1, 4])),
            Nodes::from([1, 2, 3, 4, 5])
        );
        assert_eq!(
            odd_neighbors(g, &Nodes::from([2, 5])),
            Nodes::from([0, 1, 2, 4, 5])
        );
        assert_eq!(odd_neighbors(g, &Nodes::from([0, 1, 2])), Nodes::from([5]));
        assert_eq!(odd_neighbors(g, &Nodes::from([3, 4, 5])), Nodes::from([1]));
        assert_eq!(
            odd_neighbors(g, &Nodes::from([0, 1, 2, 3, 4, 5])),
            Nodes::from([1, 5])
        );
    }

    #[test]
    fn test_zerofill() {
        let mut mat = vec![FixedBitSet::new(), FixedBitSet::new(), FixedBitSet::new()];
        zerofill(&mut mat, 10);
        for row in &mat {
            assert_eq!(row.len(), 10);
            assert!(row.is_clear());
        }
    }

    #[test]
    fn test_difference_with_hashset() {
        let mut set = hashbrown::HashSet::from([1, 2, 3]);
        set.difference_with(&[2, 3, 4]);
        assert_eq!(set, hashbrown::HashSet::from([1]));
    }

    #[test]
    fn test_difference_with_btreeset() {
        let mut set = BTreeSet::from([1, 2, 3]);
        set.difference_with(&[2, 3, 4]);
        assert_eq!(set, BTreeSet::from([1]));
    }

    #[test]
    fn test_indexmap() {
        let set = OrderedNodes::from([8, 1, 0]);
        let imap = indexmap::<BTreeMap<_, _>>(&set);
        assert_eq!(imap[&0], 0);
        assert_eq!(imap[&1], 1);
        assert_eq!(imap[&8], 2);
    }

    #[test]
    fn test_scoped_include() {
        let mut set = OrderedNodes::new();
        {
            let mut guard = ScopedInclude::new(&mut set, 0);
            // Mutable borrow
            guard.insert(1);
            // Immutable borrow
            assert_eq!(*guard, OrderedNodes::from([0, 1]));
        }
        assert_eq!(set, OrderedNodes::from([1]));
    }

    #[test]
    fn test_scoped_exclude() {
        let mut set = OrderedNodes::from([0]);
        {
            let mut guard = ScopedExclude::new(&mut set, 0);
            // Mutable borrow
            guard.insert(1);
            // Immutable borrow
            assert_eq!(*guard, OrderedNodes::from([1]));
        }
        assert_eq!(set, OrderedNodes::from([0, 1]));
    }
}
