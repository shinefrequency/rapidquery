use crate::backend::PyQueryStatement;
use pyo3::types::{PyAnyMethods, PyDictMethods, PyTupleMethods};
use sea_query::IntoIden;

#[derive(Debug, Default)]
pub enum InsertValueSource {
    #[default]
    None,
    // Select(pyo3::Py<pyo3::PyAny>),
    Single(
        // Always is `Vec<PyExpr>`
        Vec<pyo3::Py<pyo3::PyAny>>,
    ),
    Many(
        // Always is `Vec<Vec<PyExpr>>`
        Vec<Vec<pyo3::Py<pyo3::PyAny>>>,
    ),
}

#[derive(Default)]
pub struct InsertInner {
    pub replace: bool,

    // Always is `Option<TableName>`
    pub table: Option<pyo3::Py<pyo3::PyAny>>,
    pub columns: Vec<String>,
    pub source: InsertValueSource,

    // Always is `Option<OnConflict>`
    pub on_conflict: Option<pyo3::Py<pyo3::PyAny>>,
    pub returning_clause: super::returning::ReturningClause,
    pub default_values: Option<u32>,
    // TODO
    // pub with: Option<pyo3::Py<pyo3::PyAny>>,
}

impl InsertInner {
    #[inline]
    fn as_statement(&self, py: pyo3::Python) -> sea_query::InsertStatement {
        let mut stmt = sea_query::InsertStatement::new();
        if self.replace {
            stmt.replace();
        }

        if let Some(x) = &self.table {
            let x = unsafe { x.cast_bound_unchecked::<crate::common::PyTableName>(py) };
            stmt.into_table(x.get().clone());
        }

        stmt.columns(self.columns.iter().map(sea_query::Alias::new));

        match &self.source {
            InsertValueSource::None => (),
            InsertValueSource::Single(x) => unsafe {
                stmt.values(
                    x.iter()
                        .map(|x| x.cast_bound_unchecked::<crate::expression::PyExpr>(py))
                        .map(|x| x.get().inner.clone()),
                )
                .unwrap();
            },
            InsertValueSource::Many(x) => unsafe {
                for y in x.iter() {
                    stmt.values(
                        y.iter()
                            .map(|x| x.cast_bound_unchecked::<crate::expression::PyExpr>(py))
                            .map(|x| x.get().inner.clone()),
                    )
                    .unwrap();
                }
            },
        }

        if let Some(on_conflict) = &self.on_conflict {
            let on_conflict =
                unsafe { on_conflict.cast_bound_unchecked::<super::on_conflict::PyOnConflict>(py) };

            let x = on_conflict.get().inner.lock();
            stmt.on_conflict(x.as_statement(py));
        }

        if let Some(rows) = self.default_values {
            stmt.or_default_values_many(rows);
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

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "Insert", frozen, extends=PyQueryStatement)]
pub struct PyInsert {
    inner: parking_lot::Mutex<InsertInner>,
}

impl PyInsert {
    #[inline]
    fn values_from_dictionary<'a>(
        slf: pyo3::PyRef<'a, Self>,
        kwds: &'a pyo3::Bound<'_, pyo3::types::PyDict>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        {
            let lock = slf.inner.lock();

            if !lock.columns.is_empty() && lock.columns.len() != kwds.len() {
                return Err(
                    pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "values length isn't equal to columns length - this occurres when you're calling `.values()` method multiple times with different columns."
                    )
                );
            }
        }

        let mut cols = Vec::<String>::new();
        let mut vals = Vec::<pyo3::Py<pyo3::PyAny>>::new();

        unsafe {
            for (key, value) in kwds.iter() {
                let key = key.extract::<String>().unwrap_unchecked();
                cols.push(key);
                vals.push(crate::expression::PyExpr::from_bound_into_any(value)?);
            }
        }

        {
            let mut lock = slf.inner.lock();

            match std::mem::take(&mut lock.source) {
                InsertValueSource::None => {
                    lock.source = InsertValueSource::Single(vals);
                    lock.columns = cols;
                }
                InsertValueSource::Single(oldvals) => {
                    lock.source = InsertValueSource::Many(vec![oldvals, vals]);
                }
                InsertValueSource::Many(mut arr_of_vals) => {
                    arr_of_vals.push(vals);
                    lock.source = InsertValueSource::Many(arr_of_vals);
                }
            }
        }

