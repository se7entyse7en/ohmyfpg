use futures::future::FutureExt;
use ohmyfpg_core::client::{
    self, ColumnResult, Connection, ConnectionError, FetchError, MessageReadError,
};
use pyo3::conversion::IntoPy;
use pyo3::create_exception;
use pyo3::exceptions::{self, PyException, PyOSError};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

create_exception!(ohmyfpg, PyInvalidDsnError, PyException, "Invalid Dsn.");
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
        client::connect(dsn).map(|res| {
            res.map(PyConnection::from)
                .map_err(|err| PyErr::from(LocalConnectionError(err)))
        }),
    )
}

/// Connection object exposed to Python
#[pyclass(name = "Connection")]
pub struct PyConnection {
    wrappee: Arc<Mutex<Connection>>,
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
                .map(|fr| {
                    Python::with_gil(|py| {
                        let local_fr: HashMap<String, LocalColumnResult> = fr
                            .into_iter()
                            .map(|v| (v.0, LocalColumnResult(v.1)))
                            .collect();
                        local_fr.into_py(py)
                    })
                })
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

struct LocalColumnResult(ColumnResult);

impl IntoPy<Py<PyAny>> for LocalColumnResult {
    fn into_py(self, py: Python<'_>) -> Py<PyAny> {
        let tup = (
            self.0.bytes.as_slice().into_py(py),
            self.0.dtype.into_py(py),
        );
        tup.into_py(py)
    }
}

struct LocalConnectionError(ConnectionError);

impl From<LocalConnectionError> for PyErr {
    fn from(local_err: LocalConnectionError) -> Self {
        let err = local_err.0;
        match err {
            ConnectionError::InvalidDsnError(err) => PyInvalidDsnError::new_err(err.to_string()),
            ConnectionError::FetchError(err) => match err {
                FetchError::MessageReadError(err) => match err {
                    MessageReadError::UnrecognizedMessageError(err) => {
                        PyUnrecognizedMessageError::new_err(err.to_string())
                    }
                    MessageReadError::IOError(err) => PyOSError::new_err(err.to_string()),
                },
                FetchError::UnexpectedMessageError(msg) => {
                    PyUnexpectedMessageError::new_err(format!("{:?}", msg))
                }
            },
            ConnectionError::ServerError(err) => PyServerError::new_err(err.to_string()),
        }
    }
}
