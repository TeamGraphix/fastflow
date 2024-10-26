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
    #[error("Non-zero-layer node inside output nodes.")]
    ExcessiveNonZeroLayer(usize),
    #[error("Zero-layer node outside output nodes.")]
    ExcessiveZeroLayer(usize),
    #[error("Flow function has invalid codomain.")]
    InvalidFlowCodomain(usize),
    #[error("Flow function has invalid domain.")]
    InvalidFlowDomain(usize),
    #[error("Measurement specification is excessive or insufficient.")]
    InvalidMeasurementSpec,
    #[error("Flow function and partial order are inconsistent.")]
    InconsistentFlowOrder(usize, usize),
    #[error("Flow function and measurement specification are inconsistent.")]
    InconsistentFlowPlane(usize),
    #[error("Flow function and measurement specification are inconsistent.")]
    InconsistentFlowPPlane(usize),
}
