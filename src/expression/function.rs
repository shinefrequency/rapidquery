use pyo3::types::PyTupleMethods;

/// Represents a SQL function call that can be used in expressions.
///
/// A bridge between Python & [`sea_query::FunctionCall`]
#[pyo3::pyclass(module = "rapidquery._lib", name = "FunctionCall", frozen)]
pub struct PyFunctionCall {
    pub inner: parking_lot::Mutex<sea_query::FunctionCall>,
}

#[pyo3::pymethods]
impl PyFunctionCall {
    #[new]
    #[pyo3(signature=(name))]
    pub fn new(name: String) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::cust(sea_query::Alias::new(name))),
        }
    }

    pub fn arg<'a>(
        slf: pyo3::PyRef<'a, Self>,
        arg: pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let arg = super::PyExpr::try_from(arg)?;

        {
            let mut lock = slf.inner.lock();
            *lock = lock.clone().arg(arg.inner);
        }

        Ok(slf)
    }

    #[classmethod]
    fn now(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> pyo3::PyResult<Self> {
        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::cust(sea_query::Alias::new("NOW"))),
        })
    }

    #[classmethod]
    fn sum(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::sum(expr.inner)),
        })
    }

    #[classmethod]
    fn min(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::min(expr.inner)),
        })
    }

    #[classmethod]
    fn max(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::max(expr.inner)),
        })
    }

    #[classmethod]
    fn abs(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::abs(expr.inner)),
        })
    }

    #[classmethod]
    fn avg(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::avg(expr.inner)),
        })
    }

    #[classmethod]
    fn count(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::count(expr.inner)),
        })
    }

    #[classmethod]
    fn count_distinct(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::count_distinct(expr.inner)),
        })
    }

    #[classmethod]
    fn if_null(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        a: pyo3::Bound<'_, pyo3::PyAny>,
        b: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let a = super::PyExpr::try_from(a)?;
        let b = super::PyExpr::try_from(b)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::if_null(a.inner, b.inner)),
        })
    }

    #[classmethod]
    #[pyo3(signature=(*exprs))]
    fn greatest(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        exprs: &pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<Self> {
        let mut simple_exprs = Vec::with_capacity(exprs.len());

        for expr in exprs.iter() {
            let expr = super::PyExpr::try_from(expr)?;
            simple_exprs.push(expr.inner);
        }

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::greatest(simple_exprs)),
        })
    }

    #[classmethod]
    #[pyo3(signature=(*exprs))]
    fn least(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        exprs: &pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<Self> {
        let mut simple_exprs = Vec::with_capacity(exprs.len());

        for expr in exprs.iter() {
            let expr = super::PyExpr::try_from(expr)?;
            simple_exprs.push(expr.inner);
        }

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::least(simple_exprs)),
        })
    }

    #[classmethod]
    fn char_length(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::char_length(expr.inner)),
        })
    }

    #[classmethod]
    #[pyo3(signature=(*exprs))]
    fn coalesce(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        exprs: &pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<Self> {
        let mut simple_exprs = Vec::with_capacity(exprs.len());

        for expr in exprs.iter() {
            let expr = super::PyExpr::try_from(expr)?;
            simple_exprs.push(expr.inner);
        }

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::coalesce(simple_exprs)),
        })
    }

    #[classmethod]
    fn lower(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::lower(expr.inner)),
        })
    }

    #[classmethod]
    fn upper(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::upper(expr.inner)),
        })
    }

    #[classmethod]
    fn bit_and(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::bit_and(expr.inner)),
        })
    }

    #[classmethod]
    fn bit_or(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::bit_or(expr.inner)),
        })
    }

    #[classmethod]
    fn random(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::random()),
        }
    }

    #[classmethod]
    fn round(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::round(expr.inner)),
        })
    }

    #[classmethod]
    fn round_with_precision(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        a: pyo3::Bound<'_, pyo3::PyAny>,
        b: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let a = super::PyExpr::try_from(a)?;
        let b = super::PyExpr::try_from(b)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::round_with_precision(a.inner, b.inner)),
        })
    }

    #[classmethod]
    fn md5(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let expr = super::PyExpr::try_from(expr)?;

        Ok(Self {
            inner: parking_lot::Mutex::new(sea_query::Func::md5(expr.inner)),
        })
    }

    fn to_expr(&self) -> crate::expression::PyExpr {
        let lock = self.inner.lock();
        crate::expression::PyExpr::from(sea_query::SimpleExpr::FunctionCall(lock.clone()))
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();

        let mut sql = String::new();

        prepare_sql!(
            crate::backend::into_query_builder => backend => prepare_function_name(&lock.get_func(), &mut sql)
        )?;
        prepare_sql!(
            crate::backend::into_query_builder => backend => prepare_function_arguments(&lock, &mut sql)
        )?;

        Ok(sql)
    }

    fn __repr__(&self) -> String {
        let lock = self.inner.lock();
        format!("<FunctionCall {:?}>", lock)
    }
}
