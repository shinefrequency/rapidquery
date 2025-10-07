use crate::parameters::OptionalParam;
use pyo3::types::PyAnyMethods;

/// Converting column types into sea_query::ColumnType
pub mod convert;

/// Column type implementations
pub mod types;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ColumnOptions {
    PrimaryKey = 1 << 0,
    UniqueKey = 1 << 1,
    Null = 1 << 2,
    NotNull = 1 << 3,
    AutoIncrement = 1 << 4,
    StoredGenerated = 1 << 5,
}

pub struct ColumnFields {
    pub name: String,
    pub r#type: pyo3::Py<pyo3::PyAny>,
    pub options: u8,
    pub default: Option<pyo3::Py<pyo3::PyAny>>,
    pub generated: Option<pyo3::Py<pyo3::PyAny>>,
    pub extra: Option<String>,
    pub comment: Option<String>,
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "Column", frozen)]
pub struct PyColumn {
    pub(crate) inner: parking_lot::Mutex<ColumnFields>,
}

#[pyo3::pymethods]
impl PyColumn {
    #[new]
    #[pyo3(
        signature=(
            name,
            r#type,
            primary_key=false,
            unique=false,
            nullable=OptionalParam::Undefined,
            auto_increment=false,
            extra=None,
            comment=None,
            default=OptionalParam::Undefined,
            generated=OptionalParam::Undefined,
            stored_generated=false,
        )
    )]
    fn new(
        name: String,
        r#type: &pyo3::Bound<'_, pyo3::PyAny>,
        primary_key: bool,
        unique: bool,
        nullable: OptionalParam,
        auto_increment: bool,
        extra: Option<String>,
        comment: Option<String>,
        default: OptionalParam,
        generated: OptionalParam,
        stored_generated: bool,
    ) -> pyo3::PyResult<Self> {
        if !r#type.is_instance_of::<types::PyColumnTypeMeta>() {
            return Err(typeerror!(
                "expected ColumnTypeMeta for type, got {}",
                r#type.py(),
                r#type.as_ptr()
            ));
        }

        let mut options = ((primary_key as u8) * (ColumnOptions::PrimaryKey as u8))
            | ((unique as u8) * (ColumnOptions::UniqueKey as u8))
            | ((auto_increment as u8) * (ColumnOptions::AutoIncrement as u8))
            | ((stored_generated as u8) * (ColumnOptions::StoredGenerated as u8));

        if let OptionalParam::Defined(x) = &nullable {
            if unsafe { pyo3::ffi::PyBool_Check(x.as_ptr()) == 0 } {
                return Err(typeerror!(
                    "expected bool for nullable, got {}",
                    r#type.py(),
                    x.as_ptr()
                ));
            }

            if unsafe { x.as_ptr() == pyo3::ffi::Py_True() } {
                options |= ColumnOptions::Null as u8;
            } else {
                options |= ColumnOptions::NotNull as u8;
            }
        }

        let default_expr = {
            match default {
                OptionalParam::Undefined => None,
                OptionalParam::Defined(x) => Some(
                    crate::expression::PyExpr::try_with_specific_type(x, Some(r#type))?,
                ),
            }
        };

        let generated_expr = {
            match generated {
                OptionalParam::Undefined => None,
                OptionalParam::Defined(x) => Some(crate::expression::PyExpr::try_from(x)?),
            }
        };

        let py = r#type.py();
        let inner = ColumnFields {
            name,
            r#type: r#type.clone().unbind(),
            options,
            default: default_expr.map(|x| pyo3::Py::new(py, x).unwrap().into_any()),
            generated: generated_expr.map(|x| pyo3::Py::new(py, x).unwrap().into_any()),
            extra,
            comment,
        };

        Ok(PyColumn {
            inner: parking_lot::Mutex::new(inner),
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
    fn r#type(slf: pyo3::PyRef<'_, Self>) -> pyo3::Py<pyo3::PyAny> {
        slf.inner.lock().r#type.clone_ref(slf.py())
    }

    #[setter]
    fn set_type(&self, val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        if !val.is_instance_of::<types::PyColumnTypeMeta>() {
            return Err(typeerror!(
                "expected ColumnTypeMeta for type, got {}",
                val.py(),
                val.as_ptr()
            ));
        }

        let mut lock = self.inner.lock();
        lock.r#type = val.clone().unbind();

        Ok(())
    }

    #[getter]
    fn primary_key(&self) -> bool {
        (self.inner.lock().options & (ColumnOptions::PrimaryKey as u8)) > 0
    }

    #[setter]
    fn set_primary_key(&self, val: bool) {
        let mut lock = self.inner.lock();
        if val {
            lock.options |= ColumnOptions::PrimaryKey as u8;
        } else {
            lock.options &= !(ColumnOptions::PrimaryKey as u8);
        }
    }

    #[getter]
    fn nullable(&self) -> bool {
        (self.inner.lock().options & (ColumnOptions::NotNull as u8)) <= 0
    }

    #[setter]
    fn set_nullable(&self, val: bool) {
        let mut lock = self.inner.lock();
        if val {
            lock.options |= ColumnOptions::Null as u8;
            lock.options &= !(ColumnOptions::NotNull as u8);
        } else {
            lock.options |= ColumnOptions::NotNull as u8;
            lock.options &= !(ColumnOptions::Null as u8);
        }
    }

    #[getter]
    fn unique(&self) -> bool {
        (self.inner.lock().options & (ColumnOptions::NotNull as u8)) <= 0
    }

    #[setter]
    fn set_unique(&self, val: bool) {
        let mut lock = self.inner.lock();
        if val {
            lock.options |= ColumnOptions::UniqueKey as u8;
        } else {
            lock.options &= !(ColumnOptions::UniqueKey as u8);
        }
    }

    #[getter]
    fn stored_generated(&self) -> bool {
        (self.inner.lock().options & (ColumnOptions::StoredGenerated as u8)) > 0
    }

    #[setter]
    fn set_stored_generated(&self, val: bool) {
        let mut lock = self.inner.lock();
        if val {
            lock.options |= ColumnOptions::StoredGenerated as u8;
        } else {
            lock.options &= !(ColumnOptions::StoredGenerated as u8);
        }
    }

    #[getter]
    fn extra(&self) -> Option<String> {
        self.inner.lock().extra.clone()
    }

    #[setter]
    fn set_extra(&self, val: Option<String>) {
        let mut lock = self.inner.lock();
        lock.extra = val;
    }

    #[getter]
    fn comment(&self) -> Option<String> {
        self.inner.lock().comment.clone()
    }

    #[setter]
    fn set_comment(&self, val: Option<String>) {
        let mut lock = self.inner.lock();
        lock.comment = val;
    }

    #[getter]
    fn default(slf: pyo3::PyRef<'_, Self>) -> Option<pyo3::Py<pyo3::PyAny>> {
        slf.inner
            .lock()
            .default
            .as_ref()
            .map(|x| x.clone_ref(slf.py()))
    }

    #[setter]
    fn set_default(&self, val: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let py = val.py();
        let default_expr = crate::expression::PyExpr::try_from(val)?;

        let mut lock = self.inner.lock();
        lock.default = Some(pyo3::Py::new(py, default_expr).unwrap().into_any());

        Ok(())
    }

    #[getter]
    fn generated(slf: pyo3::PyRef<'_, Self>) -> Option<pyo3::Py<pyo3::PyAny>> {
        slf.inner
            .lock()
            .generated
            .as_ref()
            .map(|x| x.clone_ref(slf.py()))
    }

    #[setter]
    fn set_generated(&self, val: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let py = val.py();
        let generated_expr = crate::expression::PyExpr::try_from(val)?;

        let mut lock = self.inner.lock();
        lock.generated = Some(pyo3::Py::new(py, generated_expr).unwrap().into_any());

        Ok(())
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();

        let mut s: Vec<u8> = Vec::with_capacity(20);
        write!(&mut s, "Column({:?}, {}", lock.name, lock.r#type).unwrap();

        if lock.options & (ColumnOptions::PrimaryKey as u8) > 0 {
            write!(&mut s, ", primary_key=True").unwrap();
        }
        if lock.options & (ColumnOptions::Null as u8) > 0 {
            write!(&mut s, ", nullable=True").unwrap();
        }
        if lock.options & (ColumnOptions::NotNull as u8) > 0 {
            write!(&mut s, ", nullable=False").unwrap();
        }
        if lock.options & (ColumnOptions::UniqueKey as u8) > 0 {
            write!(&mut s, ", unique=True").unwrap();
        }
        if lock.options & (ColumnOptions::AutoIncrement as u8) > 0 {
            write!(&mut s, ", auto_increment=True").unwrap();
        }
        if let Some(x) = &lock.extra {
            write!(&mut s, ", extra={x:?}").unwrap();
        }
        if let Some(x) = &lock.comment {
            write!(&mut s, ", comment={x:?}").unwrap();
        }
        if let Some(x) = &lock.default {
            write!(&mut s, ", default={x}").unwrap();
        }
        if let Some(x) = &lock.generated {
            write!(&mut s, ", generated={x}").unwrap();
        }
        if lock.options & (ColumnOptions::StoredGenerated as u8) > 0 {
            write!(&mut s, ", stored_generated=True").unwrap();
        }

        write!(&mut s, ")").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
