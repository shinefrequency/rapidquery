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
        arg: &pyo3::Bound<'a, super::expr::PyExpr>,
    ) -> pyo3::PyRef<'a, Self> {
        let arg = arg.get().inner.clone();

        {
            let mut lock = slf.inner.lock();
            *lock = lock.clone().arg(arg);
        }

        slf
    }

    #[classmethod]
    fn min(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, expr: &pyo3::Bound<'_, super::expr::PyExpr>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::min(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn max(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, expr: &pyo3::Bound<'_, super::expr::PyExpr>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::max(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn abs(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, expr: &pyo3::Bound<'_, super::expr::PyExpr>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::abs(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn avg(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, expr: &pyo3::Bound<'_, super::expr::PyExpr>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::avg(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn count(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::count(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn count_distinct(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::count_distinct(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn if_null(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        a: &pyo3::Bound<'_, super::expr::PyExpr>,
        b: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::if_null(
                a.get().inner.clone(),
                b.get().inner.clone(),
            )),
        }
    }

    #[classmethod]
    fn greatest(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        exprs: Vec<pyo3::Py<super::expr::PyExpr>>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::greatest(
                exprs.into_iter().map(|x| x.get().inner.clone()),
            )),
        }
    }

    #[classmethod]
    fn least(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, exprs: Vec<pyo3::Py<super::expr::PyExpr>>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::least(
                exprs.into_iter().map(|x| x.get().inner.clone()),
            )),
        }
    }

    #[classmethod]
    fn char_length(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::char_length(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn coalesce(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        exprs: Vec<pyo3::Py<super::expr::PyExpr>>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::coalesce(
                exprs.into_iter().map(|x| x.get().inner.clone()),
            )),
        }
    }

    #[classmethod]
    fn lower(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::lower(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn upper(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::upper(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn bit_and(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::bit_and(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn bit_or(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        expr: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::bit_or(expr.get().inner.clone())),
        }
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
        expr: &pyo3::Bound<'_, super::expr::PyExpr>,
    ) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::round(expr.get().inner.clone())),
        }
    }

    #[classmethod]
    fn md5(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, expr: &pyo3::Bound<'_, super::expr::PyExpr>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Func::md5(expr.get().inner.clone())),
        }
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
