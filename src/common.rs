use sea_query::IntoIden;
use std::str::FromStr;

/// Asterisk type - very useful for expression creating
#[pyo3::pyclass(module = "rapidorm_core.builtins", name = "_AsteriskType", frozen)]
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
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>("invalid format"));
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

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        (slf.as_ptr() == other.as_ptr())
            || (slf.col == other.col && slf.schema == other.schema && slf.table == other.table)
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> bool {
        (slf.as_ptr() != other.as_ptr())
            && (slf.col == other.col && slf.schema == other.schema && slf.table == other.table)
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        format!(
            "ColumnRef({}, table={:?}, schema={:?})",
            slf.col,
            slf.table.as_ref().map(|x| x.to_string()),
            slf.schema.as_ref().map(|x| x.to_string()),
        )
    }
}
