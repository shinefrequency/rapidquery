use pyo3::types::PyAnyMethods;
use sea_query::IntoIden;
use std::str::FromStr;

/// Asterisk type - very useful for expression creating
#[pyo3::pyclass(module = "rapidquery._lib", name = "_AsteriskType", frozen)]
pub struct PyAsteriskType {}

#[derive(Clone, PartialEq, Debug)]
pub enum ColumnNameOrAstrisk {
    Astrisk,
    Name(sea_query::DynIden),
}

impl std::fmt::Display for ColumnNameOrAstrisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnNameOrAstrisk::Astrisk => write!(f, "*"),
            ColumnNameOrAstrisk::Name(x) => write!(f, "{:?}", x.to_string()),
        }
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "ColumnRef", frozen)]
#[derive(Clone)]
pub struct PyColumnRef {
    pub col: ColumnNameOrAstrisk,
    pub table: Option<sea_query::DynIden>,
    pub schema: Option<sea_query::DynIden>,
}

impl sea_query::IntoColumnRef for PyColumnRef {
    fn into_column_ref(self) -> sea_query::ColumnRef {
        if let ColumnNameOrAstrisk::Name(name) = self.col {
            match (self.table, self.schema) {
                (Some(table), Some(schema)) => sea_query::ColumnRef::SchemaTableColumn(schema, table, name),
                (Some(table), None) => sea_query::ColumnRef::TableColumn(table, name),
                _ => sea_query::ColumnRef::Column(name),
            }
        } else if let Some(table) = self.table {
            sea_query::ColumnRef::TableAsterisk(table)
        } else {
            sea_query::ColumnRef::Asterisk
        }
    }
}

impl From<sea_query::ColumnRef> for PyColumnRef {
    fn from(value: sea_query::ColumnRef) -> Self {
        match value {
            sea_query::ColumnRef::Asterisk => Self {
                col: ColumnNameOrAstrisk::Astrisk,
                table: None,
                schema: None,
            },
            sea_query::ColumnRef::TableAsterisk(table) => Self {
                col: ColumnNameOrAstrisk::Astrisk,
                table: Some(table),
                schema: None,
            },
            sea_query::ColumnRef::SchemaTableColumn(schema, table, name) => Self {
                col: ColumnNameOrAstrisk::Name(name),
                table: Some(table),
                schema: Some(schema),
            },
            sea_query::ColumnRef::TableColumn(table, name) => Self {
                col: ColumnNameOrAstrisk::Name(name),
                table: Some(table),
                schema: None,
            },
            sea_query::ColumnRef::Column(name) => Self {
                col: ColumnNameOrAstrisk::Name(name),
                table: None,
                schema: None,
            },
        }
    }
}

impl FromStr for PyColumnRef {
    type Err = pyo3::PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_owned();
        if s.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "cannot parse an empty string",
            ));
        }

        // Possible formats:
        //    name
        //    table.name
        //    schema.table.name
        let mut s = s.split('.').map(String::from).collect::<Vec<String>>();

        if s.len() > 3 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "invalid format",
            ));
        }

        let name = s.pop().unwrap();
        let table = s.pop();
        let schema = s.pop();

        Ok(Self {
            col: if name == "*" {
                ColumnNameOrAstrisk::Astrisk
            } else {
                ColumnNameOrAstrisk::Name(sea_query::Alias::new(name).into_iden())
            },
            table: table.map(|x| sea_query::Alias::new(x).into_iden()),
            schema: schema.map(|x| sea_query::Alias::new(x).into_iden()),
        })
    }
}

#[pyo3::pymethods]
impl PyColumnRef {
    #[new]
    #[pyo3(signature=(name, table=None, schema=None))]
    fn new(name: String, table: Option<String>, schema: Option<String>) -> Self {
        Self {
            col: ColumnNameOrAstrisk::Name(sea_query::Alias::new(name).into_iden()),
            table: table.map(|x| sea_query::Alias::new(x).into_iden()),
            schema: schema.map(|x| sea_query::Alias::new(x).into_iden()),
        }
    }

    #[getter]
    fn name(&self) -> String {
        match &self.col {
            ColumnNameOrAstrisk::Astrisk => String::from("*"),
            ColumnNameOrAstrisk::Name(x) => x.to_string(),
        }
    }

