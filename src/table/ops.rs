use crate::backend::PySchemaStatement;
use pyo3::types::PyAnyMethods;
use pyo3::PyTypeInfo;
use sea_query::IntoIden;

enum DropTableOptions {
    IfExists = 1 << 0,
    Restrict = 1 << 1,
    Cascade = 1 << 2,
}

struct DropTableInner {
    // Always is `TableName`
    name: pyo3::Py<pyo3::PyAny>,
    options: u8,
}

impl DropTableInner {
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            name: self.name.clone_ref(py),
            options: self.options,
        }
    }

    #[optimize(speed)]
    fn as_statement(&self, py: pyo3::Python<'_>) -> sea_query::TableDropStatement {
        let mut stmt = sea_query::TableDropStatement::new();

        let x = unsafe { self.name.cast_bound_unchecked::<crate::common::PyTableName>(py) };
        stmt.table(x.get().clone());

        if self.options & (DropTableOptions::IfExists as u8) > 0 {
            stmt.if_exists();
        }
        if self.options & (DropTableOptions::Restrict as u8) > 0 {
            stmt.restrict();
        }
        if self.options & (DropTableOptions::Cascade as u8) > 0 {
            stmt.cascade();
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "DropTable", frozen, extends=PySchemaStatement)]
pub struct PyDropTable {
    inner: parking_lot::Mutex<DropTableInner>,
}

#[pyo3::pymethods]
impl PyDropTable {
    #[new]
    #[pyo3(signature=(
        name,
        if_exists=false,
        restrict=false,
        cascade=false,
    ))]
    fn new(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
        if_exists: bool,
        restrict: bool,
        cascade: bool,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let name = crate::common::PyTableName::from_pyobject(name)?;

        let inner = DropTableInner {
            name,
            options: ((if_exists as u8) * (DropTableOptions::IfExists as u8))
                | ((restrict as u8) * (DropTableOptions::Restrict as u8))
                | ((cascade as u8) * (DropTableOptions::Cascade as u8)),
        };

        let slf = Self {
            inner: parking_lot::Mutex::new(inner),
        };
        Ok(pyo3::PyClassInitializer::from((slf, PySchemaStatement)))
    }

    #[getter]
    fn name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.name.clone_ref(py)
    }

    #[setter]
    fn set_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.name = crate::common::PyTableName::from_pyobject(val)?;
        Ok(())
    }

    #[getter]
    fn if_exists(slf: pyo3::PyRef<'_, Self>) -> bool {
        (slf.inner.lock().options & (DropTableOptions::IfExists as u8)) > 0
    }

    #[setter]
    fn set_if_exists(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        if val {
            lock.options |= DropTableOptions::IfExists as u8;
        } else {
            lock.options &= !(DropTableOptions::IfExists as u8);
        }
    }

    #[getter]
    fn restrict(slf: pyo3::PyRef<'_, Self>) -> bool {
        (slf.inner.lock().options & (DropTableOptions::Restrict as u8)) > 0
    }

    #[setter]
    fn set_restrict(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        if val {
            lock.options |= DropTableOptions::Restrict as u8;
        } else {
            lock.options &= !(DropTableOptions::Restrict as u8);
        }
    }

    #[getter]
    fn cascade(slf: pyo3::PyRef<'_, Self>) -> bool {
        (slf.inner.lock().options & (DropTableOptions::Cascade as u8)) > 0
    }

    #[setter]
    fn set_cascade(slf: pyo3::PyRef<'_, Self>, val: bool) {
        let mut lock = slf.inner.lock();
        if val {
            lock.options |= DropTableOptions::Cascade as u8;
        } else {
            lock.options &= !(DropTableOptions::Cascade as u8);
        }
    }

    fn __copy__(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let slf = Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn copy(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let slf = Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
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

        let mut s: Vec<u8> = Vec::with_capacity(20);

        write!(s, "<DropTable {:?}", lock.name).unwrap();

        if lock.options & (DropTableOptions::IfExists as u8) > 0 {
            write!(s, " if_exists=True").unwrap();
        }
        if lock.options & (DropTableOptions::Cascade as u8) > 0 {
            write!(s, " cascade=True").unwrap();
        }
        if lock.options & (DropTableOptions::Restrict as u8) > 0 {
            write!(s, " restrict=False").unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}

struct RenameTableInner {
    // Always is `TableName`
    from_name: pyo3::Py<pyo3::PyAny>,

    // Always is `TableName`
    to_name: pyo3::Py<pyo3::PyAny>,
}

impl RenameTableInner {
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            from_name: self.from_name.clone_ref(py),
            to_name: self.to_name.clone_ref(py),
        }
    }

    fn as_statement(&self, py: pyo3::Python<'_>) -> sea_query::TableRenameStatement {
        let mut stmt = sea_query::TableRenameStatement::new();

        let from = unsafe {
            self.from_name
                .cast_bound_unchecked::<crate::common::PyTableName>(py)
        };
        let to = unsafe {
            self.to_name
                .cast_bound_unchecked::<crate::common::PyTableName>(py)
        };

        stmt.table(from.get().clone(), to.get().clone());
        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "RenameTable", frozen, extends=PySchemaStatement)]
