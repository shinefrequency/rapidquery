use pyo3::types::PyAnyMethods;

/// Represents a SQL expression that can be built into SQL code.
///
/// A bridge between Python & [`sea_query::SimpleExpr`]
#[pyo3::pyclass(module = "rapidquery._lib", name = "Expr", frozen)]
pub struct PyExpr {
    // TOD: support subquery and case
    pub(crate) inner: sea_query::SimpleExpr,
}

impl From<sea_query::SimpleExpr> for PyExpr {
    fn from(value: sea_query::SimpleExpr) -> Self {
        Self { inner: value }
    }
}

impl PyExpr {
    #[inline]
    #[optimize(speed)]
    pub fn from_bound_into_any(x: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(x.as_ptr()) == crate::typeref::EXPR_TYPE {
                Ok(x.unbind())
            } else {
                let py = x.py();
                let e = Self::try_from(x)?;
                Ok(pyo3::Py::new(py, e)?.into_any())
            }
        }
    }

    #[inline]
    #[optimize(speed)]
    pub fn from_adapted_value(py: pyo3::Python, value: &crate::adaptation::PyAdaptedValue) -> Self {
        let simple_expr = {
            let mut lock = value.inner.lock();
            lock.create_simple_expr(py)
        };

        simple_expr.into()
    }

    #[inline]
    #[optimize(speed)]
    pub fn from_function_call(value: &super::function::PyFunctionCall) -> Self {
        let simple_expr = {
            let lock = value.inner.lock();
            sea_query::SimpleExpr::FunctionCall(lock.clone())
        };

        simple_expr.into()
    }

    #[inline]
    #[optimize(speed)]
    pub fn from_column_ref(value: sea_query::ColumnRef) -> Self {
        sea_query::Expr::column(value).into()
    }

    #[inline]
    #[optimize(speed)]
    pub fn from_simple_expr(value: sea_query::SimpleExpr) -> Self {
        Self { inner: value }
    }

    #[inline]
    #[optimize(speed)]
    pub fn from_tuple<I>(values: I) -> Self
    where
        I: IntoIterator<Item = sea_query::SimpleExpr>,
    {
        let values = sea_query::Expr::tuple(values);

        Self { inner: values.into() }
    }

    pub fn try_with_specific_type(
        value: pyo3::Bound<'_, pyo3::PyAny>,
        r#type: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
    ) -> pyo3::PyResult<Self> {
        use sea_query::IntoColumnRef;

        unsafe {
            let type_ptr = pyo3::ffi::Py_TYPE(value.as_ptr());

            if type_ptr == crate::typeref::EXPR_TYPE {
                let value = value.cast_into_unchecked::<Self>();

                Ok(Self {
                    inner: value.get().inner.clone(),
                })
            } else if type_ptr == crate::typeref::ADAPTED_VALUE_TYPE {
                let value = value.cast_into_unchecked::<crate::adaptation::PyAdaptedValue>();

                Ok(Self::from_adapted_value(value.py(), value.get()))
            } else if type_ptr == crate::typeref::ASTERISK_TYPE {
                Ok(Self {
                    inner: sea_query::Expr::column(sea_query::Asterisk),
                })
            } else if type_ptr == crate::typeref::COLUMN_REF_TYPE {
                let value = value.cast_into_unchecked::<crate::common::PyColumnRef>();

                Ok(Self::from_column_ref(value.get().clone().into_column_ref()))
            } else if type_ptr == crate::typeref::COLUMN_TYPE {
                let value = value.cast_into_unchecked::<crate::column::PyColumn>();
                let mut lock = value.get().inner.lock();

                Ok(Self::from_column_ref(lock.as_column_ref(value.py())))
            } else if type_ptr == crate::typeref::FUNCTION_CALL_TYPE {
                let value = value.cast_into_unchecked::<super::function::PyFunctionCall>();

                Ok(Self::from_function_call(value.get()))
            } else if type_ptr == crate::typeref::SELECT_STATEMENT_TYPE {
                let value = value.cast_into_unchecked::<crate::query::select::PySelect>();
                let stmt = value.get().inner.lock();
                let stmt = stmt.as_statement(value.py());

                Ok(Self::from_simple_expr(sea_query::SimpleExpr::SubQuery(
                    None,
                    Box::new(stmt.into_sub_query_statement()),
                )))
            } else if pyo3::ffi::PyTuple_Check(value.as_ptr()) == 1 {
                let value = value.cast_into_unchecked::<pyo3::types::PyTuple>();
                let mut arr: Vec<Self> = Vec::new();

                for x in value {
                    arr.push(Self::try_from(x)?);
                }

                Ok(Self::from_tuple(arr.into_iter().map(|x| x.inner.clone())))
            } else {
                let py = value.py();
                let mut value = crate::adaptation::ReturnableValue::from_bound(value, r#type)?;

                Ok(value.create_simple_expr(py).into())
            }
        }
    }
}