    #[getter]
    fn table(&self) -> Option<String> {
        self.table.as_ref().map(|x| x.to_string())
    }

    #[getter]
    fn schema(&self) -> Option<String> {
        self.schema.as_ref().map(|x| x.to_string())
    }

    #[classmethod]
    fn parse(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, string: String) -> pyo3::PyResult<Self> {
        Self::from_str(&string)
    }

    #[pyo3(signature=(**kwds))]
    fn copy_with(&self, kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>) -> pyo3::PyResult<Self> {
        use pyo3::types::PyDictMethods;

        let mut cloned = self.clone();
        if kwds.is_none() {
            return Ok(cloned);
        }

        let kwds = unsafe { kwds.unwrap_unchecked() };

        for (key, val) in kwds.iter() {
            #[cfg(debug_assertions)]
            let key = key.extract::<String>().unwrap();

            #[cfg(not(debug_assertions))]
            let key = unsafe { key.extract::<String>().unwrap_unchecked() };

            // All of values are Option<string>
            let val = unsafe {
                if pyo3::ffi::Py_IsNone(val.as_ptr()) == 1 {
                    None
                } else if pyo3::ffi::PyUnicode_CheckExact(val.as_ptr()) == 1 {
                    Some(val.extract::<String>().unwrap_unchecked())
                } else {
                    return Err(typeerror!(
                        "expected str or None, got {:?}",
                        val.py(),
                        val.as_ptr()
                    ));
                }
            };

            if key == "name" {
                if let Some(x) = val {
                    // Ignore name=None
                    cloned.col = ColumnNameOrAstrisk::Name(sea_query::Alias::new(x).into_iden());
                }
            } else if key == "table" {
                cloned.table = val.map(|x| sea_query::Alias::new(x).into_iden());
            } else if key == "schema" {
                cloned.schema = val.map(|x| sea_query::Alias::new(x).into_iden());
            } else {
                return Err(typeerror!(format!(
                    "got an unexpected keyword argument '{}'",
                    key
                ),));
            }
        }

        Ok(cloned)
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        if slf.as_ptr() == other.as_ptr() {
            return true;
        }

        slf.col == other.col && slf.schema == other.schema && slf.table == other.table
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        if slf.as_ptr() == other.as_ptr() {
            return false;
        }

        slf.col != other.col || slf.schema != other.schema || slf.table != other.table
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s = Vec::new();

        write!(s, "<ColumnRef {}", self.col).unwrap();
        if let Some(x) = &self.table {
            write!(s, " table={:?}", x.to_string()).unwrap();
        }
        if let Some(x) = &self.schema {
            write!(s, " schema={:?}", x.to_string()).unwrap();
        }

        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "TableName", frozen)]
#[derive(Clone)]
pub struct PyTableName {
    pub name: sea_query::DynIden,
    pub schema: Option<sea_query::DynIden>,
    pub database: Option<sea_query::DynIden>,
    pub alias: Option<sea_query::DynIden>,
}

impl sea_query::IntoTableRef for PyTableName {
    fn into_table_ref(self) -> sea_query::TableRef {
        match (self.schema, self.database, self.alias) {
            (Some(schema), Some(database), Some(alias)) => {
                sea_query::TableRef::DatabaseSchemaTableAlias(database, schema, self.name, alias)
            }
            (Some(schema), None, Some(alias)) => {
                sea_query::TableRef::SchemaTableAlias(schema, self.name, alias)
            }
            (Some(schema), Some(database), None) => {
                sea_query::TableRef::DatabaseSchemaTable(database, schema, self.name)
            }
            (Some(schema), None, None) => sea_query::TableRef::SchemaTable(schema, self.name),
            (None, None, Some(alias)) => sea_query::TableRef::TableAlias(self.name, alias),
            _ => sea_query::TableRef::Table(self.name),
        }
    }
}

impl TryFrom<sea_query::TableRef> for PyTableName {
    type Error = ();