pub struct PyRenameTable {
    inner: parking_lot::Mutex<RenameTableInner>,
}

#[pyo3::pymethods]
impl PyRenameTable {
    #[new]
    #[pyo3(signature=(from_name, to_name))]
    fn new(
        from_name: &pyo3::Bound<'_, pyo3::PyAny>,
        to_name: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let from_name = crate::common::PyTableName::from_pyobject(from_name)?;
        let to_name = crate::common::PyTableName::from_pyobject(to_name)?;

        let inner = RenameTableInner { from_name, to_name };

        let slf = Self {
            inner: parking_lot::Mutex::new(inner),
        };
        Ok(pyo3::PyClassInitializer::from((slf, PySchemaStatement)))
    }

    #[getter]
    #[allow(clippy::wrong_self_convention)]
    fn from_name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.from_name.clone_ref(py)
    }

    #[setter]
    fn set_from_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.from_name = crate::common::PyTableName::from_pyobject(val)?;
        Ok(())
    }

    #[getter]
    fn to_name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.to_name.clone_ref(py)
    }

    #[setter]
    fn set_to_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.to_name = crate::common::PyTableName::from_pyobject(val)?;
        Ok(())
    }

    fn __copy__(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let slf = Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn copy(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let slf = Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_schema!(
            backend => build_any(stmt)
        )
    }

    fn __repr__(&self) -> String {
        let lock = self.inner.lock();
        format!("<RenameTable {} {}>", lock.from_name, lock.to_name)
    }
}

struct TruncateTableInner {
    // Always is `TableName`
    name: pyo3::Py<pyo3::PyAny>,
}

impl TruncateTableInner {
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            name: self.name.clone_ref(py),
        }
    }

    fn as_statement(&self, py: pyo3::Python<'_>) -> sea_query::TableTruncateStatement {
        let mut stmt = sea_query::TableTruncateStatement::new();

        let name = unsafe { self.name.cast_bound_unchecked::<crate::common::PyTableName>(py) };

        stmt.table(name.get().clone());
        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "TruncateTable", frozen, extends=PySchemaStatement)]
pub struct PyTruncateTable {
    inner: parking_lot::Mutex<TruncateTableInner>,
}

#[pyo3::pymethods]
impl PyTruncateTable {
    #[new]
    #[pyo3(signature=(name))]
    fn new(name: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let name = crate::common::PyTableName::from_pyobject(name)?;

        let inner = TruncateTableInner { name };

        let slf = Self {
            inner: parking_lot::Mutex::new(inner),
        };
        Ok(pyo3::PyClassInitializer::from((slf, PySchemaStatement)))
    }

    #[getter]
    fn name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.name.clone_ref(py)
    }

    #[setter]
    fn set_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.name = crate::common::PyTableName::from_pyobject(val)?;
        Ok(())
    }

    fn __copy__(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let slf = Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn copy(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let slf = Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_schema!(
             backend => build_any(stmt)
        )
    }

    fn __repr__(&self) -> String {
        let lock = self.inner.lock();
        format!("<TruncateTable {}>", lock.name)
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "AlterTableOptionMeta", frozen, subclass)]
pub struct PyAlterTableOptionMeta;

