use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct ForeignKeyActionAlias(sea_query::ForeignKeyAction);

impl FromStr for ForeignKeyActionAlias {
    type Err = pyo3::PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();

        if lower == "cascade" {
            Ok(Self(sea_query::ForeignKeyAction::Cascade))
        } else if lower == "no action" {
            Ok(Self(sea_query::ForeignKeyAction::NoAction))
        } else if lower == "restrict" {
            Ok(Self(sea_query::ForeignKeyAction::Restrict))
        } else if lower == "set default" {
            Ok(Self(sea_query::ForeignKeyAction::SetDefault))
        } else if lower == "set null" {
            Ok(Self(sea_query::ForeignKeyAction::SetNull))
        } else {
            Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "unknown foreign key action: {s}"
            )))
        }
    }
}

impl From<ForeignKeyActionAlias> for sea_query::ForeignKeyAction {
    fn from(value: ForeignKeyActionAlias) -> Self {
        value.0
    }
}

impl From<sea_query::ForeignKeyAction> for ForeignKeyActionAlias {
    fn from(value: sea_query::ForeignKeyAction) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for ForeignKeyActionAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            sea_query::ForeignKeyAction::Cascade => write!(f, "CASCADE"),
            sea_query::ForeignKeyAction::NoAction => write!(f, "NO ACTION"),
            sea_query::ForeignKeyAction::Restrict => write!(f, "RESTRICT"),
            sea_query::ForeignKeyAction::SetDefault => write!(f, "SET DEFAULT"),
            sea_query::ForeignKeyAction::SetNull => write!(f, "SET NULL"),
        }
    }
}

pub struct ForeignKeyInner {
    pub name: String,

    // Always is `TableName`
    pub to_table: pyo3::Py<pyo3::PyAny>,
    pub to_columns: Vec<String>,

    // Always is `Option<TableName>`
    pub from_table: Option<pyo3::Py<pyo3::PyAny>>,
    pub from_columns: Vec<String>,

    pub on_delete: Option<ForeignKeyActionAlias>,
    pub on_update: Option<ForeignKeyActionAlias>,
}

impl ForeignKeyInner {
    pub fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            name: self.name.clone(),
            to_table: self.to_table.clone_ref(py),
            to_columns: self.to_columns.clone(),
            from_table: self.from_table.as_ref().map(|x| x.clone_ref(py)),
            from_columns: self.to_columns.clone(),
            on_delete: self.on_delete,
            on_update: self.on_update,
        }
    }

    #[optimize(speed)]
    pub fn as_statement(&self, py: pyo3::Python<'_>) -> sea_query::ForeignKeyCreateStatement {
        let mut stmt = sea_query::ForeignKeyCreateStatement::new();

        stmt.name(&self.name);

        if let Some(from_table) = &self.from_table {
            let from_table = unsafe { from_table.cast_bound_unchecked::<crate::common::PyTableName>(py) };

            stmt.from_tbl(from_table.get().clone());
        }

        let to_table = unsafe {
            self.to_table
                .cast_bound_unchecked::<crate::common::PyTableName>(py)
        };
        stmt.to_tbl(to_table.get().clone());

        for c in &self.from_columns {
            stmt.from_col(sea_query::Alias::new(c));
        }

        for c in &self.to_columns {
            stmt.to_col(sea_query::Alias::new(c));
        }

        if let Some(x) = self.on_delete {
            stmt.on_delete(x.into());
        }
        if let Some(x) = self.on_update {
            stmt.on_update(x.into());
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "ForeignKey", frozen)]
pub struct PyForeignKey {
    pub inner: parking_lot::Mutex<ForeignKeyInner>,
}