    fn try_from(value: sea_query::TableRef) -> Result<Self, Self::Error> {
        match value {
            sea_query::TableRef::DatabaseSchemaTableAlias(db, schema, name, alias) => Ok(Self {
                name,
                schema: Some(schema),
                database: Some(db),
                alias: Some(alias),
            }),
            sea_query::TableRef::SchemaTableAlias(schema, name, alias) => Ok(Self {
                name,
                schema: Some(schema),
                database: None,
                alias: Some(alias),
            }),
            sea_query::TableRef::TableAlias(name, alias) => Ok(Self {
                name,
                schema: None,
                database: None,
                alias: Some(alias),
            }),
            sea_query::TableRef::DatabaseSchemaTable(db, schema, name) => Ok(Self {
                name,
                schema: Some(schema),
                database: Some(db),
                alias: None,
            }),
            sea_query::TableRef::SchemaTable(schema, name) => Ok(Self {
                name,
                schema: Some(schema),
                database: None,
                alias: None,
            }),
            sea_query::TableRef::Table(name) => Ok(Self {
                name,
                schema: None,
                database: None,
                alias: None,
            }),
            _ => Err(()),
        }
    }
}

impl FromStr for PyTableName {
    type Err = pyo3::PyErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "cannot parse an empty string",
            ));
        }

        // Possible formats:
        //    name
        //    schema.name
        //    database.schema.name
        let mut s = s.split('.').map(String::from).collect::<Vec<String>>();

        if s.len() > 3 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "invalid format",
            ));
        }

        let name = s.pop().map(|x| sea_query::Alias::new(x).into_iden()).unwrap();
        let schema = s.pop().map(|x| sea_query::Alias::new(x).into_iden());
        let database = s.pop().map(|x| sea_query::Alias::new(x).into_iden());

        Ok(Self {
            name,
            schema,
            database,
            alias: None,
        })
    }
}

impl PyTableName {
    pub fn from_pyobject(value: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(value.as_ptr()) == crate::typeref::TABLE_NAME_TYPE {
                Ok(value.clone().unbind())
            } else if let Ok(x) = value.extract::<String>() {
                let tb = crate::common::PyTableName::from_str(&x)?;

                Ok(pyo3::Py::new(value.py(), tb)?.into_any())
            } else {
                Err(typeerror!(
                    "expected TableName or str, got {:?}",
                    value.py(),
                    value.as_ptr()
                ))
            }
        }
    }
}

#[pyo3::pymethods]
impl PyTableName {
    #[new]
    #[pyo3(signature=(name, schema=None, database=None, alias=None))]
    fn new(name: String, schema: Option<String>, database: Option<String>, alias: Option<String>) -> Self {
        Self {
            name: sea_query::Alias::new(name).into_iden(),
            schema: schema.map(|x| sea_query::Alias::new(x).into_iden()),
            database: database.map(|x| sea_query::Alias::new(x).into_iden()),
            alias: alias.map(|x| sea_query::Alias::new(x).into_iden()),
        }
    }

    #[classmethod]
    fn parse(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, string: String) -> pyo3::PyResult<Self> {
        Self::from_str(&string)
    }

    #[getter]
    fn name(&self) -> String {
        self.name.to_string()
    }

    #[getter]
    fn schema(&self) -> Option<String> {
        self.schema.as_ref().map(|x| x.to_string())
    }

    #[getter]
    fn database(&self) -> Option<String> {
        self.database.as_ref().map(|x| x.to_string())
    }

    #[getter]
    fn alias(&self) -> Option<String> {
        self.alias.as_ref().map(|x| x.to_string())
    }

