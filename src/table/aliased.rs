use sea_query::IntoIden;

#[pyo3::pyclass(module = "rapidquery._lib", name = "_AliasedTableColumnsSequence", frozen)]
#[allow(non_camel_case_types)]
pub struct Py_AliasedTableColumnsSequence {
    pub inner: std::sync::Arc<parking_lot::Mutex<super::table::TableInner>>,
    pub alias: sea_query::DynIden,
}

#[pyo3::pymethods]
impl Py_AliasedTableColumnsSequence {
    fn __getattr__(slf: pyo3::PyRef<'_, Self>, name: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let col_name = unsafe {
            let lock = slf.inner.lock();

            let (name, _) = lock
                .columns
                .iter()
                .find(|(x, _)| x.eq(&name))
                .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyKeyError, _>(name.to_owned()))?;

            sea_query::Alias::new(name)
        };

        let result = crate::common::PyColumnRef {
            col: crate::common::ColumnNameOrAstrisk::Name(col_name.into_iden()),
            table: Some(slf.alias.clone()),
            schema: None,
        };
        pyo3::Py::new(slf.py(), result).map(|x| x.into_any())
    }

    fn get(slf: pyo3::PyRef<'_, Self>, name: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let col_name = unsafe {
            let lock = slf.inner.lock();

            let (name, _) = lock
                .columns
                .iter()
                .find(|(x, _)| x.eq(&name))
                .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyKeyError, _>(name.to_owned()))?;

            sea_query::Alias::new(name)
        };

        let result = crate::common::PyColumnRef {
            col: crate::common::ColumnNameOrAstrisk::Name(col_name.into_iden()),
            table: Some(slf.alias.clone()),
            schema: None,
        };
        pyo3::Py::new(slf.py(), result).map(|x| x.into_any())
    }

    fn __len__(slf: pyo3::PyRef<'_, Self>) -> usize {
        let lock = slf.inner.lock();
        lock.columns.len()
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "AliasedTable", frozen)]
pub struct PyAliasedTable {
    pub inner: std::sync::Arc<parking_lot::Mutex<super::table::TableInner>>,
    pub alias: sea_query::DynIden,
}

#[pyo3::pymethods]
impl PyAliasedTable {
    #[new]
    fn new(table: &pyo3::Bound<'_, pyo3::PyAny>, alias: String) -> pyo3::PyResult<Self> {
        let alias = sea_query::Alias::new(alias).into_iden();

        if let Ok(table) = table.cast::<super::table::PyTable>() {
            let inner = table.get();
            Ok(Self {
                inner: std::sync::Arc::clone(&inner.inner),
                alias,
            })
        } else if let Ok(table) = table.cast::<Self>() {
            let inner = table.get();

            Ok(Self {
                inner: std::sync::Arc::clone(&inner.inner),
                alias,
            })
        } else {
            Err(typeerror!(
                "expected Table or AliasedTable, got {:?}",
                table.py(),
                table.as_ptr()
            ))
        }
    }

    #[getter]
    pub fn name(&self, py: pyo3::Python) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let mut x = unsafe {
            let lock = self.inner.lock();
            let name = lock.name.cast_bound_unchecked::<crate::common::PyTableName>(py);
            name.get().clone()
        };

        x.alias = Some(self.alias.clone());
        pyo3::Py::new(py, x).map(|x| x.into_any())
    }

    #[getter]
    fn columns(&self, py: pyo3::Python) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let map = Py_AliasedTableColumnsSequence {
            inner: std::sync::Arc::clone(&self.inner),
            alias: self.alias.clone(),
        };
        pyo3::Py::new(py, map).map(|x| x.into_any())
    }

    #[getter]
    fn c(&self, py: pyo3::Python) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let map = Py_AliasedTableColumnsSequence {
            inner: std::sync::Arc::clone(&self.inner),
            alias: self.alias.clone(),
        };
        pyo3::Py::new(py, map).map(|x| x.into_any())
    }

    fn __repr__(&self, py: pyo3::Python) -> String {
        format!("<AliasedTable name={}>", self.name(py).unwrap(),)
    }
}