#[pyo3::pymethods]
impl PyAlterTableOptionMeta {
    #[new]
    fn new() -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "don't use directly AlterTableOptionMeta class; use AlterTable*Option classes",
        ))
    }
}

#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "AlterTableAddColumnOption",
    frozen,
    extends=PyAlterTableOptionMeta,
)]
pub struct PyAlterTableAddColumnOption {
    // Always is `Column`
    column: pyo3::Py<pyo3::PyAny>,
    if_not_exists: bool,
}

#[pyo3::pymethods]
impl PyAlterTableAddColumnOption {
    #[new]
    #[pyo3(signature=(column, if_not_exists=false))]
    fn new(
        column: &pyo3::Bound<'_, pyo3::PyAny>,
        if_not_exists: bool,
    ) -> pyo3::PyResult<(Self, PyAlterTableOptionMeta)> {
        unsafe {
            if std::hint::unlikely(pyo3::ffi::Py_TYPE(column.as_ptr()) != crate::typeref::COLUMN_TYPE) {
                return Err(typeerror!(
                    "expected Column, got {:?}",
                    column.py(),
                    column.as_ptr()
                ));
            }
        }

        Ok((
            Self {
                column: column.clone().unbind(),
                if_not_exists,
            },
            PyAlterTableOptionMeta,
        ))
    }

    #[getter]
    fn column(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        self.column.clone_ref(py)
    }

    #[getter]
    fn if_not_exists(&self) -> bool {
        self.if_not_exists
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s: Vec<u8> = Vec::with_capacity(20);

        write!(s, "<AlterTableAddColumnOption {}", self.column).unwrap();
        if self.if_not_exists {
            write!(s, " if_not_exists=True").unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}

#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "AlterTableModifyColumnOption",
    frozen,
    extends=PyAlterTableOptionMeta,
)]
pub struct PyAlterTableModifyColumnOption {
    // Always is `Column`
    column: pyo3::Py<pyo3::PyAny>,
}

#[pyo3::pymethods]
impl PyAlterTableModifyColumnOption {
    #[new]
    #[pyo3(signature=(column))]
    fn new(column: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<(Self, PyAlterTableOptionMeta)> {
        unsafe {
            if std::hint::unlikely(pyo3::ffi::Py_TYPE(column.as_ptr()) != crate::typeref::COLUMN_TYPE) {
                return Err(typeerror!(
                    "expected Column, got {:?}",
                    column.py(),
                    column.as_ptr()
                ));
            }
        }

        Ok((
            Self {
                column: column.clone().unbind(),
            },
            PyAlterTableOptionMeta,
        ))
    }

    #[getter]
    fn column(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        self.column.clone_ref(py)
    }

    fn __repr__(&self) -> String {
        format!("<AlterTableModifyColumnOption {}>", self.column)
    }
}

#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "AlterTableRenameColumnOption",
    frozen,
    extends=PyAlterTableOptionMeta,
)]
pub struct PyAlterTableRenameColumnOption {
    from_name: String,
    to_name: String,
}

#[pyo3::pymethods]
impl PyAlterTableRenameColumnOption {
    #[new]
    #[pyo3(signature=(from_name, to_name))]
    fn new(from_name: String, to_name: String) -> pyo3::PyResult<(Self, PyAlterTableOptionMeta)> {
        Ok((Self { from_name, to_name }, PyAlterTableOptionMeta))
    }

    #[getter]
    #[allow(clippy::wrong_self_convention)]
    fn from_name(&self) -> String {
        self.from_name.clone()
    }

    #[getter]
    fn to_name(&self) -> String {
        self.to_name.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "<AlterTableRenameColumnOption from_name={:?} to_name={:?}>",
            self.from_name, self.to_name
        )
    }
}

#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "AlterTableDropColumnOption",
    frozen,
    extends=PyAlterTableOptionMeta,
)]
pub struct PyAlterTableDropColumnOption {
    name: String,
}

