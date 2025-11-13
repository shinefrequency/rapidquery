use crate::backend::PySchemaStatement;
use sea_query::IntoIden;

#[derive(Debug, Clone)]
pub struct IndexTypeAlias(sea_query::IndexType);

impl From<String> for IndexTypeAlias {
    fn from(value: String) -> Self {
        let lower = value.to_ascii_lowercase();

        if lower == "hash" {
            Self(sea_query::IndexType::Hash)
        } else if lower == "full text" {
            Self(sea_query::IndexType::FullText)
        } else if lower == "btree" {
            Self(sea_query::IndexType::BTree)
        } else {
            Self(sea_query::IndexType::Custom(
                sea_query::Alias::new(value).into_iden(),
            ))
        }
    }
}

impl From<IndexTypeAlias> for sea_query::IndexType {
    fn from(value: IndexTypeAlias) -> Self {
        value.0
    }
}

impl From<sea_query::IndexType> for IndexTypeAlias {
    fn from(value: sea_query::IndexType) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for IndexTypeAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            sea_query::IndexType::BTree => write!(f, "BTREE"),
            sea_query::IndexType::FullText => write!(f, "FULL TEXT"),
            sea_query::IndexType::Hash => write!(f, "HASH"),
            sea_query::IndexType::Custom(x) => write!(f, "{}", x.to_string()),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexOptions {
    Primary = 1 << 0,
    Unique = 1 << 1,
    IfNotExists = 1 << 2,
    NullsNotDistinct = 1 << 3,
}

pub struct IndexInner {
    pub name: String,

    // Always is `Vec<IndexColumn>`
    pub columns: Vec<pyo3::Py<pyo3::PyAny>>,

    // Always is `Option<TableName>`
    pub table: Option<pyo3::Py<pyo3::PyAny>>,

    pub options: u8,
    pub index_type: Option<IndexTypeAlias>,

    // Always is `Option<Expr>`
    pub r#where: Option<pyo3::Py<pyo3::PyAny>>,
    pub include: Vec<String>,
}

impl IndexInner {
    pub fn regenerate_name(&mut self, py: pyo3::Python) {
        let table_name = match &self.table {
            Some(x) => unsafe {
                let bound = x.cast_bound_unchecked::<crate::common::PyTableName>(py);

                bound.get().name.to_string()
            },
            None => String::new(),
        };

        // ix_<table>_<column_names...>
        let mut s = format!("ix_{table_name}");

        for col in self.columns.iter() {
            let bound = unsafe { col.cast_bound_unchecked::<crate::common::PyIndexColumn>(py) };

            s.push('_');
            s += &bound.get().name;
        }

        self.name = s;
    }

    pub fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            name: self.name.clone(),
            columns: self.columns.iter().map(|x| x.clone_ref(py)).collect(),
            table: self.table.as_ref().map(|x| x.clone_ref(py)),
            options: self.options,
            index_type: self.index_type.clone(),
            r#where: self.r#where.as_ref().map(|x| x.clone_ref(py)),
            include: self.include.clone(),
        }
    }

    #[optimize(speed)]
    pub fn as_statement(&self, py: pyo3::Python) -> sea_query::IndexCreateStatement {
        let mut stmt = sea_query::IndexCreateStatement::new();

        stmt.name(&self.name);

        for col in &self.columns {
            #[cfg(not(debug_assertions))]
            let col = unsafe { col.bind(py).cast_unchecked::<crate::common::PyIndexColumn>() };

            #[cfg(debug_assertions)]
            let col = col.bind(py).cast::<crate::common::PyIndexColumn>().unwrap();

            let col = col.get();
            stmt.col(col.clone());
        }

        if let Some(x) = &self.table {
            #[cfg(not(debug_assertions))]
            let x = unsafe { x.bind(py).cast_unchecked::<crate::common::PyTableName>() };

            #[cfg(debug_assertions)]
            let x = x.bind(py).cast::<crate::common::PyTableName>().unwrap();

            let x = x.get();

            stmt.table(x.clone());
        }

        if let Some(x) = &self.index_type {
            stmt.index_type(x.clone().into());
        }

        for c in &self.include {
            stmt.include(sea_query::Alias::new(c.clone()));
        }

        if self.options & (IndexOptions::Primary as u8) > 0 {
            stmt.primary();
        }
        if self.options & (IndexOptions::Unique as u8) > 0 {
            stmt.unique();
        }
        if self.options & (IndexOptions::IfNotExists as u8) > 0 {
            stmt.if_not_exists();
        }
        if self.options & (IndexOptions::NullsNotDistinct as u8) > 0 {
            stmt.nulls_not_distinct();
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "Index", frozen, extends=PySchemaStatement)]
pub struct PyIndex {
    pub inner: parking_lot::Mutex<IndexInner>,
}

