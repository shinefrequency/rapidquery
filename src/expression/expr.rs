use pyo3::types::PyAnyMethods;
use sea_query::QueryBuilder;

#[pyo3::pyclass(module = "rapidquery._lib", name = "Expr", frozen)]
pub struct PyExpr {
    pub(crate) inner: parking_lot::Mutex<sea_query::SimpleExpr>,
}

impl PyExpr {
    #[inline]
    #[optimize(speed)]
    fn from_adapted_value(py: pyo3::Python, value: &crate::adaptation::PyAdaptedValue) -> Self {
        let simple_expr = {
            let mut lock = value.inner.lock();
            lock.create_simple_expr(py)
        };

        Self {
            inner: parking_lot::Mutex::new(simple_expr),
        }
    }

    #[inline]
    #[optimize(speed)]
    fn from_function_call(value: &super::function::PyFunctionCall) -> Self {
        let simple_expr = {
            let lock = value.inner.lock();
            sea_query::SimpleExpr::FunctionCall(lock.clone())
        };

        Self {
            inner: parking_lot::Mutex::new(simple_expr),
        }
    }

    #[inline]
    #[optimize(speed)]
    fn from_column_ref(value: sea_query::ColumnRef) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Expr::column(value)),
        }
    }

    #[inline]
    #[optimize(speed)]
    fn from_tuple<I>(values: I) -> Self
    where
        I: IntoIterator<Item = sea_query::SimpleExpr>,
    {
        let values = sea_query::Expr::tuple(values);

        Self {
            inner: parking_lot::Mutex::new(values.into()),
        }
    }
}

impl TryFrom<pyo3::Bound<'_, pyo3::PyAny>> for PyExpr {
    type Error = pyo3::PyErr;

    fn try_from(value: pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, Self::Error> {
        use sea_query::IntoColumnRef;

        unsafe {
            let type_ptr = pyo3::ffi::Py_TYPE(value.as_ptr());

            if type_ptr == crate::typeref::ADAPTED_VALUE_TYPE {
                let value = value.cast_into_unchecked::<crate::adaptation::PyAdaptedValue>();

                Ok(Self::from_adapted_value(value.py(), value.get()))
            } else if type_ptr == crate::typeref::ASTERISK_TYPE {
                Ok(Self {
                    inner: parking_lot::Mutex::new(sea_query::Expr::column(sea_query::Asterisk)),
                })
            } else if type_ptr == crate::typeref::COLUMN_REF_TYPE {
                let value = value.cast_into_unchecked::<crate::common::PyColumnRef>();

                Ok(Self::from_column_ref(value.get().clone().into_column_ref()))
            } else if type_ptr == crate::typeref::FUNCTION_CALL_TYPE {
                let value = value.cast_into_unchecked::<super::function::PyFunctionCall>();

                Ok(Self::from_function_call(value.get()))
            } else if pyo3::ffi::PyTuple_Check(value.as_ptr()) == 1 {
                let value = value.cast_into_unchecked::<pyo3::types::PyTuple>();
                let mut arr: Vec<Self> = Vec::new();

                for x in value {
                    arr.push(Self::try_from(x)?);
                }

                Ok(Self::from_tuple(arr.into_iter().map(|x| x.inner.lock().clone())))
            } else if type_ptr == crate::typeref::EXPR_TYPE {
                let value = value.cast_into_unchecked::<Self>();
                let x = value.get().inner.lock();

                Ok(Self { inner: parking_lot::Mutex::new(x.clone()) })
            } else {
                let py = value.py();
                let mut value = crate::adaptation::ReturnableValue::from_bound(value, None)?;

                Ok(Self {
                    inner: parking_lot::Mutex::new(value.create_simple_expr(py)),
                })
            }
        }
    }
}

#[pyo3::pymethods]
impl PyExpr {
    #[new]
    #[pyo3(signature=(value, /))]
    fn __new__(value: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        // Go fast path when value is PyExpr
        if let Ok(x) = value.cast::<Self>() {
            return Ok(pyo3::PyClassInitializer::from(x.clone()));
        }

        Ok(pyo3::PyClassInitializer::from(Self::try_from(value)?))
    }

    #[classmethod]
    fn val(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, value: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) != crate::typeref::ADAPTED_VALUE_TYPE {
                return Err(typeerror!(
                    "expected AdaptedValue, got {}",
                    value.py(),
                    value.as_ptr()
                ));
            }

