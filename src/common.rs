//! Common functionalities.

use std::collections::BTreeSet;

use thiserror::Error;

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

/// Error type for flow validation.
#[derive(Debug, Error)]
pub enum FlowValidationError {
    #[error("non-zero-layer node inside output nodes ({0})")]
    ExcessiveNonZeroLayer(usize),
    #[error("zero-layer node outside output nodes ({0})")]
    ExcessiveZeroLayer(usize),
    #[error("flow function has invalid codomain ({0})")]
    InvalidFlowCodomain(usize),
    #[error("flow function has invalid domain ({0})")]
    InvalidFlowDomain(usize),
    #[error("measurement specification is excessive or insufficient ({0})")]
    InvalidMeasurementSpec(usize),
    #[error("flow function and partial order are inconsistent ({0}, {1})")]
    InconsistentFlowOrder(usize, usize),
    #[error("flow function and measurement specification are inconsistent ({0})")]
    InconsistentFlowPlane(usize),
    #[error("flow function and measurement specification are inconsistent ({0})")]
    InconsistentFlowPPlane(usize),
}
