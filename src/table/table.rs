use crate::backend::PySchemaStatement;
use pyo3::types::PyAnyMethods;

type ColumnsSequence = Vec<(String, pyo3::Py<pyo3::PyAny>)>;

pub struct TableInner {
    // Always is `TableName`
    pub name: pyo3::Py<pyo3::PyAny>,

    // TODO: use `indexmap` crate to optimize lookup from `O(n)` into `O(1)`
    // Always is `ColumnsSequence<String, Column>`
    pub columns: ColumnsSequence,

    // Always is `Vec<Index>`
    pub indexes: Vec<pyo3::Py<pyo3::PyAny>>,

    // Always is `Vec<ForeignKeySpec>`
    pub foreign_keys: Vec<pyo3::Py<pyo3::PyAny>>,

    // Always is `Vec<Expr>`
    pub checks: Vec<pyo3::Py<pyo3::PyAny>>,
    pub if_not_exists: bool,
    pub temporary: bool,
    pub comment: Option<String>,
    pub engine: Option<String>,
    pub collate: Option<String>,
    pub character_set: Option<String>,
    pub extra: Option<String>,
}

impl TableInner {
    #[optimize(speed)]
    pub fn as_table_create_statement(&self, py: pyo3::Python) -> sea_query::TableCreateStatement {
        let mut stmt = sea_query::TableCreateStatement::new();

        stmt.table(unsafe {
            let x = self.name.cast_bound_unchecked::<crate::common::PyTableName>(py);
            x.get().clone()
        });

        for (_, col) in self.columns.iter() {
            let colbound = unsafe { col.cast_bound_unchecked::<crate::column::PyColumn>(py) };
            let collock = colbound.get().inner.lock();

            stmt.col(collock.as_column_def(py));
        }

        for ix in self.indexes.iter() {
            let ixbound = unsafe { ix.cast_bound_unchecked::<crate::index::PyIndex>(py) };
            let ixlock = ixbound.get().inner.lock();

            // We only want PRIMARY KEY indexes here.
            if ixlock.options & (crate::index::IndexOptions::Primary as u8) > 0 {
                stmt.primary_key(&mut ixlock.as_statement(py));
            }
        }

        for fk in self.foreign_keys.iter() {
            let fkbound = unsafe { fk.cast_bound_unchecked::<crate::foreign_key::PyForeignKey>(py) };

            let fklock = fkbound.get().inner.lock();
            stmt.foreign_key(&mut fklock.as_statement(py));
        }

        for check in self.checks.iter() {
            let check_expr = unsafe { check.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            let check_expr = check_expr.get();

            stmt.check(check_expr.inner.clone());
        }

        if self.if_not_exists {
            stmt.if_not_exists();
        }
        if self.temporary {
            stmt.temporary();
        }

        if let Some(x) = &self.comment {
            stmt.comment(x);
        }
        if let Some(x) = &self.engine {
            stmt.engine(x);
        }
        if let Some(x) = &self.collate {
            stmt.collate(x);
        }
        if let Some(x) = &self.character_set {
            stmt.character_set(x);
        }
        if let Some(x) = &self.extra {
            stmt.extra(x);
        }

        stmt
    }

    #[optimize(speed)]
    pub fn as_index_create_statements(&self, py: pyo3::Python) -> Vec<sea_query::IndexCreateStatement> {
        let mut vec = Vec::with_capacity(self.indexes.len());

        for ix in self.indexes.iter() {
            let ixbound = unsafe { ix.cast_bound_unchecked::<crate::index::PyIndex>(py) };
            let ixlock = ixbound.get().inner.lock();

            // We only want PRIMARY KEY indexes here.
            if ixlock.options & (crate::index::IndexOptions::Primary as u8) > 0 {
                continue;
            }

            vec.push(ixlock.as_statement(py));
        }

        vec
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "_TableColumnsSequence", frozen)]
#[allow(non_camel_case_types)]
pub struct Py_TableColumnsSequence {
    pub inner: std::sync::Arc<parking_lot::Mutex<TableInner>>,
}

#[pyo3::pymethods]
impl Py_TableColumnsSequence {
    fn __getattr__(slf: pyo3::PyRef<'_, Self>, name: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let lock = slf.inner.lock();

        lock.columns
            .iter()
            .find(|(x, _)| x.eq(&name))
            .map(|(_, x)| x.clone_ref(slf.py()))
            .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyKeyError, _>(name.to_owned()))
    }

