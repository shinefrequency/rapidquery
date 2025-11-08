use crate::parameters::OptionalParam;
use pyo3::types::PyAnyMethods;
use sea_query::IntoIden;

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

#[derive(Debug)]
#[non_exhaustive]
pub enum LazyColumnRef {
    None,
    TableName(
        // Always is `TableName`
        pyo3::Py<pyo3::PyAny>,
    ),
    ColumnRef(sea_query::ColumnRef),
}

impl LazyColumnRef {
    #[inline]
    #[optimize(speed)]
    fn clone_ref(&self, py: pyo3::Python) -> Self {
        match self {
            Self::None => Self::None,
            Self::ColumnRef(x) => Self::ColumnRef(x.clone()),
            Self::TableName(x) => Self::TableName(x.clone_ref(py)),
        }
    }

    #[inline]
    #[optimize(speed)]
    fn set_name(&mut self, name: &str) {
        match self {
            Self::None => (),
            Self::TableName(_) => (),
            Self::ColumnRef(x) => match x {
                sea_query::ColumnRef::Column(_) => {
                    *x = sea_query::ColumnRef::Column(sea_query::Alias::new(name).into_iden());
                }
                sea_query::ColumnRef::TableColumn(tb, _) => {
                    *x = sea_query::ColumnRef::TableColumn(
                        tb.clone(),
                        sea_query::Alias::new(name).into_iden(),
                    );
                }
                sea_query::ColumnRef::SchemaTableColumn(sc, tb, _) => {
                    *x = sea_query::ColumnRef::SchemaTableColumn(
                        sc.clone(),
                        tb.clone(),
                        sea_query::Alias::new(name).into_iden(),
                    );
                }
                _ => (),
            },
        }
    }

    #[inline]
    #[optimize(speed)]
    fn with_name(&mut self, py: pyo3::Python, name: &str) -> sea_query::ColumnRef {
        match self {
            Self::None => {
                let x = sea_query::ColumnRef::Column(sea_query::Alias::new(name).into_iden());
                *self = Self::ColumnRef(x.clone());
                x
            }
            Self::TableName(tb) => {
                let bound = tb.cast_bound::<crate::common::PyTableName>(py).unwrap();
                let ptr = bound.get();

                let x = {
                    if let Some(schema) = &ptr.schema {
                        sea_query::ColumnRef::SchemaTableColumn(
                            schema.clone(),
                            ptr.name.clone(),
                            sea_query::Alias::new(name).into_iden(),
                        )
                    } else {
                        sea_query::ColumnRef::TableColumn(
                            ptr.name.clone(),
                            sea_query::Alias::new(name).into_iden(),
                        )
                    }
                };

                *self = Self::ColumnRef(x.clone());
                x
            }
            Self::ColumnRef(x) => x.clone(),
        }
    }
}

/// A bridge between Python & [`sea_query::ColumnDef`]
pub struct ColumnInner {
    pub name: String,

    // Always is `ColumnTypeMeta`
    pub r#type: pyo3::Py<pyo3::PyAny>,
    pub options: u8,

    // Always is `Option<Expr>`
    pub default: Option<pyo3::Py<pyo3::PyAny>>,

    // Always is `Option<Expr>`
    pub generated: Option<pyo3::Py<pyo3::PyAny>>,
    pub extra: Option<String>,
    pub comment: Option<String>,
    pub column_ref: LazyColumnRef,
}

impl ColumnInner {
    #[inline]
    #[optimize(speed)]
    pub fn as_column_ref(&mut self, py: pyo3::Python) -> sea_query::ColumnRef {
        self.column_ref.with_name(py, &self.name)
    }

    #[inline]
    #[optimize(speed)]
    pub fn as_simple_expr(&mut self, py: pyo3::Python) -> sea_query::SimpleExpr {
        sea_query::SimpleExpr::Column(self.as_column_ref(py))
    }

    #[inline]
    #[optimize(speed)]
    pub fn as_column_def(&self, py: pyo3::Python<'_>) -> sea_query::ColumnDef {
        let mut column_def = sea_query::ColumnDef::new_with_type(
            sea_query::Alias::new(self.name.clone()),
            #[cfg(debug_assertions)]
            convert::convert_to_column_type(self.r#type.bind(py)).unwrap(),
            #[cfg(not(debug_assertions))]
            unsafe {
                convert::convert_to_column_type(self.r#type.bind(py)).unwrap()
            },
        );

        if self.options & (ColumnOptions::PrimaryKey as u8) > 0 {
            column_def.primary_key();
        }
        if self.options & (ColumnOptions::UniqueKey as u8) > 0 {
            column_def.unique_key();
        }
        if self.options & (ColumnOptions::Null as u8) > 0 {
            column_def.null();
        }
        if self.options & (ColumnOptions::NotNull as u8) > 0 {
            column_def.not_null();
        }
        if self.options & (ColumnOptions::AutoIncrement as u8) > 0 {
            column_def.auto_increment();
        }

        if let Some(default) = &self.default {
            let default_expr = unsafe { default.cast_bound_unchecked::<crate::expression::PyExpr>(py) };

            let default_expr = default_expr.get();
            column_def.default(default_expr.inner.clone());
        }

        if let Some(generated) = &self.generated {
            let generated_expr = unsafe { generated.cast_bound_unchecked::<crate::expression::PyExpr>(py) };

            let generated_expr = generated_expr.get();

            column_def.generated(
                generated_expr.inner.clone(),
                self.options & (ColumnOptions::StoredGenerated as u8) > 0,
            );
        }

        if let Some(x) = &self.extra {
            column_def.extra(x);
        }
        if let Some(x) = &self.comment {
            column_def.comment(x);
        }

        column_def
    }

    pub fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            name: self.name.clone(),
            r#type: self.r#type.clone_ref(py),
            options: self.options,
            default: self.default.as_ref().map(|x| x.clone_ref(py)),
            generated: self.generated.as_ref().map(|x| x.clone_ref(py)),
            extra: self.extra.clone(),
            comment: self.comment.clone(),
            column_ref: self.column_ref.clone_ref(py),
        }
    }
}

