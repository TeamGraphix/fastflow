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

use gflow::Plane;
use pflow::PPlane;
use pyo3::prelude::*;

// MEMO: Data verification is done in the Python layer

// fastflow._impl
#[pymodule]
#[pyo3(name = "_impl")]
#[allow(clippy::similar_names)]
fn entrypoint(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // fastflow._impl.flow
    let mod_flow = PyModule::new_bound(m.py(), "flow")?;
    mod_flow.add_function(wrap_pyfunction!(flow::find, &mod_flow)?)?;
    m.add_submodule(&mod_flow)?;
    // fastflow._impl.gflow
    let mod_gflow = PyModule::new_bound(m.py(), "gflow")?;
    mod_gflow.add_class::<Plane>()?;
    mod_gflow.add_function(wrap_pyfunction!(gflow::find, &mod_gflow)?)?;
    m.add_submodule(&mod_gflow)?;
    // fastflow._impl.pflow
    let mod_pflow = PyModule::new_bound(m.py(), "pflow")?;
    mod_pflow.add_class::<PPlane>()?;
    mod_pflow.add_function(wrap_pyfunction!(pflow::find, &mod_pflow)?)?;
    m.add_submodule(&mod_pflow)?;
    Ok(())
}
