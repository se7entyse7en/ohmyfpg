use pyo3::prelude::*;
use tokio::net::TcpStream;

#[pyclass]
pub struct Connection {
    _stream: TcpStream,
}

/// Returns a `Connection` object
#[pyfunction]
fn connect(py: Python, dsn: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        println!("Connecting to {}...", dsn);
        let stream = TcpStream::connect(dsn).await?;
        println!("Connected");
        let connection = Connection { _stream: stream };
        Ok(connection)
    })
}

/// A Python module implemented in Rust.
#[pymodule]
fn ohmyfpg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Connection>()?;
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    Ok(())
}
