use pyo3::prelude::*;
pub mod bindings;

/// Entrypoint for `ohmyfpg` Rust implementation
#[pymodule]
fn ohmyfpg(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<bindings::PyConnection>()?;
    m.add(
        "PyInvalidDSNError",
        py.get_type::<bindings::PyInvalidDsnError>(),
    )?;
    m.add_function(wrap_pyfunction!(bindings::connect, m)?)?;
    Ok(())
}