/// Defines a table column with its properties and constraints.
///
/// Represents a complete column definition including:
/// - Column name and data type
/// - Constraints (primary key, unique, nullable)
/// - Auto-increment behavior
/// - Default values and generated columns
/// - Comments and extra specifications
#[pyo3::pyclass(module = "rapidquery._lib", name = "Column", frozen, generic, subclass)]
pub struct PyColumn {
    pub(crate) inner: parking_lot::Mutex<ColumnInner>,
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
    #[allow(clippy::too_many_arguments)]
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
                OptionalParam::Defined(x) => Some(crate::expression::PyExpr::try_with_specific_type(
                    x.to_owned(),
                    Some(r#type),
                )?),
            }
        };

        let generated_expr = {
            match generated {
                OptionalParam::Undefined => None,
                OptionalParam::Defined(x) => Some(crate::expression::PyExpr::try_from(x.to_owned())?),
            }
        };

        let py = r#type.py();
        let inner = ColumnInner {
            name,
            r#type: r#type.clone().unbind(),
            options,
            default: default_expr.map(|x| pyo3::Py::new(py, x).unwrap().into_any()),
            generated: generated_expr.map(|x| pyo3::Py::new(py, x).unwrap().into_any()),
            extra,
            comment,
            column_ref: LazyColumnRef::None,
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
        lock.column_ref.set_name(&val);
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
        (self.inner.lock().options & (ColumnOptions::NotNull as u8)) == 0
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
        (self.inner.lock().options & (ColumnOptions::UniqueKey as u8)) > 0
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
    fn auto_increment(&self) -> bool {
        (self.inner.lock().options & (ColumnOptions::AutoIncrement as u8)) > 0
    }

    #[setter]
    fn set_auto_increment(&self, val: bool) {
        let mut lock = self.inner.lock();
        if val {
            lock.options |= ColumnOptions::AutoIncrement as u8;
        } else {
            lock.options &= !(ColumnOptions::AutoIncrement as u8);
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
        slf.inner.lock().default.as_ref().map(|x| x.clone_ref(slf.py()))
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
        slf.inner.lock().generated.as_ref().map(|x| x.clone_ref(slf.py()))
    }

    #[setter]
    fn set_generated(&self, val: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let py = val.py();
        let generated_expr = crate::expression::PyExpr::try_from(val)?;

        let mut lock = self.inner.lock();
        lock.generated = Some(pyo3::Py::new(py, generated_expr).unwrap().into_any());

        Ok(())
    }

    fn to_column_ref(&self, py: pyo3::Python) -> crate::common::PyColumnRef {
        let mut lock = self.inner.lock();
        lock.as_column_ref(py).into()
    }

    fn to_expr(&self, py: pyo3::Python) -> crate::expression::PyExpr {
        let mut lock = self.inner.lock();
        lock.as_simple_expr(py).into()
    }

    fn adapt(
        &self,
        value: pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<crate::adaptation::PyAdaptedValue> {
        let py = value.py();
        let lock = self.inner.lock();
        let value = crate::adaptation::ReturnableValue::from_bound(value, Some(lock.r#type.bind(py)))?;

        Ok(value.into())
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

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();

        let mut s: Vec<u8> = Vec::with_capacity(20);

        write!(s, "<Column {:?} type={}", lock.name, lock.r#type).unwrap();

        if lock.options & (ColumnOptions::PrimaryKey as u8) > 0 {
            write!(s, " primary_key=True").unwrap();
        }
        if lock.options & (ColumnOptions::Null as u8) > 0 {
            write!(s, " nullable=True").unwrap();
        }
        if lock.options & (ColumnOptions::NotNull as u8) > 0 {
            write!(s, " nullable=False").unwrap();
        }
        if lock.options & (ColumnOptions::UniqueKey as u8) > 0 {
            write!(s, " unique=True").unwrap();
        }
        if lock.options & (ColumnOptions::AutoIncrement as u8) > 0 {
            write!(s, " auto_increment=True").unwrap();
        }
        if let Some(x) = &lock.extra {
            write!(s, " extra={x:?}").unwrap();
        }
        if let Some(x) = &lock.comment {
            write!(s, " comment={x:?}").unwrap();
        }
        if let Some(x) = &lock.default {
            write!(s, " default={x}").unwrap();
        }
        if let Some(x) = &lock.generated {
            write!(s, " generated={x}").unwrap();
        }
        if lock.options & (ColumnOptions::StoredGenerated as u8) > 0 {
            write!(s, " stored_generated=True").unwrap();
        }
        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
