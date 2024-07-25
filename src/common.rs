//! Common functionalities.

use std::collections::BTreeSet;

pub type Nodes = hashbrown::HashSet<usize>;
pub type OrderedNodes = BTreeSet<usize>;
pub type Graph = Vec<Nodes>;
pub type Layer = Vec<usize>;
