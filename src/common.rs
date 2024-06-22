use std::{
    collections::{BTreeSet, HashMap, HashSet},
    hash::Hash,
};

pub type Flow = HashMap<usize, usize>;
pub type GFlow = HashMap<usize, HashSet<usize>>;
pub type Layer = Vec<usize>;

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