#[inline]
#[optimize(speed)]
fn convert_pyobject_into_index_column(
    py: pyo3::Python,
    obj: pyo3::Py<pyo3::PyAny>,
) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
    unsafe {
        if pyo3::ffi::Py_TYPE(obj.as_ptr()) == crate::typeref::INDEX_COLUMN_TYPE {
            return Ok(obj);
        }

        if let Ok(x) = obj.extract::<&str>(py) {
            return Ok(pyo3::Py::new(py, crate::common::PyIndexColumn::from(x))?.into_any());
        }

        Err(typeerror!(
            "expected IndexColumn or str, got {:?}",
            py,
            obj.as_ptr()
        ))
    }
}

#[pyo3::pymethods]
impl PyIndex {
    #[new]
    #[pyo3(
        signature=(
            columns,
            name=None,
            table=None,
            if_not_exists=false,
            primary=false,
            unique=false,
            nulls_not_distinct=false,
            include=Vec::new(),
            index_type=None,
            r#where=None
        )
    )]
    fn new(
        py: pyo3::Python,
        columns: Vec<pyo3::Py<pyo3::PyAny>>,
        name: Option<String>,
        table: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
        if_not_exists: bool,
        primary: bool,
        unique: bool,
        nulls_not_distinct: bool,
        include: Vec<String>,
        index_type: Option<String>,
        r#where: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let mut cols = Vec::with_capacity(columns.capacity());
        for col in columns {
            cols.push(convert_pyobject_into_index_column(py, col)?);
        }

        let table: Option<pyo3::Py<pyo3::PyAny>> = {
            match table {
                Some(table) => Some(crate::common::PyTableName::from_pyobject(table)?),
                None => None,
            }
        };

        let name = name.unwrap_or_default();

        let r#where: Option<pyo3::Py<pyo3::PyAny>> = {
            match r#where {
                Some(x) => unsafe {
                    if pyo3::ffi::Py_TYPE(x.as_ptr()) != crate::typeref::EXPR_TYPE {
                        return Err(typeerror!("expected Expr as where, got {:?}", x.py(), x.as_ptr()));
                    }

                    Some(x.clone().unbind())
                },
                None => None,
            }
        };

        let options = ((primary as u8) * (IndexOptions::Primary as u8))
            | ((unique as u8) * (IndexOptions::Unique as u8))
            | ((if_not_exists as u8) * (IndexOptions::IfNotExists as u8))
            | ((nulls_not_distinct as u8) * (IndexOptions::NullsNotDistinct as u8));

        let mut inner = IndexInner {
            name,
            columns: cols,
            table,
            options,
            index_type: index_type.map(|x| x.into()),
            r#where,
            include,
        };

        if inner.name.is_empty() {
            inner.regenerate_name(py);
        }

        let slf = Self {
            inner: parking_lot::Mutex::new(inner),
        };

        Ok(pyo3::PyClassInitializer::from((slf, PySchemaStatement)))
    }

    #[getter]
    fn name(&self) -> String {
        let lock = self.inner.lock();
        lock.name.clone()
    }

    #[setter]
    fn set_name(&self, val: String) {
        let mut lock = self.inner.lock();
        lock.name = val;
    }

    #[getter]
    fn table(&self, py: pyo3::Python) -> Option<pyo3::Py<pyo3::PyAny>> {
        let lock = self.inner.lock();
        lock.table.as_ref().map(|x| x.clone_ref(py))
    }

    #[setter]
    fn set_table(&self, val: Option<&pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let val: Option<pyo3::Py<pyo3::PyAny>> = {
            match val {
                Some(val) => Some(crate::common::PyTableName::from_pyobject(val)?),
                None => None,
            }
        };

        let mut lock = self.inner.lock();
        lock.table = val;
        Ok(())
    }

    #[getter]
    fn primary(slf: pyo3::PyRef<'_, Self>) -> bool {
        (slf.inner.lock().options & (IndexOptions::Primary as u8)) > 0
    }

    #[setter]
    fn set_primary(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        if val {
            lock.options |= IndexOptions::Primary as u8;
        } else {
            lock.options &= !(IndexOptions::Primary as u8);
        }
    }

    #[getter]
    fn unique(slf: pyo3::PyRef<'_, Self>) -> bool {
        (slf.inner.lock().options & (IndexOptions::Unique as u8)) > 0
    }

    #[setter]
    fn set_unique(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        if val {
            lock.options |= IndexOptions::Unique as u8;
        } else {
            lock.options &= !(IndexOptions::Unique as u8);
        }
    }

    #[getter]
    fn if_not_exists(slf: pyo3::PyRef<'_, Self>) -> bool {
        (slf.inner.lock().options & (IndexOptions::IfNotExists as u8)) > 0
    }

    #[setter]
    fn set_if_not_exists(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        if val {
            lock.options |= IndexOptions::IfNotExists as u8;
        } else {
            lock.options &= !(IndexOptions::IfNotExists as u8);
        }
    }

    #[getter]
    fn nulls_not_distinct(slf: pyo3::PyRef<'_, Self>) -> bool {
        (slf.inner.lock().options & (IndexOptions::NullsNotDistinct as u8)) > 0
    }

    #[setter]
    fn set_nulls_not_distinct(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        if val {
            lock.options |= IndexOptions::NullsNotDistinct as u8;
        } else {
            lock.options &= !(IndexOptions::NullsNotDistinct as u8);
        }
    }

    #[getter]
    fn columns(&self, py: pyo3::Python) -> Vec<pyo3::Py<pyo3::PyAny>> {
        let lock = self.inner.lock();
        lock.columns.iter().map(|x| x.clone_ref(py)).collect()
    }

    #[setter]
    fn set_columns(&self, py: pyo3::Python, val: Vec<pyo3::Py<pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let mut cols = Vec::with_capacity(val.capacity());

        for col in val {
            cols.push(convert_pyobject_into_index_column(py, col)?);
        }

        let mut lock = self.inner.lock();
        lock.columns = cols;

        Ok(())
    }

    #[getter]
    fn index_type(&self) -> Option<String> {
        let lock = self.inner.lock();
        lock.index_type.as_ref().map(|x| x.to_string())
    }

    #[setter]
    fn set_index_type(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.index_type = val.map(|x| x.into());

        Ok(())
    }

    #[getter]
    fn include(&self) -> Vec<String> {
        let lock = self.inner.lock();
        lock.include.iter().map(|x| x.to_string()).collect()
    }

    #[setter]
    fn set_include(&self, val: Vec<String>) {
        let mut lock = self.inner.lock();
        lock.include = val;
    }

    fn __copy__(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let lock = self.inner.lock();

        let slf = Self {
            inner: parking_lot::Mutex::new(lock.clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn copy(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let lock = self.inner.lock();

        let slf = Self {
            inner: parking_lot::Mutex::new(lock.clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_schema!(backend => build_any(stmt))
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();
        let mut s = Vec::with_capacity(50);

        write!(s, "<Index {:?} columns=[", lock.name).unwrap();

        let n = lock.columns.len() - 1;
        for (index, col) in lock.columns.iter().enumerate() {
            if index == n {
                write!(s, "{col}]").unwrap();
            } else {
                write!(s, "{col}, ").unwrap();
            }
        }

        if let Some(x) = &lock.table {
            write!(s, " table={}", x).unwrap();
        }
        if lock.options & (IndexOptions::IfNotExists as u8) > 0 {
            write!(s, " if_not_exists=True").unwrap();
        }
        if lock.options & (IndexOptions::Primary as u8) > 0 {
            write!(s, " primary=True").unwrap();
        }
        if lock.options & (IndexOptions::Unique as u8) > 0 {
            write!(s, " unique=True").unwrap();
        }
        if lock.options & (IndexOptions::IfNotExists as u8) > 0 {
            write!(s, " if_not_exists=True").unwrap();
        }
        if lock.options & (IndexOptions::NullsNotDistinct as u8) > 0 {
            write!(s, " nulls_not_distinct=True").unwrap();
        }

        if let Some(x) = &lock.index_type {
            write!(s, " index_type={:?}", x.to_string()).unwrap();
        }
        if !lock.include.is_empty() {
            write!(s, " include={:?}", lock.include).unwrap();
        }
        if let Some(x) = &lock.r#where {
            write!(s, " where={x}").unwrap();
        }

        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}

pub struct DropIndexInner {
    pub name: String,
    pub table: Option<pyo3::Py<pyo3::PyAny>>,
    pub if_exists: bool,
}

impl DropIndexInner {
    pub fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            name: self.name.clone(),
            table: self.table.as_ref().map(|x| x.clone_ref(py)),
            if_exists: self.if_exists,
        }
    }

    pub(crate) fn as_statement(&self, py: pyo3::Python) -> sea_query::IndexDropStatement {
        let mut stmt = sea_query::IndexDropStatement::new();

        stmt.name(&self.name);

        if let Some(x) = &self.table {
            #[cfg(not(debug_assertions))]
            let x = unsafe { x.bind(py).cast_unchecked::<crate::common::PyTableName>() };

            #[cfg(debug_assertions)]
            let x = x.bind(py).cast::<crate::common::PyTableName>().unwrap();

            let x = x.get();

            stmt.table(x.clone());
        }

        if self.if_exists {
            stmt.if_exists();
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "DropIndex", frozen)]
pub struct PyDropIndex {
    pub inner: parking_lot::Mutex<DropIndexInner>,
}

#[pyo3::pymethods]
impl PyDropIndex {
    #[new]
    #[pyo3(signature=(name, table=None, if_exists=false))]
    fn new(
        name: String,
        table: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
        if_exists: bool,
    ) -> pyo3::PyResult<Self> {
        let table: Option<pyo3::Py<pyo3::PyAny>> = {
            match table {
                Some(table) => Some(crate::common::PyTableName::from_pyobject(table)?),
                None => None,
            }
        };

        let inner = DropIndexInner {
            name,
            table,
            if_exists,
        };

        Ok(Self {
            inner: parking_lot::Mutex::new(inner),
        })
    }

    #[getter]
    fn name(&self) -> String {
        let lock = self.inner.lock();
        lock.name.clone()
    }

    #[setter]
    fn set_name(&self, val: String) {
        let mut lock = self.inner.lock();
        lock.name = val;
    }

    #[getter]
    fn table(&self, py: pyo3::Python) -> Option<pyo3::Py<pyo3::PyAny>> {
        let lock = self.inner.lock();
        lock.table.as_ref().map(|x| x.clone_ref(py))
    }

    #[setter]
    fn set_table(&self, val: Option<&pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let val: Option<pyo3::Py<pyo3::PyAny>> = {
            match val {
                Some(val) => Some(crate::common::PyTableName::from_pyobject(val)?),
                None => None,
            }
        };

        let mut lock = self.inner.lock();
        lock.table = val;
        Ok(())
    }

    #[getter]
    fn if_exists(slf: pyo3::PyRef<'_, Self>) -> bool {
        slf.inner.lock().if_exists
    }

    #[setter]
    fn set_if_exists(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        lock.if_exists = val;
    }

    fn __copy__(&self, py: pyo3::Python) -> Self {
        let lock = self.inner.lock();

        Self {
            inner: parking_lot::Mutex::new(lock.clone_ref(py)),
        }
    }

    fn copy(&self, py: pyo3::Python) -> Self {
        let lock = self.inner.lock();

        Self {
            inner: parking_lot::Mutex::new(lock.clone_ref(py)),
        }
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_schema!(backend => build_any(stmt))
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();
        let mut s = Vec::with_capacity(50);

        write!(s, "<DropIndex {:?}", lock.name).unwrap();
        if let Some(x) = &lock.table {
            write!(s, " table={}", x).unwrap();
        }
        if lock.if_exists {
            write!(s, " if_exists=True").unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