    #[pyo3(signature=(**kwds))]
    fn copy_with(&self, kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>) -> pyo3::PyResult<Self> {
        use pyo3::types::PyDictMethods;

        let mut cloned = self.clone();
        if kwds.is_none() {
            return Ok(cloned);
        }

        let kwds = unsafe { kwds.unwrap_unchecked() };

        for (key, val) in kwds.iter() {
            #[cfg(debug_assertions)]
            let key = key.extract::<String>().unwrap();

            #[cfg(not(debug_assertions))]
            let key = unsafe { key.extract::<String>().unwrap_unchecked() };

            // All of values are Option<string>
            let val = unsafe {
                if pyo3::ffi::Py_IsNone(val.as_ptr()) == 1 {
                    None
                } else if pyo3::ffi::PyUnicode_CheckExact(val.as_ptr()) == 1 {
                    Some(val.extract::<String>().unwrap_unchecked())
                } else {
                    return Err(typeerror!(
                        "expected str or None, got {:?}",
                        val.py(),
                        val.as_ptr()
                    ));
                }
            };

            if key == "name" {
                if let Some(x) = val {
                    // Ignore name=None
                    cloned.name = sea_query::Alias::new(x).into_iden();
                }
            } else if key == "database" {
                cloned.database = val.map(|x| sea_query::Alias::new(x).into_iden());
            } else if key == "schema" {
                cloned.schema = val.map(|x| sea_query::Alias::new(x).into_iden());
            } else if key == "alias" {
                cloned.alias = val.map(|x| sea_query::Alias::new(x).into_iden());
            } else {
                return Err(typeerror!(format!(
                    "got an unexpected keyword argument '{}'",
                    key
                ),));
            }
        }

        Ok(cloned)
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: &pyo3::Bound<'_, Self>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(true);
        }

        let other = other.get();
        Ok(slf.name == other.name && slf.database == other.database && slf.schema == other.schema)
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: &pyo3::Bound<'_, Self>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(false);
        }

        let other = other.get();
        Ok(slf.name != other.name || slf.database != other.database || slf.schema != other.schema)
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s = Vec::new();

        write!(s, "<TableName {:?}", self.name.to_string()).unwrap();
        if let Some(x) = &self.schema {
            write!(s, " schema={:?}", x.to_string()).unwrap();
        }
        if let Some(x) = &self.database {
            write!(s, " database={:?}", x.to_string()).unwrap();
        }
        if let Some(x) = &self.alias {
            write!(s, " alias={:?}", x.to_string()).unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "IndexColumn", frozen)]
#[derive(Clone)]
pub struct PyIndexColumn {
    pub name: String,
    pub prefix: Option<u32>,
    pub order: Option<sea_query::IndexOrder>,
}

impl sea_query::IntoIndexColumn for PyIndexColumn {
    fn into_index_column(self) -> sea_query::IndexColumn {
        match (self.prefix, self.order) {
            (Some(p), Some(o)) => (sea_query::Alias::new(self.name), p, o).into_index_column(),
            (Some(p), None) => (sea_query::Alias::new(self.name), p).into_index_column(),
            (None, Some(o)) => (sea_query::Alias::new(self.name), o).into_index_column(),
            (None, None) => sea_query::Alias::new(self.name).into_index_column(),
        }
    }
}

impl From<&str> for PyIndexColumn {
    fn from(value: &str) -> Self {
        Self {
            name: value.to_owned(),
            prefix: None,
            order: None,
        }
    }
}

#[pyo3::pymethods]
impl PyIndexColumn {
    #[new]
    #[pyo3(signature=(name, prefix=None, order=None))]
    fn new(name: String, prefix: Option<u32>, order: Option<String>) -> pyo3::PyResult<Self> {
        let order = {
            if let Some(mut x) = order {
                x.make_ascii_lowercase();

                if x == "asc" {
                    Some(sea_query::IndexOrder::Asc)
                } else if x == "desc" {
                    Some(sea_query::IndexOrder::Desc)
                } else {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "invalid order value, expected 'asc', 'desc' or None; got {x:?}"
                    )));
                }
            } else {
                None
            }
        };

        Ok(Self { name, prefix, order })
    }

    #[getter]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[getter]
    fn prefix(&self) -> Option<u32> {
        self.prefix
    }

    #[getter]
    fn order(&self) -> Option<String> {
        self.order
            .clone()
            .map(|x| match x {
                sea_query::IndexOrder::Asc => "asc",
                sea_query::IndexOrder::Desc => "desc",
            })
            .map(String::from)
    }

    fn __copy__(&self) -> Self {
        self.clone()
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s = Vec::new();
        write!(&mut s, "<IndexColumn {:?}", self.name).unwrap();

        if let Some(x) = self.prefix {
            write!(&mut s, " prefix={}", x).unwrap();
        }
        if let Some(x) = &self.order {
            if matches!(x, sea_query::IndexOrder::Asc) {
                write!(&mut s, " order='asc'").unwrap();
            } else {
                write!(&mut s, " order='desc'").unwrap();
            }
        }
        write!(&mut s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
