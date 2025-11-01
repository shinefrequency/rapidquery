use pyo3::types::{PyStringMethods, PyTypeMethods};

/// Returns the type name of a [`pyo3::ffi::PyObject`].
///
/// Returns `"<unknown>"` on failure.
#[optimize(speed)]
pub unsafe fn get_type_name<'a>(py: pyo3::Python<'a>, obj: *mut pyo3::ffi::PyObject) -> String {
    let type_ = pyo3::ffi::Py_TYPE(obj);

    if std::hint::unlikely(type_.is_null()) {
        String::from("<unknown>")
    } else {
        let obj = unsafe { pyo3::types::PyType::from_borrowed_type_ptr(py, type_) };

        #[cfg(debug_assertions)]
        let name = obj.fully_qualified_name().unwrap();

        #[cfg(not(debug_assertions))]
        let name = obj.fully_qualified_name().unwrap_unchecked();

        name.to_string_lossy().into_owned()
    }
}

/// Creates a new [`pyo3::exceptions::PyTypeError`]
///
/// ```rust
/// typeerror!(
///     "expected str, got {}",
///     py,
///     value.as_ptr(),
/// )
///
/// typeerror!("type error description")
/// ```
#[macro_export]
macro_rules! typeerror {
    (
        $message:expr,
    ) => {
        pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>($message)
    };

    (
        $message:expr,
        $py:expr,
        $($ptr:expr),*
    ) => {
        pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            format!(
                $message,
                $(
                    unsafe { $crate::macros::get_type_name($py, $ptr) },
                )*
            )
        )
    };
}

#[macro_export]
macro_rules! prepare_sql {
    ($converter:expr => $backend:expr => $method:ident($value:expr, &mut $sql:expr)) => {{
        let builder = $converter($backend)?;

        let assert_unwind = std::panic::AssertUnwindSafe(|| builder.$method($value, &mut $sql));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))
    }};
}

#[macro_export]
macro_rules! build_schema {
    ($backend:expr => $build_func:ident($stmt:expr)) => {{
        let builder = $crate::backend::into_schema_builder($backend)?;

        let assert_unwind = std::panic::AssertUnwindSafe(|| $stmt.$build_func(&*builder));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))
    }};
}

#[macro_export]
macro_rules! build_query_parts {
    ($backend:expr => $build_func:ident($stmt:expr)) => {{
        let builder = $crate::backend::into_query_builder($backend)?;

        let (placeholder, numbered) = builder.placeholder();
        let mut sql = sea_query::SqlWriterValues::new(placeholder, numbered);

        let assert_unwind =
            std::panic::AssertUnwindSafe(|| $stmt.build_collect_any_into(&*builder, &mut sql));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))?;

        let (sql, values) = sql.into_parts();

        let values = {
            values
                .into_iter()
                .map(|x| $crate::adaptation::SerializedValue::from(x))
                .map(|x| $crate::adaptation::ReturnableValue::from(x))
                .map(|x| $crate::adaptation::PyAdaptedValue::from(x))
        };

        unsafe {
            let tuple_ptr = pyo3::ffi::PyTuple_New(values.len() as isize);
            if tuple_ptr.is_null() {
                return Err(pyo3::PyErr::fetch($backend.py()));
            }

            for (index, key) in values.enumerate() {
                let key = pyo3::Py::new($backend.py(), key).unwrap().into_ptr();

                if pyo3::ffi::PyTuple_SetItem(tuple_ptr, index as isize, key) == -1 {
                    pyo3::ffi::Py_XDECREF(tuple_ptr);
                    pyo3::ffi::Py_XDECREF(key);
                    return Err(pyo3::PyErr::fetch($backend.py()));
                }
            }

            Ok((sql, pyo3::Py::from_owned_ptr($backend.py(), tuple_ptr)))
        }
    }};
}

#[macro_export]
macro_rules! build_query_string {
    ($backend:expr => $build_func:ident($stmt:expr)) => {{
        let builder = $crate::backend::into_query_builder($backend)?;

        let mut sql = String::with_capacity(255);

        let assert_unwind =
            std::panic::AssertUnwindSafe(|| $stmt.build_collect_any_into(&*builder, &mut sql));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))?;

        Ok(sql)
    }};
}