impl TryFrom<pyo3::Bound<'_, pyo3::PyAny>> for PyExpr {
    type Error = pyo3::PyErr;

    fn try_from(value: pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, Self::Error> {
        Self::try_with_specific_type(value, None)
    }
}

#[pyo3::pymethods]
impl PyExpr {
    #[new]
    #[pyo3(signature=(value, /))]
    fn new(value: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        // Go fast path when value is PyExpr
        if let Ok(x) = value.cast::<Self>() {
            return Ok(pyo3::PyClassInitializer::from(x.clone()));
        }

        Ok(pyo3::PyClassInitializer::from(Self::try_from(value)?))
    }

    #[classmethod]
    fn val(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        value: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
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
    fn func(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        value: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
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
    fn col(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        value: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
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

            Err(typeerror!(
                "expected ColumnRef or str, got {}",
                value.py(),
                value.as_ptr()
            ))
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
                    values.into_iter().map(|x| x.get().inner.clone()),
                ));
            }

            if pyo3::ffi::PyList_CheckExact(value.as_ptr()) == 1 {
                let value = value.cast_unchecked::<pyo3::types::PyList>();
                let mut values: Vec<pyo3::Bound<'_, Self>> = Vec::with_capacity(value.len()?);

                for op in value {
                    values.push(op.cast_into_exact::<Self>()?);
                }

                return Ok(Self::from_tuple(
                    values.into_iter().map(|x| x.get().inner.clone()),
                ));
            }

            if pyo3::ffi::PySet_CheckExact(value.as_ptr()) == 1 {
                let value = value.cast_unchecked::<pyo3::types::PyList>();
                let mut values: Vec<pyo3::Bound<'_, Self>> = Vec::with_capacity(value.len()?);

                for op in value {
                    values.push(op.cast_into_exact::<Self>()?);
                }

                return Ok(Self::from_tuple(
                    values.into_iter().map(|x| x.get().inner.clone()),
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
        sea_query::Expr::column(sea_query::Asterisk).into()
    }

    #[classmethod]
    fn custom(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, value: String) -> Self {
        sea_query::SimpleExpr::Custom(value).into()
    }

    #[classmethod]
    fn current_date(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentDate).into()
    }

    #[classmethod]
    fn current_time(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentTime).into()
    }

