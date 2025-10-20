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

        let x = unsafe {
            self.name
                .cast_bound_unchecked::<crate::common::PyTableName>(py)
        };
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

#[pyo3::pyclass(module = "rapidquery._lib", name = "DropTable", frozen)]
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
    ) -> pyo3::PyResult<Self> {
        let name = crate::common::PyTableName::from_pyobject(name)?;

        let inner = DropTableInner {
            name,
            options: (if_exists as u8) * (DropTableOptions::IfExists as u8)
                | (restrict as u8) * (DropTableOptions::Restrict as u8)
                | (cascade as u8) * (DropTableOptions::Cascade as u8),
        };

        Ok(Self {
            inner: parking_lot::Mutex::new(inner),
        })
    }

    #[getter]
    fn name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.name.clone_ref(py)
    }

    #[setter]
    fn set_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        unsafe {
            if pyo3::ffi::Py_TYPE(val.as_ptr()) == crate::typeref::TABLE_NAME_TYPE {
                return Err(typeerror!(
                    "expected TableName, got {:?}",
                    val.py(),
                    val.as_ptr()
                ));
            }
        }

        let mut lock = self.inner.lock();
        lock.name = val.clone().unbind();
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

    fn __copy__(&self, py: pyo3::Python) -> Self {
        Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        }
    }

    fn copy(&self, py: pyo3::Python) -> Self {
        Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        }
    }

    fn build(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_schema!(
            crate::backend::into_schema_builder => backend => build_any(stmt)
        )
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

#[pyo3::pyclass(module = "rapidquery._lib", name = "RenameTable", frozen)]
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
    ) -> pyo3::PyResult<Self> {
        let from_name = crate::common::PyTableName::from_pyobject(from_name)?;
        let to_name = crate::common::PyTableName::from_pyobject(to_name)?;

        let inner = RenameTableInner { from_name, to_name };

        Ok(Self {
            inner: parking_lot::Mutex::new(inner),
        })
    }

    #[getter]
    fn from_name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.from_name.clone_ref(py)
    }

    #[setter]
    fn set_from_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        unsafe {
            if pyo3::ffi::Py_TYPE(val.as_ptr()) == crate::typeref::TABLE_NAME_TYPE {
                return Err(typeerror!(
                    "expected TableName, got {:?}",
                    val.py(),
                    val.as_ptr()
                ));
            }
        }

        let mut lock = self.inner.lock();
        lock.from_name = val.clone().unbind();
        Ok(())
    }

    #[getter]
    fn to_name(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        lock.to_name.clone_ref(py)
    }

    #[setter]
    fn set_to_name(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        unsafe {
            if pyo3::ffi::Py_TYPE(val.as_ptr()) == crate::typeref::TABLE_NAME_TYPE {
                return Err(typeerror!(
                    "expected TableName, got {:?}",
                    val.py(),
                    val.as_ptr()
                ));
            }
        }

        let mut lock = self.inner.lock();
        lock.to_name = val.clone().unbind();
        Ok(())
    }

    fn __copy__(&self, py: pyo3::Python) -> Self {
        Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        }
    }

    fn copy(&self, py: pyo3::Python) -> Self {
        Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone_ref(py)),
        }
    }

    fn build(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_schema!(
            crate::backend::into_schema_builder => backend => build_any(stmt)
        )
    }

    fn __repr__(&self) -> String {
        let lock = self.inner.lock();
        format!("<RenameTable {} {}>", lock.from_name, lock.to_name)
    }
}
