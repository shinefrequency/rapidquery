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
        $message:literal,
    ) => {
        pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>($message)
    };

    (
        $message:literal,
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
        let builder = match $converter($backend) {
            Some(x) => x,
            None => {
                return Err(typeerror!(
                    "expected BackendMeta, got {}",
                    $backend.py(),
                    $backend.as_ptr()
                ))
            }
        };

        let assert_unwind = std::panic::AssertUnwindSafe(|| builder.$method($value, &mut $sql));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))
    }};
}

#[macro_export]
macro_rules! build_schema {
    // ($build_func:ident($stmt:expr, $backend:expr), $backend:expr, $converter:expr, $build_func:ident) => {{
    ($converter:expr => $backend:expr => $build_func:ident($stmt:expr)) => {{
        let builder = match $converter($backend) {
            Some(x) => x,
            None => {
                return Err(typeerror!(
                    "expected BackendMeta, got {}",
                    $backend.py(),
                    $backend.as_ptr()
                ))
            }
        };

        let assert_unwind = std::panic::AssertUnwindSafe(|| $stmt.$build_func(&*builder));

        std::panic::catch_unwind(assert_unwind)
            .map_err(|_| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("build failed"))
    }};
}
