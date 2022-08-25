use crate::client::{self, ColumnResult, Connection, ConnectionError};
use futures::future::FutureExt;
use pyo3::conversion::IntoPy;
use pyo3::create_exception;
use pyo3::exceptions::{self, PyException, PyOSError};
use pyo3::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

create_exception!(ohmyfpg, PyInvalidDSNError, PyException, "Invalid DSN.");
create_exception!(
    ohmyfpg,
    PyUnrecognizedMessageError,
    PyException,
    "Unrecognized message."
);
create_exception!(
    ohmyfpg,
    PyUnexpectedMessageError,
    PyException,
    "Unexpected message."
);
create_exception!(ohmyfpg, PyServerError, PyException, "Server error.");

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
    wrappee: Arc<Mutex<Connection>>,
}

impl IntoPy<Py<PyAny>> for ColumnResult {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        let tup = (self.bytes.as_slice().into_py(py), self.dtype.into_py(py));
        tup.into_py(py)
    }
}

#[pymethods]
impl PyConnection {
    fn fetch<'a>(&self, py: Python<'a>, query_string: String) -> PyResult<&'a PyAny> {
        let mutext_conn = Arc::clone(&self.wrappee);
        pyo3_asyncio::tokio::future_into_py(py, async move {
            mutext_conn
                .lock()
                .await
                .fetch(query_string)
                .await
                .map(|fr| Python::with_gil(|py| fr.into_py(py)))
                .map_err(|_| exceptions::PyException::new_err("TODO"))
        })
    }
}

impl From<Connection> for PyConnection {
    fn from(conn: Connection) -> Self {
        PyConnection {
            wrappee: Arc::new(Mutex::new(conn)),
        }
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
            ConnectionError::UnexpectedMessage(msg) => {
                PyUnexpectedMessageError::new_err(format!("{:?}", msg))
            }
            ConnectionError::ServerError(err) => PyServerError::new_err(err.to_string()),
        }
    }
}