#[pyo3::pymethods]
impl PyForeignKey {
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
        on_delete: Option<String>,
        on_update: Option<String>,
    ) -> pyo3::PyResult<Self> {
        let py = to_table.py();

        let on_delete = match on_delete {
            None => None,
            Some(x) => Some(ForeignKeyActionAlias::from_str(&x)?),
        };

        let on_update = match on_update {
            None => None,
            Some(x) => Some(ForeignKeyActionAlias::from_str(&x)?),
        };

        let to_table: pyo3::Py<pyo3::PyAny> = crate::common::PyTableName::from_pyobject(to_table)?;

        let from_table: Option<pyo3::Py<pyo3::PyAny>> = {
            match from_table {
                Some(from_table) => Some(crate::common::PyTableName::from_pyobject(from_table)?),
                None => None,
            }
        };

        let name = match name {
            Some(x) => x,
            None => {
                let to_table_name = unsafe {
                    let bound = to_table.cast_bound_unchecked::<crate::common::PyTableName>(py);

                    bound.get().name.to_string()
                };

                let from_table_name = match &from_table {
                    Some(x) => unsafe {
                        let bound = x.cast_bound_unchecked::<crate::common::PyTableName>(py);

                        bound.get().name.to_string()
                    },
                    None => String::new(),
                };

                let mut s = format!("fk_{from_table_name}");

                for col in from_columns.iter() {
                    s.push('_');
                    s += col;
                }

                s.push('_');
                s += &to_table_name;

                for col in to_columns.iter() {
                    s.push('_');
                    s += col;
                }

                s
            }
        };

        if from_columns.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "from_columns is empty",
            ));
        }
        if to_columns.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "to_columns is empty",
            ));
        }

        if from_columns.len() != to_columns.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "from_columns and to_columns must have same length ({} != {})",
                from_columns.len(),
                to_columns.len()
            )));
        }

        Ok(Self {
            inner: parking_lot::Mutex::new(ForeignKeyInner {
                name,
                to_table,
                to_columns,
                from_table,
                from_columns,
                on_delete,
                on_update,
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
    #[allow(clippy::wrong_self_convention)]
    fn from_table(&self, py: pyo3::Python) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.inner.lock().from_table.as_ref().map(|x| x.clone_ref(py))
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
    #[allow(clippy::wrong_self_convention)]
    fn from_columns(&self) -> Vec<String> {
        self.inner.lock().from_columns.clone()
    }

    #[setter]
    fn set_from_columns(&self, val: Vec<String>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();

        if val.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "from_columns cannot be empty",
            ));
        }

        if lock.to_columns.len() != val.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "from_columns and to_columns must have same length ({} != {})",
                val.len(),
                lock.to_columns.len(),
            )));
        }

        lock.from_columns = val;
        Ok(())
    }

    #[getter]
    fn to_columns(&self) -> Vec<String> {
        self.inner.lock().to_columns.clone()
    }

    #[setter]
    fn set_to_columns(&self, val: Vec<String>) -> pyo3::PyResult<()> {
        let mut lock = self.inner.lock();

        if val.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "to_columns cannot be empty",
            ));
        }

        if lock.from_columns.len() != val.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "from_columns and to_columns must have same length ({} != {})",
                lock.from_columns.len(),
                val.len()
            )));
        }

        lock.to_columns = val;
        Ok(())
    }

    #[getter]
    fn on_delete(&self) -> Option<String> {
        self.inner.lock().on_delete.map(|x| x.to_string())
    }

    #[setter]
    fn set_on_delete(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let val = match val {
            None => None,
            Some(x) => Some(ForeignKeyActionAlias::from_str(&x)?),
        };

        let mut lock = self.inner.lock();
        lock.on_delete = val;

        Ok(())
    }

    #[getter]
    fn on_update(&self) -> Option<String> {
        self.inner.lock().on_update.map(|x| x.to_string())
    }

    #[setter]
    fn set_on_update(&self, val: Option<String>) -> pyo3::PyResult<()> {
        let val = match val {
            None => None,
            Some(x) => Some(ForeignKeyActionAlias::from_str(&x)?),
        };

        let mut lock = self.inner.lock();
        lock.on_update = val;

        Ok(())
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

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();
        let mut s = Vec::with_capacity(50);

        write!(
            s,
            "<ForeignKey {:?} to_table={} to_columns={:?} from_columns={:?}",
            lock.name, lock.to_table, lock.to_columns, lock.from_columns
        )
        .unwrap();

        if let Some(x) = &lock.from_table {
            write!(s, " from_table={}", x).unwrap();
        }
        if let Some(x) = &lock.on_delete {
            write!(s, " on_delete={:?}", x.to_string()).unwrap();
        }
        if let Some(x) = &lock.on_update {
            write!(s, " on_update={:?}", x.to_string()).unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