    fn get(slf: pyo3::PyRef<'_, Self>, name: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let lock = slf.inner.lock();

        lock.columns
            .iter()
            .find(|(x, _)| x.eq(&name))
            .map(|(_, x)| x.clone_ref(slf.py()))
            .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyKeyError, _>(name.to_owned()))
    }

    fn append(&self, col: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        unsafe {
            let mut lock = self.inner.lock();

            if std::hint::unlikely(pyo3::ffi::Py_TYPE(col.as_ptr()) != crate::typeref::COLUMN_TYPE) {
                return Err(typeerror!("expected Column, got {:?}", col.py(), col.as_ptr()));
            }

            let colbound = col.cast_unchecked::<crate::column::PyColumn>();
            let mut colobj = colbound.get().inner.lock();

            colobj.column_ref = crate::column::LazyColumnRef::TableName(lock.name.clone_ref(col.py()));

            let name = colobj.name.clone();
            drop(colobj);

            lock.columns.push((name, col.unbind()));
        }

        Ok(())
    }

    fn remove(slf: pyo3::PyRef<'_, Self>, name: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let mut lock = slf.inner.lock();

        let position = lock
            .columns
            .iter()
            .position(|(x, _)| x.eq(&name))
            .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyKeyError, _>(name.to_owned()))?;

        let (_, x) = lock.columns.remove(position);
        Ok(x.clone_ref(slf.py()))
    }

    fn to_list(slf: pyo3::PyRef<'_, Self>) -> Vec<pyo3::Py<pyo3::PyAny>> {
        let lock = slf.inner.lock();

        lock.columns.iter().map(|(_, x)| x.clone_ref(slf.py())).collect()
    }

    fn clear(slf: pyo3::PyRef<'_, Self>) {
        let mut lock = slf.inner.lock();
        lock.columns.clear();
    }

    fn __len__(slf: pyo3::PyRef<'_, Self>) -> usize {
        let lock = slf.inner.lock();
        lock.columns.len()
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "Table", frozen, extends=PySchemaStatement)]
pub struct PyTable {
    pub inner: std::sync::Arc<parking_lot::Mutex<TableInner>>,
}

