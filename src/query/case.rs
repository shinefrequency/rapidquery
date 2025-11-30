#[derive(Default)]
pub struct CaseInner {
    // Always is `Vec<(PyExpr, PyExpr)>`
    when: Vec<(pyo3::Py<pyo3::PyAny>, pyo3::Py<pyo3::PyAny>)>,
    // Always is `Option<PyExpr>`
    r#else: Option<pyo3::Py<pyo3::PyAny>>,
}

impl CaseInner {
    #[inline]
    pub fn as_statement(&self, py: pyo3::Python) -> sea_query::CaseStatement {
        let mut stmt = sea_query::CaseStatement::new();

        for (cond, then) in &self.when {
            let cond = unsafe { cond.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            let then = unsafe { then.cast_bound_unchecked::<crate::expression::PyExpr>(py) };

            stmt = stmt.case(cond.get().inner.clone(), then.get().inner.clone());
        }

        if let Some(x) = &self.r#else {
            let x = unsafe { x.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            stmt = stmt.finally(x.get().inner.clone());
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "Case", frozen)]
pub struct PyCase {
    pub inner: parking_lot::Mutex<CaseInner>,
}

#[pyo3::pymethods]
impl PyCase {
    #[new]
    fn new() -> Self {
        Self {
            inner: parking_lot::Mutex::new(Default::default()),
        }
    }

    fn when<'a>(
        slf: pyo3::PyRef<'a, Self>,
        cond: pyo3::Bound<'a, pyo3::PyAny>,
        then: pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let cond = crate::expression::PyExpr::from_bound_into_any(cond)?;
        let then = crate::expression::PyExpr::from_bound_into_any(then)?;

        {
            let mut lock = slf.inner.lock();
            lock.when.push((cond, then));
        }

        Ok(slf)
    }

    fn else_<'a>(
        slf: pyo3::PyRef<'a, Self>,
        expr: pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let expr = crate::expression::PyExpr::from_bound_into_any(expr)?;

        {
            let mut lock = slf.inner.lock();
            lock.r#else = Some(expr);
        }

        Ok(slf)
    }

    fn to_expr(&self, py: pyo3::Python) -> crate::expression::PyExpr {
        let stmt = {
            let lock = self.inner.lock();
            lock.as_statement(py)
        };

        crate::expression::PyExpr::from(sea_query::SimpleExpr::Case(Box::new(stmt)))
    }
}
