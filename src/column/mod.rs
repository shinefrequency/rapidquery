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

pub struct ColumnFields {
    // TODO: add table name here
    pub name: String,
    pub r#type: pyo3::Py<pyo3::PyAny>,
    pub options: u8,
    pub default: Option<pyo3::Py<pyo3::PyAny>>,
    pub generated: Option<pyo3::Py<pyo3::PyAny>>,
    pub extra: Option<String>,
    pub comment: Option<String>,
}

impl ColumnFields {
    #[inline]
    #[optimize(speed)]
    pub fn as_column_ref(&self) -> sea_query::ColumnRef {
        sea_query::ColumnRef::Column(sea_query::Alias::new(&self.name).into_iden())
    }

    #[inline]
    #[optimize(speed)]
    pub fn as_simple_expr(&self) -> sea_query::SimpleExpr {
        sea_query::SimpleExpr::Column(self.as_column_ref())
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
        }
    }
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

    #[inline]
    #[optimize(speed)]
    fn to_column_ref(&self) -> crate::common::PyColumnRef {
        let lock = self.inner.lock();
        lock.as_column_ref().into()
    }

    fn to_expr(&self) -> crate::expression::PyExpr {
        let lock = self.inner.lock();
        lock.as_simple_expr().into()
    }

    fn cast_as(&self, value: String) -> crate::expression::PyExpr {
        let lock = self.inner.lock();
        let expr =
            sea_query::ExprTrait::cast_as(lock.as_simple_expr(), sea_query::Alias::new(value));
        expr.into()
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn like(&self, pattern: String, escape: Option<char>) -> crate::expression::PyExpr {
        let e = sea_query::LikeExpr::new(pattern);

        let expr = self.inner.lock().as_simple_expr();

        if let Some(x) = escape {
            sea_query::ExprTrait::like(expr, e.escape(x)).into()
        } else {
            sea_query::ExprTrait::like(expr, e).into()
        }
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn not_like(&self, pattern: String, escape: Option<char>) -> crate::expression::PyExpr {
        let e = sea_query::LikeExpr::new(pattern);

        let expr = self.inner.lock().as_simple_expr();

        if let Some(x) = escape {
            sea_query::ExprTrait::like(expr, e.escape(x)).into()
        } else {
            sea_query::ExprTrait::like(expr, e).into()
        }
    }

    fn __eq__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::eq(expr, expr2).into()
    }

    fn __ne__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::ne(expr, expr2).into()
    }

    fn __gt__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::gt(expr, expr2).into()
    }

    fn __ge__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::gte(expr, expr2).into()
    }

    fn __lt__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::lt(expr, expr2).into()
    }

    fn __le__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::lte(expr, expr2).into()
    }

    fn __add__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::add(expr, expr2).into()
    }

    fn __sub__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::sub(expr, expr2).into()
    }

    fn __and__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::bit_and(expr, expr2).into()
    }

    fn __or__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::bit_or(expr, expr2).into()
    }

    fn __truediv__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::div(expr, expr2).into()
    }

    fn is_(&self, other: &pyo3::Bound<'_, crate::expression::PyExpr>) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::is(expr, expr2).into()
    }

    fn is_not(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::is_not(expr, expr2).into()
    }

    fn is_null(&self) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();

        sea_query::ExprTrait::is_null(expr).into()
    }

    fn is_not_null(&self) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();

        sea_query::ExprTrait::is_not_null(expr).into()
    }

    fn __lshift__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::left_shift(expr, expr2).into()
    }

    fn __rshift__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::right_shift(expr, expr2).into()
    }

    fn __mod__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::modulo(expr, expr2).into()
    }

    fn __mul__(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::ExprTrait::mul(expr, expr2).into()
    }

    fn pg_concat(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::extension::postgres::PgExpr::concat(expr, expr2).into()
    }

    fn pg_contained(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::extension::postgres::PgExpr::contained(expr, expr2).into()
    }

    fn get_json_field(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::extension::postgres::PgExpr::get_json_field(expr, expr2).into()
    }

    fn cast_json_field(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::extension::postgres::PgExpr::cast_json_field(expr, expr2).into()
    }

    fn pg_contains(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::extension::postgres::PgExpr::contains(expr, expr2).into()
    }

    fn pg_matches(
        &self,
        other: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = other.get().inner.lock().clone();

        sea_query::extension::postgres::PgExpr::matches(expr, expr2).into()
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn pg_ilike(&self, pattern: String, escape: Option<char>) -> crate::expression::PyExpr {
        let e = sea_query::LikeExpr::new(pattern);

        let expr = self.inner.lock().as_simple_expr();

        if let Some(x) = escape {
            sea_query::extension::postgres::PgExpr::ilike(expr, e.escape(x)).into()
        } else {
            sea_query::extension::postgres::PgExpr::ilike(expr, e).into()
        }
    }

    #[pyo3(signature=(pattern, escape=None))]
    fn pg_not_ilike(&self, pattern: String, escape: Option<char>) -> crate::expression::PyExpr {
        let e = sea_query::LikeExpr::new(pattern);

        let expr = self.inner.lock().as_simple_expr();

        if let Some(x) = escape {
            sea_query::extension::postgres::PgExpr::not_ilike(expr, e.escape(x)).into()
        } else {
            sea_query::extension::postgres::PgExpr::not_ilike(expr, e).into()
        }
    }

    fn between(
        &self,
        a: &pyo3::Bound<'_, crate::expression::PyExpr>,
        b: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = a.get().inner.lock().clone();
        let expr3 = b.get().inner.lock().clone();

        sea_query::ExprTrait::between(expr, expr2, expr3).into()
    }

    fn not_between(
        &self,
        a: &pyo3::Bound<'_, crate::expression::PyExpr>,
        b: &pyo3::Bound<'_, crate::expression::PyExpr>,
    ) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();
        let expr2 = a.get().inner.lock().clone();
        let expr3 = b.get().inner.lock().clone();

        sea_query::ExprTrait::not_between(expr, expr2, expr3).into()
    }

    fn in_(&self, other: Vec<pyo3::Py<crate::expression::PyExpr>>) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();

        sea_query::ExprTrait::is_in(
            expr,
            other.into_iter().map(|x| x.get().inner.lock().clone()),
        )
        .into()
    }

    fn not_in(&self, other: Vec<pyo3::Py<crate::expression::PyExpr>>) -> crate::expression::PyExpr {
        let expr = self.inner.lock().as_simple_expr();

        sea_query::ExprTrait::is_not_in(
            expr,
            other.into_iter().map(|x| x.get().inner.lock().clone()),
        )
        .into()
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
