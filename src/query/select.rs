use crate::backend::PyQueryStatement;
use pyo3::types::{PyAnyMethods, PyTupleMethods};
use sea_query::IntoIden;

#[pyo3::pyclass(module = "rapidquery._lib", name = "SelectExpr", frozen)]
pub struct PySelectExpr {
    // Always is `PyExpr`
    pub expr: pyo3::Py<pyo3::PyAny>,

    // Always is `PyExpr`
    pub alias: Option<String>,
    // TODO
    // pub window: pyo3::Py<pyo3::PyAny>,
}

// impl From<PySelectExpr> for sea_query::SelectExpr {
//     fn from(value: PySelectExpr) -> Self {
//         let expr = value.expr
//         sea_query::SelectExpr { expr: value.expr }
//     }
// }

impl PySelectExpr {
    pub fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            expr: self.expr.clone_ref(py),
            alias: self.alias.clone(),
        }
    }

    pub fn as_select_expr(&self, py: pyo3::Python) -> sea_query::SelectExpr {
        let expr = unsafe { self.expr.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
        let expr = expr.get().inner.clone();

        sea_query::SelectExpr {
            expr,
            alias: self.alias.as_ref().map(|x| sea_query::Alias::new(x).into_iden()),
            window: None,
        }
    }

    #[inline]
    #[optimize(speed)]
    pub fn from_bound_into_any(
        bound: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        use pyo3::PyTypeInfo;

        unsafe {
            if pyo3::ffi::Py_TYPE(bound.as_ptr()) == crate::typeref::EXPR_TYPE {
                let slf = Self {
                    expr: bound.clone().unbind(),
                    alias: None,
                };

                return pyo3::Py::new(bound.py(), slf).map(|x| x.into_any());
            }

            if PySelectExpr::is_exact_type_of(bound) {
                return Ok(bound.clone().unbind());
            }

            let expr = crate::expression::PyExpr::from_bound_into_any(bound.clone())?;
            let slf = Self { expr, alias: None };

            return pyo3::Py::new(bound.py(), slf).map(|x| x.into_any());
        }
    }
}

#[pyo3::pymethods]
impl PySelectExpr {
    #[new]
    #[pyo3(signature=(expr, alias=None))]
    fn new(expr: &pyo3::Bound<'_, pyo3::PyAny>, alias: Option<String>) -> pyo3::PyResult<pyo3::Py<Self>> {
        use pyo3::PyTypeInfo;

        if PySelectExpr::is_exact_type_of(expr) {
            let slf = unsafe { expr.clone().cast_into_unchecked::<Self>() };

            if let Some(x) = alias {
                let expr = slf.get().expr.clone_ref(slf.py());

                let new_slf = Self { expr, alias: Some(x) };
                Ok(pyo3::Py::new(slf.py(), new_slf).unwrap())
            } else {
                Ok(slf.unbind())
            }
        } else {
            let py = expr.py();
            let expr = crate::expression::PyExpr::from_bound_into_any(expr.clone())?;
            let slf = Self { expr, alias };

            Ok(pyo3::Py::new(py, slf).unwrap())
        }
    }
}

#[derive(Debug, Default)]
pub enum SelectDistinct {
    #[default]
    None,
    Distinct,
    DistinctOn(
        // Always is `Vec<ColumnRef | String>`
        Vec<pyo3::Py<pyo3::PyAny>>,
    ),
}

#[derive(Default)]
pub struct SelectInner {
    pub distinct: SelectDistinct,

    // TODO: support subquery
    // Always is `Vec<TableName>`
    pub table: Vec<pyo3::Py<pyo3::PyAny>>,

    // Always is `Option<SelectExpr>`
    pub cols: Vec<pyo3::Py<pyo3::PyAny>>,

    // Always is `Option<PyExpr>`
    pub r#where: Option<pyo3::Py<pyo3::PyAny>>,