            let x = value.cast_unchecked::<crate::adaptation::PyAdaptedValue>();
            Ok(Self::from_adapted_value(value.py(), x.get()))
        }
    }

    #[classmethod]
    fn func(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, value: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) != crate::typeref::FUNCTION_CALL_TYPE {
                return Err(typeerror!(
                    "expected FunctionCall, got {}",
                    value.py(),
                    value.as_ptr()
                ));
            }

            let x = value.cast_unchecked::<super::function::PyFunctionCall>();
            Ok(Self::from_function_call(x.get()))
        }
    }

    #[classmethod]
    fn col(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, value: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        use sea_query::IntoColumnRef;
        use std::str::FromStr;

        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::COLUMN_REF_TYPE {
                let x = value.cast_unchecked::<crate::common::PyColumnRef>();
                return Ok(Self::from_column_ref(x.get().clone().into_column_ref()));
            }

            if pyo3::ffi::PyUnicode_CheckExact(value.as_ptr()) == 1 {
                let x = value.extract::<&str>().unwrap_unchecked();
                let colref = crate::common::PyColumnRef::from_str(x)?.into_column_ref();
                return Ok(Self::from_column_ref(colref));
            }

            Err(typeerror!("expected ColumnRef or str, got {}", value.py(), value.as_ptr()))
        }
    }

    #[classmethod]
    fn tuple(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        value: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyTuple_CheckExact(value.as_ptr()) == 1 {
                let value = value.cast_unchecked::<pyo3::types::PyTuple>();
                let mut values: Vec<pyo3::Bound<'_, Self>> = Vec::with_capacity(value.len()?);

                for op in value {
                    values.push(op.cast_into_exact::<Self>()?);
                }

                return Ok(Self::from_tuple(
                    values.into_iter().map(|x| x.get().inner.lock().clone()),
                ));
            }

            if pyo3::ffi::PyList_CheckExact(value.as_ptr()) == 1 {
                let value = value.cast_unchecked::<pyo3::types::PyList>();
                let mut values: Vec<pyo3::Bound<'_, Self>> = Vec::with_capacity(value.len()?);

                for op in value {
                    values.push(op.cast_into_exact::<Self>()?);
                }

                return Ok(Self::from_tuple(
                    values.into_iter().map(|x| x.get().inner.lock().clone()),
                ));
            }

            if pyo3::ffi::PySet_CheckExact(value.as_ptr()) == 1 {
                let value = value.cast_unchecked::<pyo3::types::PyList>();
                let mut values: Vec<pyo3::Bound<'_, Self>> = Vec::with_capacity(value.len()?);

                for op in value {
                    values.push(op.cast_into_exact::<Self>()?);
                }

                return Ok(Self::from_tuple(
                    values.into_iter().map(|x| x.get().inner.lock().clone()),
                ));
            }

            Err(typeerror!(
                "expected tuple/list/set of Exprs, got {}",
                value.py(),
                value.as_ptr()
            ))
        }
    }

    #[classmethod]
    fn asterisk(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::Expr::column(sea_query::Asterisk)),
        }
    }

    #[classmethod]
    fn custom(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, value: String) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::SimpleExpr::Custom(value)),
        }
    }

    #[classmethod]
    fn current_date(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentDate)),
        }
    }

    #[classmethod]
    fn current_time(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentTime)),
        }
    }

    #[classmethod]
    fn current_timestamp(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentTimestamp)),
        }
    }

    #[classmethod]
    fn null(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self {
            inner: parking_lot::Mutex::new(sea_query::SimpleExpr::Keyword(sea_query::Keyword::Null)),
        }
    }

    fn cast_as(slf: pyo3::PyRef<'_, Self>, value: String) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();

            let result = lock.clone().cast_as(sea_query::Alias::new(value));
            *lock = result;
        }

        slf
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn like(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> pyo3::PyRef<'_, Self> {
        {
            let e = sea_query::LikeExpr::new(pattern);

            let mut lock = slf.inner.lock();

            if let Some(x) = escape {
                *lock = sea_query::ExprTrait::like(lock.clone(), e.escape(x));
            } else {
                *lock = sea_query::ExprTrait::like(lock.clone(), e);
            }
        }

        slf
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn not_like(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> pyo3::PyRef<'_, Self> {
        {
            let e = sea_query::LikeExpr::new(pattern);

            let mut lock = slf.inner.lock();

            if let Some(x) = escape {
                *lock = sea_query::ExprTrait::not_like(lock.clone(), e.escape(x));
            } else {
                *lock = sea_query::ExprTrait::not_like(lock.clone(), e);
            }
        }

        slf
    }

    fn __eq__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, Self>) -> pyo3::PyRef<'a, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::eq(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::ne(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __gt__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::gt(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __ge__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::gte(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __lt__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::lt(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __le__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::lte(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __add__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::add(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __sub__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::sub(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __and__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::bit_and(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __or__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::bit_or(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __truediv__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::div(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn is_(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::is(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn is_not(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::is_not(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn is_null(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::is_null(lock.clone());
            *lock = result;
        }

        slf
    }

    fn is_not_null(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::is_not_null(lock.clone());
            *lock = result;
        }

        slf
    }

    fn __lshift__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::left_shift(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __rshift__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::right_shift(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __mod__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::modulo(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn __mul__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::mul(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn sqlite_matches(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::extension::sqlite::SqliteExpr::matches(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn sqlite_glob(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::extension::sqlite::SqliteExpr::glob(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn pg_concat(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::extension::postgres::PgExpr::concat(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn pg_contained(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::extension::postgres::PgExpr::contained(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn cast_json_field(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::extension::postgres::PgExpr::cast_json_field(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn pg_contains(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::extension::postgres::PgExpr::contains(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn pg_matches(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::extension::postgres::PgExpr::matches(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    fn get_json_field(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let other = other.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::extension::postgres::PgExpr::get_json_field(lock.clone(), other);
            *lock = result;
        }

        slf
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn pg_ilike(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> pyo3::PyRef<'_, Self> {
        {
            let e = sea_query::LikeExpr::new(pattern);

            let mut lock = slf.inner.lock();

            if let Some(x) = escape {
                *lock = sea_query::extension::postgres::PgExpr::ilike(lock.clone(), e.escape(x));
            } else {
                *lock = sea_query::extension::postgres::PgExpr::ilike(lock.clone(), e);
            }
        }

        slf
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn pg_not_ilike(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> pyo3::PyRef<'_, Self> {
        {
            let e = sea_query::LikeExpr::new(pattern);

            let mut lock = slf.inner.lock();

            if let Some(x) = escape {
                *lock = sea_query::extension::postgres::PgExpr::not_ilike(lock.clone(), e.escape(x));
            } else {
                *lock = sea_query::extension::postgres::PgExpr::not_ilike(lock.clone(), e);
            }
        }

        slf
    }

    fn between(slf: pyo3::PyRef<'_, Self>, a: pyo3::Py<Self>, b: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let a = a.get().inner.lock().clone();
        let b = b.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::between(lock.clone(), a, b);
            *lock = result;
        }

        slf
    }

    fn not_between(slf: pyo3::PyRef<'_, Self>, a: pyo3::Py<Self>, b: pyo3::Py<Self>) -> pyo3::PyRef<'_, Self> {
        let a = a.get().inner.lock().clone();
        let b = b.get().inner.lock().clone();

        {
            let mut lock = slf.inner.lock();

            let result = sea_query::ExprTrait::not_between(lock.clone(), a, b);
            *lock = result;
        }

        slf
    }

    fn in_(slf: pyo3::PyRef<'_, Self>, expr: Vec<pyo3::Py<Self>>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();

            let result =
                sea_query::ExprTrait::is_in(lock.clone(), expr.into_iter().map(|x| x.get().inner.lock().clone()));
            *lock = result;
        }

        slf
    }

    fn not_in(slf: pyo3::PyRef<'_, Self>, expr: Vec<pyo3::Py<Self>>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();

            let result =
                sea_query::ExprTrait::is_not_in(lock.clone(), expr.into_iter().map(|x| x.get().inner.lock().clone()));
            *lock = result;
        }

        slf
    }

    fn __copy__(&self) -> Self {
        Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone()),
        }
    }

    fn copy(&self) -> Self {
        Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone()),
        }
    }

    fn to_sql(&self) -> String {
        let mut sql = String::with_capacity(10);
        let lock = self.inner.lock();

        sea_query::PostgresQueryBuilder.prepare_simple_expr_common(&lock, &mut sql);
        sql
    }

    fn __repr__(&self) -> String {
        let mut sql = String::from("<SimpleExpr ");
        let lock = self.inner.lock();

        sea_query::PostgresQueryBuilder.prepare_simple_expr_common(&lock, &mut sql);
        sql + ">"
    }
}
