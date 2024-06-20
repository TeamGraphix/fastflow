// MEMO: Data verification is done in the Python layer.
use pyo3::prelude::*;
pub mod common;
pub mod flow;

#[pymodule]
fn _impl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(flow::find, m)?)?;
    Ok(())
}