    #[classmethod]
    fn current_timestamp(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::CurrentTimestamp).into()
    }

    #[classmethod]
    fn null(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        sea_query::SimpleExpr::Keyword(sea_query::Keyword::Null).into()
    }

    #[classmethod]
    fn exists(
        cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        stmt: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(stmt.as_ptr()) != crate::typeref::SELECT_STATEMENT_TYPE {
                return Err(typeerror!("expected Select, got {}", stmt.py(), stmt.as_ptr()));
            }

            let stmt = {
                let val = stmt.cast_unchecked::<crate::query::select::PySelect>();
                let lock = val.get().inner.lock();
                lock.as_statement(cls.py())
            };

            Ok(sea_query::Expr::exists(stmt).into())
        }
    }

    #[classmethod]
    fn any(
        cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        stmt: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(stmt.as_ptr()) != crate::typeref::SELECT_STATEMENT_TYPE {
                return Err(typeerror!("expected Select, got {}", stmt.py(), stmt.as_ptr()));
            }

            let stmt = {
                let val = stmt.cast_unchecked::<crate::query::select::PySelect>();
                let lock = val.get().inner.lock();
                lock.as_statement(cls.py())
            };

            Ok(sea_query::Expr::any(stmt).into())
        }
    }

    #[classmethod]
    fn some(
        cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        stmt: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(stmt.as_ptr()) != crate::typeref::SELECT_STATEMENT_TYPE {
                return Err(typeerror!("expected Select, got {}", stmt.py(), stmt.as_ptr()));
            }

            let stmt = {
                let val = stmt.cast_unchecked::<crate::query::select::PySelect>();
                let lock = val.get().inner.lock();
                lock.as_statement(cls.py())
            };

            Ok(sea_query::Expr::some(stmt).into())
        }
    }

    #[classmethod]
    fn all(
        cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        stmt: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(stmt.as_ptr()) != crate::typeref::SELECT_STATEMENT_TYPE {
                return Err(typeerror!("expected Select, got {}", stmt.py(), stmt.as_ptr()));
            }

            let stmt = {
                let val = stmt.cast_unchecked::<crate::query::select::PySelect>();
                let lock = val.get().inner.lock();
                lock.as_statement(cls.py())
            };

            Ok(sea_query::Expr::all(stmt).into())
        }
    }

    fn cast_as(slf: pyo3::PyRef<'_, Self>, value: String) -> Self {
        slf.inner.clone().cast_as(sea_query::Alias::new(value)).into()
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn like(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> Self {
        let e = sea_query::LikeExpr::new(pattern);

        if let Some(x) = escape {
            sea_query::ExprTrait::like(slf.inner.clone(), e.escape(x)).into()
        } else {
            sea_query::ExprTrait::like(slf.inner.clone(), e).into()
        }
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn not_like(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> Self {
        let e = sea_query::LikeExpr::new(pattern);

        if let Some(x) = escape {
            sea_query::ExprTrait::not_like(slf.inner.clone(), e.escape(x)).into()
        } else {
            sea_query::ExprTrait::not_like(slf.inner.clone(), e).into()
        }
    }

    fn __eq__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::eq(slf.inner.clone(), other.inner).into())
    }

    fn __ne__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::ne(slf.inner.clone(), other.inner).into())
    }

    fn __gt__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::gt(slf.inner.clone(), other.inner).into())
    }

    fn __ge__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::gte(slf.inner.clone(), other.inner).into())
    }

    fn __lt__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::lt(slf.inner.clone(), other.inner).into())
    }

    fn __le__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::lte(slf.inner.clone(), other.inner).into())
    }

    fn __add__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::add(slf.inner.clone(), other.inner).into())
    }

    fn __sub__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::sub(slf.inner.clone(), other.inner).into())
    }

    fn __and__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::and(slf.inner.clone(), other.inner).into())
    }

    fn __or__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::or(slf.inner.clone(), other.inner).into())
    }

    fn bit_and<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::bit_and(slf.inner.clone(), other.inner).into())
    }

    fn bit_or<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::bit_or(slf.inner.clone(), other.inner).into())
    }

    fn __truediv__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::div(slf.inner.clone(), other.inner).into())
    }

    fn is_<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::is(slf.inner.clone(), other.inner).into())
    }

    fn is_not<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::is_not(slf.inner.clone(), other.inner).into())
    }

    fn is_null(slf: pyo3::PyRef<'_, Self>) -> Self {
        sea_query::ExprTrait::is_null(slf.inner.clone()).into()
    }

    fn is_not_null(slf: pyo3::PyRef<'_, Self>) -> Self {
        sea_query::ExprTrait::is_not_null(slf.inner.clone()).into()
    }

    fn __lshift__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::left_shift(slf.inner.clone(), other.inner).into())
    }

    fn __rshift__<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::right_shift(slf.inner.clone(), other.inner).into())
    }

    fn __mod__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::modulo(slf.inner.clone(), other.inner).into())
    }

    fn __mul__<'a>(slf: pyo3::PyRef<'a, Self>, other: &pyo3::Bound<'a, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::ExprTrait::mul(slf.inner.clone(), other.inner).into())
    }

    fn sqlite_matches<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::sqlite::SqliteExpr::matches(slf.inner.clone(), other.inner).into())
    }

    fn sqlite_glob<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::sqlite::SqliteExpr::glob(slf.inner.clone(), other.inner).into())
    }

    fn sqlite_get_json_field<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::sqlite::SqliteExpr::get_json_field(slf.inner.clone(), other.inner).into())
    }

    fn sqlite_cast_json_field<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::sqlite::SqliteExpr::cast_json_field(slf.inner.clone(), other.inner).into())
    }

    fn pg_concat<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::postgres::PgExpr::concat(slf.inner.clone(), other.inner).into())
    }

    fn pg_contained<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::postgres::PgExpr::contained(slf.inner.clone(), other.inner).into())
    }

    fn pg_get_json_field<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::postgres::PgExpr::get_json_field(slf.inner.clone(), other.inner).into())
    }

    fn pg_cast_json_field<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::postgres::PgExpr::cast_json_field(slf.inner.clone(), other.inner).into())
    }

    fn pg_contains<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::postgres::PgExpr::contains(slf.inner.clone(), other.inner).into())
    }

    fn pg_matches<'a>(
        slf: pyo3::PyRef<'a, Self>,
        other: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let other = Self::try_from(other.clone())?;
        Ok(sea_query::extension::postgres::PgExpr::matches(slf.inner.clone(), other.inner).into())
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn pg_ilike(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> Self {
        let e = sea_query::LikeExpr::new(pattern);

        if let Some(x) = escape {
            sea_query::extension::postgres::PgExpr::ilike(slf.inner.clone(), e.escape(x)).into()
        } else {
            sea_query::extension::postgres::PgExpr::ilike(slf.inner.clone(), e).into()
        }
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn pg_not_ilike(slf: pyo3::PyRef<'_, Self>, pattern: String, escape: Option<char>) -> Self {
        let e = sea_query::LikeExpr::new(pattern);

        if let Some(x) = escape {
            sea_query::extension::postgres::PgExpr::not_ilike(slf.inner.clone(), e.escape(x)).into()
        } else {
            sea_query::extension::postgres::PgExpr::not_ilike(slf.inner.clone(), e).into()
        }
    }

    fn between<'a>(
        slf: pyo3::PyRef<'a, Self>,
        a: &pyo3::Bound<'a, pyo3::PyAny>,
        b: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let a = Self::try_from(a.clone())?;
        let b = Self::try_from(b.clone())?;

        Ok(sea_query::ExprTrait::between(slf.inner.clone(), a.inner, b.inner).into())
    }

    fn not_between<'a>(
        slf: pyo3::PyRef<'a, Self>,
        a: &pyo3::Bound<'a, pyo3::PyAny>,
        b: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        let a = Self::try_from(a.clone())?;
        let b = Self::try_from(b.clone())?;

        Ok(sea_query::ExprTrait::not_between(slf.inner.clone(), a.inner, b.inner).into())
    }

    fn in_subquery(slf: pyo3::PyRef<'_, Self>, stmt: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(stmt.as_ptr()) != crate::typeref::SELECT_STATEMENT_TYPE {
                return Err(typeerror!("expected Select, got {}", stmt.py(), stmt.as_ptr()));
            }

            let stmt = {
                let val = stmt.cast_unchecked::<crate::query::select::PySelect>();
                let lock = val.get().inner.lock();
                lock.as_statement(slf.py())
            };

            Ok(sea_query::ExprTrait::in_subquery(slf.inner.clone(), stmt).into())
        }
    }

    fn not_in_subquery(
        slf: pyo3::PyRef<'_, Self>,
        stmt: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_TYPE(stmt.as_ptr()) != crate::typeref::SELECT_STATEMENT_TYPE {
                return Err(typeerror!("expected Select, got {}", stmt.py(), stmt.as_ptr()));
            }

            let stmt = {
                let val = stmt.cast_unchecked::<crate::query::select::PySelect>();
                let lock = val.get().inner.lock();
                lock.as_statement(slf.py())
            };

            Ok(sea_query::ExprTrait::not_in_subquery(slf.inner.clone(), stmt).into())
        }
    }

    fn in_(slf: pyo3::PyRef<'_, Self>, other: Vec<pyo3::Py<pyo3::PyAny>>) -> pyo3::PyResult<Self> {
        if other.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "`other` parameter cannot be an empty sequence",
            ));
        }

        let mut exprs = Vec::with_capacity(other.len());
        for exp in other.into_iter() {
            let exp = Self::try_from(exp.into_bound(slf.py()))?;
            exprs.push(exp.inner);
        }

        Ok(sea_query::ExprTrait::is_in(slf.inner.clone(), exprs).into())
    }

    fn not_in(slf: pyo3::PyRef<'_, Self>, other: Vec<pyo3::Py<pyo3::PyAny>>) -> pyo3::PyResult<Self> {
        if other.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "`other` parameter cannot be an empty sequence",
            ));
        }

        let mut exprs = Vec::with_capacity(other.len());
        for exp in other.into_iter() {
            let exp = Self::try_from(exp.into_bound(slf.py()))?;
            exprs.push(exp.inner);
        }

        Ok(sea_query::ExprTrait::is_not_in(slf.inner.clone(), exprs).into())
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let mut sql = String::new();

        prepare_sql!(
            crate::backend::into_query_builder => backend => prepare_simple_expr(&self.inner, &mut sql)
        )?;

        Ok(sql)
    }

    fn __repr__(&self) -> String {
        format!("<Expr {:?}>", self.inner)
    }
}