        Ok(slf)
    }

    #[inline]
    fn values_from_tuple<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: &'a pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        {
            let lock = slf.inner.lock();

            if lock.columns.len() != PyTupleMethods::len(args) {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "values length isn't equal to columns length",
                ));
            }
        }

        let mut vals = Vec::<pyo3::Py<pyo3::PyAny>>::new();

        unsafe {
            for value in PyTupleMethods::iter(args) {
                vals.push(crate::expression::PyExpr::from_bound_into_any(value)?);
            }
        }

        {
            let mut lock = slf.inner.lock();

            match std::mem::take(&mut lock.source) {
                InsertValueSource::None => {
                    lock.source = InsertValueSource::Single(vals);
                }
                InsertValueSource::Single(oldvals) => {
                    lock.source = InsertValueSource::Many(vec![oldvals, vals]);
                }
                InsertValueSource::Many(mut arr_of_vals) => {
                    arr_of_vals.push(vals);
                    lock.source = InsertValueSource::Many(arr_of_vals);
                }
            }
        }

        Ok(slf)
    }
}

#[pyo3::pymethods]
impl PyInsert {
    #[new]
    fn new() -> (Self, PyQueryStatement) {
        let slf = Self {
            inner: parking_lot::Mutex::new(Default::default()),
        };
        (slf, PyQueryStatement)
    }

    fn replace(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();
            lock.replace = true;
        }

        slf
    }

    fn into<'a>(
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

    #[pyo3(signature=(*args))]
    fn columns<'a>(
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
            lock.columns = cols;
        }

        Ok(slf)
    }

    #[pyo3(signature=(*args, **kwds))]
    fn values<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: &'a pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&'a pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if !PyTupleMethods::is_empty(args) && kwds.is_some() {
            return Err(typeerror!("cannot use both args and kwargs at the same time",));
        }

        if !PyTupleMethods::is_empty(args) {
            Self::values_from_tuple(slf, args)
        } else if kwds.is_some() {
            Self::values_from_dictionary(slf, kwds.unwrap())
        } else {
            Err(typeerror!("no arguments provided",))
        }
    }

    #[pyo3(signature=(rows=1))]
    fn or_default_values(slf: pyo3::PyRef<'_, Self>, rows: u32) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();
            lock.default_values = Some(rows);
        }

        slf
    }

    fn on_conflict<'a>(
        slf: pyo3::PyRef<'a, Self>,
        action: &'a pyo3::Bound<'a, super::on_conflict::PyOnConflict>,
    ) -> pyo3::PyRef<'a, Self> {
        {
            let mut lock = slf.inner.lock();
            lock.on_conflict = Some(action.clone().unbind().into_any());
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

        write!(s, "<Insert").unwrap();

        if lock.replace {
            write!(s, " replace=True").unwrap();
        }
        if let Some(x) = &lock.table {
            write!(s, " into={x}").unwrap();
        }
        if !lock.columns.is_empty() {
            write!(s, " columns={:?}", lock.columns).unwrap();
        }
        if let Some(x) = &lock.on_conflict {
            write!(s, " on_conflict={x}").unwrap();
        }

        match &lock.source {
            InsertValueSource::None => {
                if let Some(x) = lock.default_values {
                    write!(s, " default_rows={x}").unwrap();
                }
            }
            InsertValueSource::Single(x) => {
                write!(s, " values=[").unwrap();

                let n = x.len();
                for (index, ix) in x.iter().enumerate() {
                    if index + 1 == n {
                        write!(s, "{ix}").unwrap();
                    } else {
                        write!(s, "{ix}, ").unwrap();
                    }
                }
                write!(s, "]").unwrap();
            }
            InsertValueSource::Many(x) => {
                write!(s, " values=[[").unwrap();

                let n = x.len();
                for (index_1, nested) in x.iter().enumerate() {
                    let j = nested.len();
                    for (index_2, val) in nested.iter().enumerate() {
                        if index_2 + 1 == j {
                            write!(s, "{val}").unwrap();
                        } else {
                            write!(s, "{val}, ").unwrap();
                        }
                    }

                    if index_1 + 1 < n {
                        write!(s, "], [").unwrap();
                    }
                }
                write!(s, "]]").unwrap();
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
