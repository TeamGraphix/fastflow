use pyo3::prelude::*;
mod common;
mod flow;
mod gf2_linalg;
mod gflow;

// MEMO: Data verification is done in the Python layer.

// fastflow._impl
#[pymodule]
fn _impl(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
