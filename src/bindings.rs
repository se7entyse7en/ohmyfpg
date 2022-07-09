use crate::client::{self, Connection, ConnectionError};
use futures::future::FutureExt;
use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyOSError};
use pyo3::prelude::*;

create_exception!(ohmyfpg, PyInvalidDSNError, PyException, "Invalid DSN.");
create_exception!(
    ohmyfpg,
    PyUnrecognizedMessageError,
    PyException,
    "Unrecognized message."
);

/// Connect to the database and return a `Connection` object.
#[pyfunction]
pub fn connect(py: Python, dsn: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(
        py,
        client::connect(dsn).map(|res| res.map(PyConnection::from).map_err(PyErr::from)),
    )
}

/// Connection object exposed to Python
#[pyclass(name = "Connection")]
pub struct PyConnection {
    _wrappee: Connection,
}

impl From<Connection> for PyConnection {
    fn from(conn: Connection) -> Self {
        PyConnection { _wrappee: conn }
    }
}

impl From<ConnectionError> for PyErr {
    fn from(err: ConnectionError) -> Self {
        match err {
            ConnectionError::InvalidDSNError(err) => PyInvalidDSNError::new_err(err.to_string()),
            ConnectionError::IOError(err) => PyOSError::new_err(err.to_string()),
            ConnectionError::UnrecognizedMessage(msg_type) => {
                PyUnrecognizedMessageError::new_err(msg_type)
            }
        }
    }
}
