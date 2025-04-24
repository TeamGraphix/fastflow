//! Common functionalities.

use std::collections::BTreeSet;

use pyo3::{exceptions::PyValueError, prelude::*};
use thiserror::Error;

use crate::{gflow::Plane, pflow::PPlane};

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
///
/// Python name does not contain `Error` as it is not a subclass of `Exception`.
#[pyclass(name = "FlowValidationMessage")]
#[derive(Debug, Error)]
pub enum FlowValidationError {
    #[error("layer-{layer} node {node} inside output nodes")]
    ExcessiveNonZeroLayer { node: usize, layer: usize },
    #[error("zero-layer node {node} outside output nodes")]
    ExcessiveZeroLayer { node: usize },
    #[error("f({node}) has invalid codomain")]
    InvalidFlowCodomain { node: usize },
    #[error("f({node}) has invalid domain")]
    InvalidFlowDomain { node: usize },
    #[error("node {node} has invalid measurement specification")]
    InvalidMeasurementSpec { node: usize },
    #[error("flow-order inconsistency on edge ({}, {})",.edge.0, .edge.1)]
    InconsistentFlowOrder { edge: (usize, usize) },
    #[error("broken {plane:?} measurement on node {node}")]
    InconsistentFlowPlane { node: usize, plane: Plane },
    #[error("broken {pplane:?} measurement on node {node}")]
    InconsistentFlowPPlane { node: usize, pplane: PPlane },
}

impl From<FlowValidationError> for PyErr {
    fn from(e: FlowValidationError) -> Self {
        PyValueError::new_err(e)
    }
}
