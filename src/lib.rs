//! Entry point of the Rust binding.
//!
//! From the Python side, bindings are visible as `fastflow._impl.XXX`.

#[cfg(test)]
pub mod test_utils;

pub mod common;
pub mod flow;
pub mod gf2_linalg;
pub mod gflow;
pub mod pflow;
pub mod validate;

use pyo3::prelude::*;

// MEMO: Data verification is done in the Python layer

// fastflow._impl
#[pymodule]
#[pyo3(name = "_impl")]
fn entrypoint(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // fastflow._impl.flow
    let mod_flow = PyModule::new_bound(m.py(), "flow")?;
    mod_flow.add_function(wrap_pyfunction!(flow::find, &mod_flow)?)?;
    m.add_submodule(&mod_flow)?;
    // fastflow._impl.gflow
    let mod_gflow = PyModule::new_bound(m.py(), "gflow")?;
    mod_gflow.add_function(wrap_pyfunction!(gflow::find, &mod_gflow)?)?;
    m.add_submodule(&mod_gflow)?;
    Ok(())
}
