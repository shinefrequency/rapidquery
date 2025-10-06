mod expr;
mod function;

pub use expr::PyExpr;
pub use function::PyFunctionCall;

#[pyo3::pyfunction]
#[pyo3(signature=(arg1, *args))]
pub fn all(
    arg1: pyo3::Bound<'_, PyExpr>,
    args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
    let py = arg1.py();
    let mut expr = arg1.unbind();

    for m in args {
        let m = m.cast_into_exact::<PyExpr>()?;

        let result = sea_query::ExprTrait::and(expr.get().inner.lock().clone(), m.get().inner.lock().clone());
        expr = pyo3::Py::new(
            py,
            PyExpr {
                inner: parking_lot::Mutex::new(result),
            },
        )?;
    }

    Ok(expr.into_any())
}

#[pyo3::pyfunction]
#[pyo3(signature=(arg1, *args))]
pub fn any(
    arg1: pyo3::Bound<'_, PyExpr>,
    args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
    let py = arg1.py();
    let mut expr = arg1.unbind();

    for m in args {
        let m = m.cast_into_exact::<PyExpr>()?;

        let result = sea_query::ExprTrait::or(expr.get().inner.lock().clone(), m.get().inner.lock().clone());
        expr = pyo3::Py::new(
            py,
            PyExpr {
                inner: parking_lot::Mutex::new(result),
            },
        )?;
    }

    Ok(expr.into_any())
}
