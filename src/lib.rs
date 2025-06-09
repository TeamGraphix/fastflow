//! Entry point of the Rust binding.
//!
//! From the Python side, bindings are visible as `fastflow._impl.XXX`.
#![warn(clippy::pedantic)]

#[macro_use]
mod internal;

pub mod common;
pub mod flow;
pub mod gflow;
pub mod pflow;
pub mod solver;

use common::FlowValidationError;
use gflow::Plane;
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray2};
use pflow::PPlane;
use pyo3::{prelude::*, Bound, Python};
use solver::Solver;

// MEMO: Data verification is done in the Python layer

/// Solve linear equations on GF(2).
#[pyfunction]
#[allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]
pub fn solve<'py>(
    py: Python<'py>,
    a: PyReadonlyArray2<'py, bool>,
    b: PyReadonlyArray2<'py, bool>,
) -> Vec<Option<Bound<'py, PyArray1<bool>>>> {
    let a = a.as_array();
    let b = b.as_array();
    let mut solver = Solver::from_eq(&a, &b);
    let (_, neqs) = b.dim();
    (0..neqs)
        .map(|i| solver.solve(i).map(|x| x.into_pyarray(py)))
        .collect()
}

// fastflow._impl
#[pymodule]
#[pyo3(name = "_impl")]
#[allow(clippy::similar_names)]
fn entrypoint(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Remapped to fastflow._impl.FlowValidationMessage
    m.add_class::<FlowValidationError>()?;
    // fastflow._impl.solve
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    // fastflow._impl.flow
    let mod_flow = PyModule::new(m.py(), "flow")?;
    mod_flow.add_function(wrap_pyfunction!(flow::find, &mod_flow)?)?;
    mod_flow.add_function(wrap_pyfunction!(flow::verify, &mod_flow)?)?;
    m.add_submodule(&mod_flow)?;
    // fastflow._impl.gflow
    let mod_gflow = PyModule::new(m.py(), "gflow")?;
    mod_gflow.add_class::<Plane>()?;
    mod_gflow.add_function(wrap_pyfunction!(gflow::find, &mod_gflow)?)?;
    mod_gflow.add_function(wrap_pyfunction!(gflow::verify, &mod_gflow)?)?;
    m.add_submodule(&mod_gflow)?;
    // fastflow._impl.pflow
    let mod_pflow = PyModule::new(m.py(), "pflow")?;
    mod_pflow.add_class::<PPlane>()?;
    mod_pflow.add_function(wrap_pyfunction!(pflow::find, &mod_pflow)?)?;
    mod_pflow.add_function(wrap_pyfunction!(pflow::verify, &mod_pflow)?)?;
    m.add_submodule(&mod_pflow)?;
    Ok(())
}