#[pyo3::pymethods]
impl PyAlterTableDropColumnOption {
    #[new]
    #[pyo3(signature=(name))]
    fn new(name: String) -> pyo3::PyResult<(Self, PyAlterTableOptionMeta)> {
        Ok((Self { name }, PyAlterTableOptionMeta))
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    fn __repr__(&self) -> String {
        format!("<AlterTableDropColumnOption {:?}>", self.name,)
    }
}

#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "AlterTableAddForeignKeyOption",
    frozen,
    extends=PyAlterTableOptionMeta,
)]
pub struct PyAlterTableAddForeignKeyOption {
    // Always is `ForeignKeySpec`
    foreign_key: pyo3::Py<pyo3::PyAny>,
}

#[pyo3::pymethods]
impl PyAlterTableAddForeignKeyOption {
    #[new]
    #[pyo3(signature=(foreign_key))]
    fn new(foreign_key: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<(Self, PyAlterTableOptionMeta)> {
        unsafe {
            if std::hint::unlikely(!foreign_key.is_exact_instance_of::<crate::foreign_key::PyForeignKey>()) {
                return Err(typeerror!(
                    "expected ForeignKeySpec, got {:?}",
                    foreign_key.py(),
                    foreign_key.as_ptr()
                ));
            }
        }

        Ok((
            Self {
                foreign_key: foreign_key.clone().unbind(),
            },
            PyAlterTableOptionMeta,
        ))
    }

    #[getter]
    fn foreign_key(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        self.foreign_key.clone_ref(py)
    }

    fn __repr__(&self) -> String {
        format!("<AlterTableAddForeignKeyOption {}>", self.foreign_key)
    }
}

#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "AlterTableDropForeignKeyOption",
    frozen,
    extends=PyAlterTableOptionMeta,
)]
pub struct PyAlterTableDropForeignKeyOption {
    name: String,
}

#[pyo3::pymethods]
impl PyAlterTableDropForeignKeyOption {
    #[new]
    #[pyo3(signature=(name))]
    fn new(name: String) -> pyo3::PyResult<(Self, PyAlterTableOptionMeta)> {
        Ok((Self { name }, PyAlterTableOptionMeta))
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    fn __repr__(&self) -> String {
        format!("<AlterTableDropForeignKeyOption {:?}>", self.name,)
    }
}

struct AlterTableInner {
    /// Always is `TableName`
    name: pyo3::Py<pyo3::PyAny>,
    options: Vec<pyo3::Py<pyo3::PyAny>>,
}

impl AlterTableInner {
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            name: self.name.clone_ref(py),
            options: self.options.iter().map(|x| x.clone_ref(py)).collect(),
        }
    }

    fn as_statement(&self, py: pyo3::Python) -> sea_query::TableAlterStatement {
        let mut stmt = sea_query::TableAlterStatement::new();

        let x = unsafe { self.name.cast_bound_unchecked::<crate::common::PyTableName>(py) };
        stmt.table(x.get().clone());

        for op in self.options.iter() {
            unsafe {
                let op_type = pyo3::ffi::Py_TYPE(op.as_ptr());

                if op_type == PyAlterTableAddColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableAddColumnOption>(py);
                    let x = bound.get();

                    let column = x.column.cast_bound_unchecked::<crate::column::PyColumn>(py);
                    let column = column.get().inner.lock().as_column_def(py);

                    if x.if_not_exists {
                        stmt.add_column_if_not_exists(column);
                    } else {
                        stmt.add_column(column);
                    }
                } else if op_type == PyAlterTableAddForeignKeyOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableAddForeignKeyOption>(py);
                    let x = bound.get();

                    let spec = x
                        .foreign_key
                        .cast_bound_unchecked::<crate::foreign_key::PyForeignKey>(py);
                    let spec = spec.get().inner.lock().as_statement(py);

                    stmt.add_foreign_key(spec.get_foreign_key());
                } else if op_type == PyAlterTableDropColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableDropColumnOption>(py);
                    let x = bound.get();

                    stmt.drop_column(sea_query::Alias::new(&x.name).into_iden());
                } else if op_type == PyAlterTableDropForeignKeyOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableDropForeignKeyOption>(py);
                    let x = bound.get();

                    stmt.drop_foreign_key(sea_query::Alias::new(&x.name).into_iden());
                } else if op_type == PyAlterTableModifyColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableModifyColumnOption>(py);
                    let x = bound.get();

                    let column = x.column.cast_bound_unchecked::<crate::column::PyColumn>(py);
                    let column = column.get().inner.lock().as_column_def(py);

                    stmt.modify_column(column);
                } else if op_type == PyAlterTableRenameColumnOption::type_object_raw(py) {
                    let bound = op.cast_bound_unchecked::<PyAlterTableRenameColumnOption>(py);
                    let x = bound.get();

                    stmt.rename_column(
                        sea_query::Alias::new(&x.from_name).into_iden(),
                        sea_query::Alias::new(&x.to_name).into_iden(),
                    );
                }
            }
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "AlterTable", frozen, extends=PySchemaStatement)]
pub struct PyAlterTable {
    inner: parking_lot::Mutex<AlterTableInner>,
}

