pub struct ForeignKeySpecInner {
    pub name: String,

    /// Always is [`crate::common::TableName`]
    pub to_table: pyo3::Py<pyo3::PyAny>,
    pub to_columns: Vec<String>,

    /// Always is [`Option<crate::common::TableName>`]
    pub from_table: Option<pyo3::Py<pyo3::PyAny>>,
    pub from_columns: Vec<String>,

    pub on_delete: Option<sea_query::ForeignKeyAction>,
    pub on_update: Option<sea_query::ForeignKeyAction>,
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "ForeignKeySpec", frozen)]
pub struct PyForeignKeySpec {
    pub inner: parking_lot::Mutex<ForeignKeySpecInner>,
}

#[pyo3::pymethods]
impl PyForeignKeySpec {
    #[new]
    #[pyo3(
        signature=(
            from_columns,
            to_columns,
            to_table,
            from_table=None,
            name=None,
            on_delete=None,
            on_update=None
        )
    )]
    fn new(
        from_columns: Vec<String>,
        to_columns: Vec<String>,
        to_table: &pyo3::Bound<'_, pyo3::PyAny>,
        from_table: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
        name: Option<String>,
        on_delete: Option<u8>,
        on_update: Option<u8>,
    ) -> pyo3::PyResult<Self> {
        if on_delete.is_some_and(|x| x > 4) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "use FOREIGN_KEY_ACTION_* constants for on_delete",
            ));
        }

        if on_update.is_some_and(|x| x > 4) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "use FOREIGN_KEY_ACTION_* constants for on_update",
            ));
        }

        let to_table: pyo3::Py<pyo3::PyAny> = crate::common::PyTableName::from_pyobject(to_table)?;

        let from_table: Option<pyo3::Py<pyo3::PyAny>> = unsafe {
            match from_table {
                Some(from_table) => Some(crate::common::PyTableName::from_pyobject(from_table)?),
                None => None,
            }
        };

        let name = match name {
            Some(x) => x,
            None => String::from("FK_") + &uuid::Uuid::new_v4().as_simple().to_string(),
        };

        Ok(Self {
            inner: parking_lot::Mutex::new(ForeignKeySpecInner {
                name,
                to_table,
                to_columns,
                from_table,
                from_columns,
                on_delete: on_delete.map(|x| unsafe { std::mem::transmute(x) }),
                on_update: on_update.map(|x| unsafe { std::mem::transmute(x) }),
            }),
        })
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.lock().name.clone()
    }

    #[setter]
    fn set_name(&self, val: String) {
        let mut lock = self.inner.lock();
        lock.name = val;
    }

    #[getter]
    fn from_table(&self, py: pyo3::Python) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.inner
            .lock()
            .from_table
            .as_ref()
            .map(|x| x.clone_ref(py))
    }

    #[setter]
    fn set_from_table(&self, value: Option<&pyo3::Bound<'_, pyo3::PyAny>>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.from_table = match value {
            Some(x) => Some(crate::common::PyTableName::from_pyobject(x)?),
            None => None,
        };
        Ok(())
    }

    #[getter]
    fn to_table(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        self.inner.lock().to_table.clone_ref(py)
    }

    #[setter]
    fn set_to_table(&self, value: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();
        lock.to_table = crate::common::PyTableName::from_pyobject(value)?;
        Ok(())
    }

    #[getter]
    fn from_columns(&self) -> Vec<String> {
        self.inner.lock().from_columns.clone()
    }

    #[setter]
    fn set_from_columns(&self, val: Vec<String>) {
        let mut lock = self.inner.lock();
        lock.from_columns = val;
    }

    #[getter]
    fn to_columns(&self) -> Vec<String> {
        self.inner.lock().to_columns.clone()
    }

    #[setter]
    fn set_to_columns(&self, val: Vec<String>) {
        let mut lock = self.inner.lock();
        lock.to_columns = val;
    }

    #[getter]
    fn on_delete(&self) -> Option<u8> {
        self.inner.lock().on_delete.map(|x| x as u8)
    }

    #[setter]
    fn set_on_delete(&self, val: Option<u8>) -> pyo3::PyResult<()> {
        if val.is_some_and(|x| x > 4) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "use FOREIGN_KEY_ACTION_* constants",
            ));
        }

        let mut lock = self.inner.lock();
        lock.on_delete = val.map(|x| unsafe { std::mem::transmute(x) });

        Ok(())
    }

    #[getter]
    fn on_update(&self) -> Option<u8> {
        self.inner.lock().on_update.map(|x| x as u8)
    }

    #[setter]
    fn set_on_update(&self, val: Option<u8>) -> pyo3::PyResult<()> {
        if val.is_some_and(|x| x > 4) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "use FOREIGN_KEY_ACTION_* constants",
            ));
        }

        let mut lock = self.inner.lock();
        lock.on_update = val.map(|x| unsafe { std::mem::transmute(x) });

        Ok(())
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();
        let mut s = Vec::with_capacity(50);

        write!(
            s,
            "<ForeignKeySpec {:?} to_table={} to_columns={:?} from_columns={:?}",
            lock.name, lock.to_table, lock.to_columns, lock.from_columns
        )
        .unwrap();

        if let Some(x) = &lock.from_table {
            write!(s, " from_table={}", x).unwrap();
        }
        if let Some(x) = &lock.on_delete {
            write!(s, " on_delete={:?}", x).unwrap();
        }
        if let Some(x) = &lock.on_update {
            write!(s, " on_update={:?}", x).unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