    // TODO
    // pub join: Vec<pyo3::Py<pyo3::PyAny>>,
    // pub groups: Vec<pyo3::Py<pyo3::PyAny>>,
    // pub having: Vec<pyo3::Py<pyo3::PyAny>>,
    // pub unions: Vec<pyo3::Py<pyo3::PyAny>>,
    // pub lock: Option<pyo3::Py<pyo3::PyAny>>,
    // pub window: Option<pyo3::Py<pyo3::PyAny>>,
    // pub with: Option<pyo3::Py<pyo3::PyAny>>,
    pub orders: Vec<pyo3::Py<pyo3::PyAny>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl SelectInner {
    #[inline]
    fn as_statement(&self, py: pyo3::Python) -> sea_query::SelectStatement {
        let mut stmt = sea_query::SelectStatement::new();

        match &self.distinct {
            SelectDistinct::None => (),
            SelectDistinct::Distinct => {
                stmt.distinct();
            }
            SelectDistinct::DistinctOn(cols) => {
                use sea_query::IntoColumnRef;

                stmt.distinct_on(cols.iter().map(|col| unsafe {
                    if pyo3::ffi::PyUnicode_Check(col.as_ptr()) == 1 {
                        let x = sea_query::Alias::new(col.extract::<String>(py).unwrap_unchecked());
                        x.into_column_ref()
                    } else {
                        let x = col.cast_bound_unchecked::<crate::common::PyColumnRef>(py).get();
                        x.clone().into_column_ref()
                    }
                }));
            }
        }

        for table in self.table.iter() {
            let x = unsafe { table.cast_bound_unchecked::<crate::common::PyTableName>(py) };
            stmt.from(x.get().clone());
        }

        if !self.cols.is_empty() {
            stmt.exprs(self.cols.iter().map(|x| unsafe {
                let expr = x.cast_bound_unchecked::<PySelectExpr>(py);
                expr.get().as_select_expr(py)
            }));
        }

        if let Some(x) = &self.r#where {
            let x = unsafe { x.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            stmt.and_where(x.get().inner.clone());
        }

        if let Some(n) = self.limit {
            stmt.limit(n);
        }

        if let Some(n) = self.offset {
            stmt.offset(n);
        }

        for order in self.orders.iter() {
            let order = unsafe { order.cast_bound_unchecked::<super::order::PyOrder>(py) };
            let order = order.get();

            let target = unsafe { order.target.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            let target = target.get().inner.clone();

            if let Some(x) = order.null_order {
                stmt.order_by_expr_with_nulls(target, order.order.clone(), x);
            } else {
                stmt.order_by_expr(target, order.order.clone());
            }
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "Select", frozen, extends=PyQueryStatement)]
pub struct PySelect {
    pub inner: parking_lot::Mutex<SelectInner>,
}

#[pyo3::pymethods]
impl PySelect {
    #[new]
    #[pyo3(signature=(*cols))]
    fn new(cols: &pyo3::Bound<'_, pyo3::types::PyTuple>) -> pyo3::PyResult<(Self, PyQueryStatement)> {
        let mut exprs = Vec::with_capacity(PyTupleMethods::len(cols));

        for expr in PyTupleMethods::iter(cols) {
            exprs.push(PySelectExpr::from_bound_into_any(&expr)?);
        }

        let slf = Self {
            inner: parking_lot::Mutex::new(SelectInner {
                cols: exprs,
                ..Default::default()
            }),
        };

        Ok((slf, PyQueryStatement))
    }

    #[pyo3(signature=(*on))]
    fn distinct<'a>(
        slf: pyo3::PyRef<'a, Self>,
        on: &'a pyo3::Bound<'a, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if PyTupleMethods::is_empty(on) {
            let mut lock = slf.inner.lock();
            lock.distinct = SelectDistinct::Distinct;
        } else {
            let mut cols = Vec::with_capacity(PyTupleMethods::len(on));

            for col in PyTupleMethods::iter(on) {
                unsafe {
                    let col_ptr = pyo3::ffi::Py_TYPE(col.as_ptr());

                    if col_ptr == crate::typeref::COLUMN_TYPE {
                        let col = col.cast_into_unchecked::<crate::column::PyColumn>();
                        let col_ref = col.get().inner.lock().as_column_ref(slf.py());
                        cols.push(
                            pyo3::Py::new(slf.py(), crate::common::PyColumnRef::from(col_ref))
                                .unwrap()
                                .into_any(),
                        );
                    } else if col_ptr == crate::typeref::COLUMN_REF_TYPE {
                        cols.push(col.unbind());
                    } else if pyo3::ffi::PyUnicode_Check(col.as_ptr()) == 1 {
                        cols.push(col.unbind());
                    } else {
                        return Err(typeerror!(
                            "expected Column or ColumnRef or str, got {:?}",
                            col.py(),
                            col.as_ptr()
                        ));
                    }
                }
            }

            let mut lock = slf.inner.lock();
            lock.distinct = SelectDistinct::DistinctOn(cols);
        }

        Ok(slf)
    }

    #[pyo3(signature=(*cols))]
    fn columns<'a>(
        slf: pyo3::PyRef<'a, Self>,
        cols: &'a pyo3::Bound<'a, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut exprs = Vec::with_capacity(PyTupleMethods::len(cols));

        for expr in PyTupleMethods::iter(cols) {
            exprs.push(PySelectExpr::from_bound_into_any(&expr)?);
        }

        {
            let mut lock = slf.inner.lock();
            lock.cols = exprs;
        }

        Ok(slf)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_table<'a>(
        slf: pyo3::PyRef<'a, Self>,
        table: &'a pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let table = {
            if let Ok(x) = table.cast_exact::<crate::table::PyTable>() {
                let guard = x.get().inner.lock();
                guard.name.clone_ref(slf.py())
            } else {
                crate::common::PyTableName::from_pyobject(table)?
            }
        };

        {
            let mut lock = slf.inner.lock();
            lock.table.push(table);
        }

        Ok(slf)
    }

    fn limit(slf: pyo3::PyRef<'_, Self>, n: u64) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();
            lock.limit = Some(n);
        }

        slf
    }

    fn offset(slf: pyo3::PyRef<'_, Self>, n: u64) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();
            lock.offset = Some(n);
        }

        slf
    }

    fn r#where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let condition = crate::expression::PyExpr::from_bound_into_any(condition)?;

        {
            let mut lock = slf.inner.lock();
            lock.r#where = Some(condition);
        }

        Ok(slf)
    }

    fn order_by<'a>(
        slf: pyo3::PyRef<'a, Self>,
        order: &'a pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let order = unsafe {
            if order.is_exact_instance_of::<super::order::PyOrder>() {
                order.clone().unbind()
            } else {
                return Err(typeerror!("expected Order, got {:?}", order.py(), order.as_ptr()));
            }
        };

        {
            let mut lock = slf.inner.lock();
            lock.orders.push(order);
        }

        Ok(slf)
    }

    fn build(
        &self,
        backend: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<(String, pyo3::Py<pyo3::PyAny>)> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_query_parts!(backend => build_collect_any_into(stmt))
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_query_string!(backend => build_collect_any_into(stmt))
    }
}
