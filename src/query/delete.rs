use crate::backend::PyQueryStatement;
use pyo3::types::{PyAnyMethods, PyTupleMethods};
use sea_query::IntoIden;

#[derive(Default)]
pub struct DeleteInner {
    // Always is `Option<TableName>`
    pub table: Option<pyo3::Py<pyo3::PyAny>>,

    // Always is `Option<PyExpr>`
    pub r#where: Option<pyo3::Py<pyo3::PyAny>>,
    pub limit: Option<u64>,
    pub returning_clause: super::returning::ReturningClause,
    pub orders: Vec<super::order::OrderClause>,
    // TODO
    // pub with: Option<pyo3::Py<pyo3::PyAny>>,
}

impl DeleteInner {
    fn as_statement(&self, py: pyo3::Python) -> sea_query::DeleteStatement {
        let mut stmt = sea_query::DeleteStatement::new();

        if let Some(x) = &self.table {
            let x = unsafe { x.cast_bound_unchecked::<crate::common::PyTableName>(py) };
            stmt.from_table(x.get().clone());
        }

        if let Some(x) = &self.r#where {
            let x = unsafe { x.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            stmt.and_where(x.get().inner.clone());
        }

        if let Some(n) = self.limit {
            stmt.limit(n);
        }

        match &self.returning_clause {
            super::returning::ReturningClause::None => (),
            super::returning::ReturningClause::All => {
                stmt.returning_all();
            }
            super::returning::ReturningClause::Columns(x) => {
                stmt.returning(sea_query::ReturningClause::Columns(
                    x.iter()
                        .map(sea_query::Alias::new)
                        .map(|x| sea_query::ColumnRef::Column(x.into_iden()))
                        .collect(),
                ));
            }
        }

        for order in self.orders.iter() {
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

#[pyo3::pyclass(module = "rapidquery._lib", name = "Delete", frozen, extends=PyQueryStatement)]
pub struct PyDelete {
    pub inner: parking_lot::Mutex<DeleteInner>,
}

#[pyo3::pymethods]
impl PyDelete {
    #[new]
    fn new() -> (Self, PyQueryStatement) {
        let slf = Self {
            inner: parking_lot::Mutex::new(Default::default()),
        };
        (slf, PyQueryStatement)
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
            lock.table = Some(table);
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

    #[pyo3(signature=(*args))]
    fn returning<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: &'a pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut cols = Vec::<String>::new();

        unsafe {
            for col in PyTupleMethods::iter(args) {
                if pyo3::ffi::Py_TYPE(col.as_ptr()) == crate::typeref::COLUMN_TYPE {
                    let col = col.cast_into_unchecked::<crate::column::PyColumn>();
                    cols.push(col.get().inner.lock().name.clone());
                } else if pyo3::ffi::PyUnicode_CheckExact(col.as_ptr()) == 1 {
                    cols.push(col.extract::<String>().unwrap_unchecked());
                } else {
                    return Err(typeerror!(
                        "expected Column or str, got {:?}",
                        col.py(),
                        col.as_ptr()
                    ));
                }
            }
        }

        {
            let mut lock = slf.inner.lock();
            lock.returning_clause = super::returning::ReturningClause::Columns(cols);
        }

        Ok(slf)
    }

    fn returning_all(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();
            lock.returning_clause = super::returning::ReturningClause::All;
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

    #[pyo3(signature=(target, order, null_order=None))]
    fn order_by<'a>(
        slf: pyo3::PyRef<'a, Self>,
        target: pyo3::Bound<'_, pyo3::PyAny>,
        order: String,
        null_order: Option<String>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let order = super::order::OrderClause::from_parameters(target, order, null_order)?;

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

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();
        let mut s = Vec::<u8>::with_capacity(30);

        write!(s, "<Delete").unwrap();

        if let Some(x) = &lock.table {
            write!(s, " from_table={x}").unwrap();
        }
        if let Some(x) = lock.limit {
            write!(s, " limit={x}").unwrap();
        }
        if let Some(x) = &lock.r#where {
            write!(s, " where={x}").unwrap();
        }

        write!(s, " orders=[").unwrap();

        let n = lock.orders.len();
        for (index, expr) in lock.orders.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{expr}]").unwrap();
            } else {
                write!(s, "{expr}, ").unwrap();
            }
        }

        match &lock.returning_clause {
            super::returning::ReturningClause::None => (),
            super::returning::ReturningClause::All => {
                write!(s, " returning_all").unwrap();
            }
            super::returning::ReturningClause::Columns(x) => {
                write!(s, " returning={x:?}").unwrap();
            }
        }

        write!(s, ">").unwrap();
        unsafe { String::from_utf8_unchecked(s) }
    }
}
