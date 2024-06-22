use std::{
    collections::{BTreeSet, HashSet},
    hash::Hash,
};

pub type Graph = Vec<HashSet<usize>>;
pub type Layer = Vec<usize>;

#[cfg(debug_assertions)]
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

pub fn odd_neighbors(g: &Graph, kset: &HashSet<usize>) -> HashSet<usize> {
    if kset.iter().any(|&ki| ki >= g.len()) {
        panic!("kset out of range");
    }
    let mut src = kset.clone();
    src.extend(
        kset.iter()
            .flat_map(|&ki| g[ki].iter().copied())
            .collect::<HashSet<_>>(),
    );
    let mut res = HashSet::new();
    for u in src {
        if kset.intersection(&g[u]).count() % 2 == 1 {
            res.insert(u);
        }
    }
    res
}

pub trait InPlaceSetOp<T: Clone> {
    fn union_with<'a>(&'a mut self, other: impl Iterator<Item = &'a T>)
    where
        // Need to ensure that we can create &'a T from T
        T: 'a;

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
        self.extend(other.map(|x| x.clone()));
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
        self.extend(other.map(|x| x.clone()));
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