#[pyo3::pymethods]
impl PyAlterTable {
    #[new]
    fn new(
        name: &pyo3::Bound<'_, pyo3::PyAny>,
        options: Vec<pyo3::Py<pyo3::PyAny>>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        let py = name.py();
        let name = crate::common::PyTableName::from_pyobject(name)?;

        for op in options.iter() {
            if !op.bind(py).is_instance_of::<PyAlterTableOptionMeta>() {
                return Err(typeerror!(
                    "expected a list of PyAlterTableOptionMeta, found {:?} in options",
                    py,
                    op.as_ptr()
                ));
            }
        }

        let slf = Self {
            inner: parking_lot::Mutex::new(AlterTableInner { name, options }),
        };
        Ok(pyo3::PyClassInitializer::from((slf, PySchemaStatement)))
    }

    #[getter]
    fn name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.name.clone_ref(py)
    }

    #[setter]
    fn set_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.name = crate::common::PyTableName::from_pyobject(val)?;
        Ok(())
    }

    #[getter]
    fn options(&self, py: pyo3::Python) -> Vec<pyo3::Py<pyo3::PyAny>> {
        self.inner
            .lock()
            .options
            .iter()
            .map(|x| x.clone_ref(py))
            .collect()
    }

    #[setter]
    fn set_options(&self, py: pyo3::Python, val: Vec<pyo3::Py<pyo3::PyAny>>) -> pyo3::PyResult<()> {
        for op in val.iter() {
            if !op.bind(py).is_instance_of::<PyAlterTableOptionMeta>() {
                return Err(typeerror!(
                    "expected a list of PyAlterTableOptionMeta, found {:?} in list",
                    py,
                    op.as_ptr()
                ));
            }
        }

        let mut lock = self.inner.lock();
        lock.options = val;
        Ok(())
    }

    fn add_option(&self, option: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        if !option.is_instance_of::<PyAlterTableOptionMeta>() {
            return Err(typeerror!(
                "expected PyAlterTableOptionMeta, got {:?}",
                option.py(),
                option.as_ptr()
            ));
        }

        let mut lock = self.inner.lock();
        lock.options.push(option.clone().unbind());
        Ok(())
    }

    fn __copy__(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let slf = Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn copy(&self, py: pyo3::Python) -> pyo3::Py<Self> {
        let slf = Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        };
        pyo3::Py::new(py, pyo3::PyClassInitializer::from((slf, PySchemaStatement))).unwrap()
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_schema!(
           backend => build_any(stmt)
        )
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();
        let mut s = Vec::with_capacity(50);

        write!(s, "<AlterTable name={} options=[", lock.name).unwrap();

        let n = lock.options.len() - 1;
        for (index, op) in lock.options.iter().enumerate() {
            if index == n {
                write!(s, "{op}").unwrap();
            } else {
                write!(s, "{op}, ").unwrap();
            }
        }
        write!(s, "]>").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
