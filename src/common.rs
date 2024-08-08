//! Common functionalities.

use std::collections::BTreeSet;

/// Set of nodes indexed by 0-based integers.
pub type Nodes = hashbrown::HashSet<usize>;
/// Simple graph encoded as list of neighbors.
pub type Graph = Vec<Nodes>;
/// Layer representation of the flow partial order.
pub type Layer = Vec<usize>;

/// Ordered set of nodes.
///
/// # Note
///
/// Used only when iteration order matters.
pub(crate) type OrderedNodes = BTreeSet<usize>;