#[pyo3::pymethods]
impl PyTable {
    #[new]
    #[pyo3(
        signature=(
            name,
            columns,
            indexes=Vec::new(),
            foreign_keys=Vec::new(),
            checks=Vec::new(),
            if_not_exists=false,
            temporary=false,
            comment=None,
            engine=None,
            collate=None,
            character_set=None,
            extra=None
        )
    )]
    fn new(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
        columns: Vec<pyo3::Py<pyo3::PyAny>>,
        indexes: Vec<pyo3::Py<pyo3::PyAny>>,
        foreign_keys: Vec<pyo3::Py<pyo3::PyAny>>,
        checks: Vec<pyo3::Py<pyo3::PyAny>>,
        if_not_exists: bool,
        temporary: bool,
        comment: Option<String>,
        engine: Option<String>,
        collate: Option<String>,
        character_set: Option<String>,
        extra: Option<String>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let py = name.py();

        let name = crate::common::PyTableName::from_pyobject(name)?;

        let mut cols = ColumnsSequence::with_capacity(columns.len());
        for col in columns {
            unsafe {
                if std::hint::unlikely(pyo3::ffi::Py_TYPE(col.as_ptr()) != crate::typeref::COLUMN_TYPE) {
                    return Err(typeerror!("expected Column, got {:?}", py, col.as_ptr()));
                }

                let colbound = col.cast_bound_unchecked::<crate::column::PyColumn>(py);
                let mut colobj = colbound.get().inner.lock();

                colobj.column_ref = crate::column::LazyColumnRef::TableName(name.clone_ref(py));

                let name = colobj.name.clone();
                drop(colobj);

                cols.push((name, col));
            }
        }

        let mut indexes_vec = Vec::with_capacity(indexes.capacity());
        for ix in indexes {
            if std::hint::unlikely(!ix.bind(py).is_instance_of::<crate::index::PyIndex>()) {
                return Err(typeerror!("expected Index, got {:?}", py, ix.as_ptr()));
            }

            let ixbound = unsafe { ix.bind(py).cast_unchecked::<crate::index::PyIndex>() };
            let mut ixlock = ixbound.get().inner.lock();

            ixlock.table = Some(name.clone_ref(py));
            ixlock.regenerate_name(py);
            drop(ixlock);

            indexes_vec.push(ix);
        }

        let mut foreign_keys_vec = Vec::with_capacity(foreign_keys.capacity());
        for fk in foreign_keys {
            if std::hint::unlikely(!fk.bind(py).is_instance_of::<crate::foreign_key::PyForeignKey>()) {
                return Err(typeerror!("expected ForeignKeySpec, got {:?}", py, fk.as_ptr()));
            }

            foreign_keys_vec.push(fk);
        }

        let mut checks_vec = Vec::with_capacity(checks.capacity());
        for expr in checks {
            if unsafe { pyo3::ffi::Py_TYPE(expr.as_ptr()) != crate::typeref::EXPR_TYPE } {
                return Err(typeerror!("expected Expr, got {:?}", py, expr.as_ptr()));
            }

            checks_vec.push(expr);
        }

        let inner = TableInner {
            name,
            columns: cols,
            indexes: indexes_vec,
            foreign_keys: foreign_keys_vec,
            checks: checks_vec,
            if_not_exists,
            temporary,
            comment,
            engine,
            collate,
            character_set,
            extra,
        };

        let slf = Self {
            inner: std::sync::Arc::new(parking_lot::Mutex::new(inner)),
        };

        Ok(pyo3::PyClassInitializer::from((slf, PySchemaStatement)))
    }

    #[getter]
    fn name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.name.clone_ref(py)
    }

    #[getter]
    fn columns(&self, py: pyo3::Python) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let map = Py_TableColumnsSequence {
            inner: std::sync::Arc::clone(&self.inner),
        };
        pyo3::Py::new(py, map).map(|x| x.into_any())
    }

    #[getter]
    fn c(&self, py: pyo3::Python) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let map = Py_TableColumnsSequence {
            inner: std::sync::Arc::clone(&self.inner),
        };
        pyo3::Py::new(py, map).map(|x| x.into_any())
    }

    #[getter]
    fn indexes(&self, py: pyo3::Python) -> Vec<pyo3::Py<pyo3::PyAny>> {
        let lock = self.inner.lock();

        lock.indexes.iter().map(|x| x.clone_ref(py)).collect()
    }

    #[setter]
    fn set_indexes(&self, py: pyo3::Python, val: Vec<pyo3::Py<pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let lock = self.inner.lock();
        let name = lock.name.clone_ref(py);
        drop(lock);

        let mut indexes_vec = Vec::with_capacity(val.capacity());
        for ix in val {
            if std::hint::unlikely(!ix.bind(py).is_instance_of::<crate::index::PyIndex>()) {
                return Err(typeerror!("expected Index, got {:?}", py, ix.as_ptr()));
            }

            let ixbound = unsafe { ix.bind(py).cast_unchecked::<crate::index::PyIndex>() };
            let mut ixlock = ixbound.get().inner.lock();

            ixlock.table = Some(name.clone_ref(py));
            drop(ixlock);

            indexes_vec.push(ix);
        }

        let mut lock = self.inner.lock();
        lock.indexes = indexes_vec;

        Ok(())
    }

    #[getter]
    fn foreign_keys(&self, py: pyo3::Python) -> Vec<pyo3::Py<pyo3::PyAny>> {
        let lock = self.inner.lock();

        lock.foreign_keys.iter().map(|x| x.clone_ref(py)).collect()
    }

    #[setter]
    fn set_foreign_keys(&self, py: pyo3::Python, val: Vec<pyo3::Py<pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let mut foreign_keys_vec = Vec::with_capacity(val.capacity());
        for fk in val {
            if std::hint::unlikely(!fk.bind(py).is_instance_of::<crate::foreign_key::PyForeignKey>()) {
                return Err(typeerror!("expected ForeignKeySpec, got {:?}", py, fk.as_ptr()));
            }

            foreign_keys_vec.push(fk);
        }

        let mut lock = self.inner.lock();
        lock.foreign_keys = foreign_keys_vec;

        Ok(())
    }

    #[getter]
    fn checks(&self, py: pyo3::Python) -> Vec<pyo3::Py<pyo3::PyAny>> {
        let lock = self.inner.lock();

        lock.checks.iter().map(|x| x.clone_ref(py)).collect()
    }

    #[setter]
    fn set_checks(&self, py: pyo3::Python, val: Vec<pyo3::Py<pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let mut checks_vec = Vec::with_capacity(val.capacity());
        for expr in val {
            if unsafe { pyo3::ffi::Py_TYPE(expr.as_ptr()) != crate::typeref::EXPR_TYPE } {
                return Err(typeerror!("expected Expr, got {:?}", py, expr.as_ptr()));
            }

            checks_vec.push(expr);
        }

        let mut lock = self.inner.lock();
        lock.checks = checks_vec;

        Ok(())
    }

    #[getter]
    fn if_not_exists(slf: pyo3::PyRef<'_, Self>) -> bool {
        slf.inner.lock().if_not_exists
    }

    #[setter]
    fn set_if_not_exists(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        lock.if_not_exists = val;
    }

    #[getter]
    fn temporary(slf: pyo3::PyRef<'_, Self>) -> bool {
        slf.inner.lock().temporary
    }

    #[setter]
    fn set_temporary(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        lock.temporary = val;
    }

    #[getter]
    fn comment(&self) -> Option<String> {
        let lock = self.inner.lock();
        lock.comment.as_ref().map(|x| x.to_string())
    }

    #[setter]
    fn set_comment(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.comment = val;

        Ok(())
    }

    #[getter]
    fn engine(&self) -> Option<String> {
        let lock = self.inner.lock();
        lock.engine.as_ref().map(|x| x.to_string())
    }

    #[setter]
    fn set_engine(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.engine = val;

        Ok(())
    }

    #[getter]
    fn collate(&self) -> Option<String> {
        let lock = self.inner.lock();
        lock.collate.as_ref().map(|x| x.to_string())
    }

    #[setter]
    fn set_collate(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.collate = val;

        Ok(())
    }

    #[getter]
    fn character_set(&self) -> Option<String> {
        let lock = self.inner.lock();
        lock.character_set.as_ref().map(|x| x.to_string())
    }

    #[setter]
    fn set_character_set(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.character_set = val;

        Ok(())
    }

    #[getter]
    fn extra(&self) -> Option<String> {
        let lock = self.inner.lock();
        lock.extra.as_ref().map(|x| x.to_string())
    }

    #[setter]
    fn set_extra(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.extra = val;

        Ok(())
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_table_create_statement(backend.py());
        let ix = lock.as_index_create_statements(backend.py());
        drop(lock);

        let mut sql = build_schema!(backend => build_any(stmt))? + ";\n";

        for ix in ix.into_iter() {
            sql += &build_schema!(backend => build_any(ix))?;
            sql.push(';');
            sql.push('\n');
        }

        Ok(sql)
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();
        let mut s = Vec::with_capacity(50);

        write!(s, "<Table name={} columns=[", lock.name).unwrap();

        let n = lock.columns.len();
        for (index, (_, col)) in lock.columns.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{col}").unwrap();
            } else {
                write!(s, "{col}, ").unwrap();
            }
        }

        write!(s, "] indexes=[").unwrap();

        let n = lock.indexes.len();
        for (index, ix) in lock.indexes.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{ix}").unwrap();
            } else {
                write!(s, "{ix}, ").unwrap();
            }
        }

        write!(s, "] foreign_keys=[").unwrap();

        let n = lock.foreign_keys.len();
        for (index, fk) in lock.foreign_keys.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{fk}").unwrap();
            } else {
                write!(s, "{fk}, ").unwrap();
            }
        }

        write!(s, "]").unwrap();

        if lock.if_not_exists {
            write!(s, " if_not_exists=True").unwrap();
        }
        if lock.temporary {
            write!(s, " temporary=True").unwrap();
        }

        if let Some(x) = &lock.comment {
            write!(s, " comment={x}").unwrap();
        }
        if let Some(x) = &lock.engine {
            write!(s, " engine={x}").unwrap();
        }
        if let Some(x) = &lock.collate {
            write!(s, " collate={x}").unwrap();
        }
        if let Some(x) = &lock.character_set {
            write!(s, " character_set={x}").unwrap();
        }

        write!(s, " checks=[").unwrap();

        let n = lock.checks.len();
        for (index, ix) in lock.checks.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{ix}").unwrap();
            } else {
                write!(s, "{ix}, ").unwrap();
            }
        }

        write!(s, "]>").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
