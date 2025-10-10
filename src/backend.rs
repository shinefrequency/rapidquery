use pyo3::types::PyAnyMethods;

#[pyo3::pyclass(module = "rapidquery._lib", name = "BackendMeta", frozen, subclass)]
pub struct PyBackendMeta;

#[pyo3::pymethods]
impl PyBackendMeta {
    #[new]
    fn new() -> pyo3::PyResult<Self> {
        Err(
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>("don't use directly BackendMeta class; use PostgreSQLBackend, MySQLBackend, or SQLiteBackend")
        )
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "PostgreSQLBackend", frozen, extends=PyBackendMeta)]
pub struct PyPostgreSQLBackend;

#[pyo3::pymethods]
impl PyPostgreSQLBackend {
    #[new]
    fn new() -> (Self, PyBackendMeta) {
        (Self, PyBackendMeta)
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "MySQLBackend", frozen, extends=PyBackendMeta)]
pub struct PyMySQLBackend;

#[pyo3::pymethods]
impl PyMySQLBackend {
    #[new]
    fn new() -> (Self, PyBackendMeta) {
        (Self, PyBackendMeta)
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "SQLiteBackend", frozen, extends=PyBackendMeta)]
pub struct PySQLiteBackend;

#[pyo3::pymethods]
impl PySQLiteBackend {
    #[new]
    fn new() -> (Self, PyBackendMeta) {
        (Self, PyBackendMeta)
    }
}

#[inline]
#[optimize(speed)]
pub(crate) fn into_query_builder(
    object: &pyo3::Bound<'_, pyo3::PyAny>,
) -> Option<Box<dyn sea_query::QueryBuilder>> {
    if object.is_exact_instance_of::<PySQLiteBackend>() {
        Some(Box::new(sea_query::SqliteQueryBuilder))
    } else if object.is_exact_instance_of::<PyMySQLBackend>() {
        Some(Box::new(sea_query::MysqlQueryBuilder))
    } else if object.is_exact_instance_of::<PyPostgreSQLBackend>() {
        Some(Box::new(sea_query::PostgresQueryBuilder))
    } else {
        None
    }
}
