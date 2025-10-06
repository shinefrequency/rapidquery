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
