#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "SchemaStatement",
    frozen,
    immutable_type,
    subclass
)]
pub struct PySchemaStatement;

#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "QueryStatement",
    frozen,
    immutable_type,
    subclass
)]
pub struct PyQueryStatement;

#[inline]
#[optimize(speed)]
pub(crate) fn into_query_builder(
    object: &pyo3::Bound<'_, pyo3::PyAny>,
) -> pyo3::PyResult<Box<dyn sea_query::QueryBuilder>> {
    let val = unsafe {
        if pyo3::ffi::PyUnicode_CheckExact(object.as_ptr()) == 0 {
            return Err(typeerror!("expected str, got {:?}", object.py(), object.as_ptr()));
        }

        let mut size: pyo3::ffi::Py_ssize_t = 0;
        let c_str = pyo3::ffi::PyUnicode_AsUTF8AndSize(object.as_ptr(), &mut size);

        if c_str.is_null() || size < 0 {
            return Err(pyo3::PyErr::fetch(object.py()));
        } else {
            std::ffi::CStr::from_ptr(c_str).to_string_lossy()
        }
    };

    if val == "sqlite" {
        Ok(Box::new(sea_query::SqliteQueryBuilder))
    } else if val == "mysql" {
        Ok(Box::new(sea_query::MysqlQueryBuilder))
    } else if val == "postgresql" || val == "postgres" {
        Ok(Box::new(sea_query::PostgresQueryBuilder))
    } else {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "invalid backend value, got {val}"
        )))
    }
}

#[inline]
#[optimize(speed)]
pub(crate) fn into_schema_builder(
    object: &pyo3::Bound<'_, pyo3::PyAny>,
) -> pyo3::PyResult<Box<dyn sea_query::SchemaBuilder>> {
    let val = unsafe {
        if pyo3::ffi::PyUnicode_CheckExact(object.as_ptr()) == 0 {
            return Err(typeerror!("expected str, got {:?}", object.py(), object.as_ptr()));
        }

        let mut size: pyo3::ffi::Py_ssize_t = 0;
        let c_str = pyo3::ffi::PyUnicode_AsUTF8AndSize(object.as_ptr(), &mut size);

        if c_str.is_null() || size < 0 {
            return Err(pyo3::PyErr::fetch(object.py()));
        } else {
            std::ffi::CStr::from_ptr(c_str).to_string_lossy()
        }
    };

    if val == "sqlite" {
        Ok(Box::new(sea_query::SqliteQueryBuilder))
    } else if val == "mysql" {
        Ok(Box::new(sea_query::MysqlQueryBuilder))
    } else if val == "postgresql" || val == "postgres" {
        Ok(Box::new(sea_query::PostgresQueryBuilder))
    } else {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "invalid backend value, got {val}"
        )))
    }
}
