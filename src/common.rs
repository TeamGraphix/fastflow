//! Common functionalities.

use std::{collections::BTreeSet, hash::Hash, ops::Deref};

use fixedbitset::FixedBitSet;

pub type Nodes = hashbrown::HashSet<usize>;
pub type OrderedNodes = BTreeSet<usize>;
pub type Graph = Vec<Nodes>;
pub type Layer = Vec<usize>;

/// Checks if the layer-zero nodes are correctly chosen.
///
/// # Arguments
///
/// - `layer`: The layer.
/// - `oset`: The set of output nodes.
/// - `iff`: If `true`, `layer[u] == 0` "iff" `u` is in `oset`. Otherwise "if".
pub fn check_initial(layer: &Layer, oset: &Nodes, iff: bool) -> anyhow::Result<()> {
    for (u, &lu) in layer.iter().enumerate() {
        match (oset.contains(&u), lu == 0) {
            (true, false) => {
                let err = anyhow::anyhow!("initial check failed")
                    .context(format!("layer({u}) != 0 && {u} in O"));
                return Err(err);
            }
            (false, true) if iff => {
                let err = anyhow::anyhow!("initial check failed")
                    .context(format!("layer({u}) == 0 && {u} not in O"));
                return Err(err);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Checks if the domain of `f` is in `vset - oset` and the codomain is in `vset - iset`.
///
/// # Arguments
///
/// - `f_flatiter`: Flow, gflow, or pflow as `impl Iterator<Item = (&usize, &usize)>`.
/// - `vset`: All nodes.
/// - `iset`: Input nodes.
/// - `oset`: Output nodes.
///
/// # Note
///
/// It is allowed for `f[i]` to contain `i`, even if `i` is in `iset`.
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
        if i != fi && !icset.contains(&fi) {
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
