use pyo3::types::{PyAnyMethods, PyDictMethods, PyTupleMethods};

pub enum OnConflictUpdate {
    Column(String),
    Expr(
        String,
        // Always is `PyExpr`
        pyo3::Py<pyo3::PyAny>,
    ),
}

#[derive(Default)]
pub enum OnConflictAction {
    #[default]
    None,
    DoNothing(Vec<String>),
    DoUpdate(Vec<OnConflictUpdate>),
}

#[derive(Default)]
pub struct OnConflictInner {
    targets: Vec<String>,
    action: OnConflictAction,

    // Always is `Option<PyExpr>`
    target_where: Option<pyo3::Py<pyo3::PyAny>>,

    // Always is `Option<PyExpr>`
    action_where: Option<pyo3::Py<pyo3::PyAny>>,
}

impl OnConflictInner {
    #[inline]
    #[optimize(speed)]
    pub(super) fn as_statement(&self, py: pyo3::Python) -> sea_query::OnConflict {
        let mut stmt = sea_query::OnConflict::columns(self.targets.iter().map(sea_query::Alias::new));

        match &self.action {
            OnConflictAction::None => (),
            OnConflictAction::DoNothing(x) => {
                if x.is_empty() {
                    stmt.do_nothing();
                } else {
                    stmt.do_nothing_on(x.iter().map(sea_query::Alias::new));
                }
            }
            OnConflictAction::DoUpdate(x) => {
                let mut columns = Vec::new();
                let mut exprs = Vec::new();

                for val in x.iter() {
                    match val {
                        OnConflictUpdate::Column(name) => columns.push(sea_query::Alias::new(name)),
                        OnConflictUpdate::Expr(name, expr) => unsafe {
                            let expr = expr.cast_bound_unchecked::<crate::expression::PyExpr>(py);
                            exprs.push((sea_query::Alias::new(name), expr.get().inner.clone()));
                        },
                    }
                }

                stmt.update_columns(columns);
                stmt.values(exprs);
            }
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "OnConflict", frozen)]
pub struct PyOnConflict {
    pub inner: parking_lot::Mutex<OnConflictInner>,
}

impl PyOnConflict {
    #[inline]
    fn update_from_dictionary<'a>(
        slf: pyo3::PyRef<'a, Self>,
        kwds: &'a pyo3::Bound<'_, pyo3::types::PyDict>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut action = Vec::with_capacity(kwds.len());

        for (key, val) in kwds.iter() {
            unsafe {
                let val = crate::expression::PyExpr::from_bound_into_any(val)?;

                action.push(OnConflictUpdate::Expr(
                    key.extract::<String>().unwrap_unchecked(),
                    val,
                ));
            }
        }

        {
            let mut lock = slf.inner.lock();
            lock.action = OnConflictAction::DoUpdate(action);
        }

        Ok(slf)
    }

    #[inline]
    fn update_from_tuple<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: &'a pyo3::Bound<'_, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut action = Vec::with_capacity(args.len());

        for key in args.iter() {
            unsafe {
                if pyo3::ffi::Py_TYPE(key.as_ptr()) == crate::typeref::COLUMN_TYPE {
                    let col = key.cast_into_unchecked::<crate::column::PyColumn>();
                    action.push(OnConflictUpdate::Column(col.get().inner.lock().name.clone()));
                } else if pyo3::ffi::PyUnicode_CheckExact(key.as_ptr()) == 1 {
                    action.push(OnConflictUpdate::Column(
                        key.extract::<String>().unwrap_unchecked(),
                    ));
                } else {
                    return Err(typeerror!(
                        "expected str or Column, got {:?}",
                        key.py(),
                        key.as_ptr()
                    ));
                }
            }
        }

        {
            let mut lock = slf.inner.lock();
            lock.action = OnConflictAction::DoUpdate(action);
        }

        Ok(slf)
    }
}

#[pyo3::pymethods]
impl PyOnConflict {
    #[new]
    #[pyo3(signature=(*targets))]
    fn new(targets: &pyo3::Bound<'_, pyo3::types::PyTuple>) -> pyo3::PyResult<Self> {
        if targets.is_empty() {
            // Fast path for empty targets
            return Ok(Self {
                inner: parking_lot::Mutex::new(Default::default()),
            });
        }

        let mut tgs: Vec<String> = Vec::with_capacity(targets.len());

        for col in targets.iter() {
            unsafe {
                let col_type_ptr = pyo3::ffi::Py_TYPE(col.as_ptr());

                if col_type_ptr == crate::typeref::COLUMN_TYPE {
                    let col = col.cast_into_unchecked::<crate::column::PyColumn>();
                    tgs.push(col.get().inner.lock().name.clone());
                } else if pyo3::ffi::PyUnicode_CheckExact(col.as_ptr()) == 1 {
                    tgs.push(col.extract::<String>().unwrap_unchecked());
                } else {
                    return Err(typeerror!(
                        "expected str or Column, got {:?}",
                        col.py(),
                        col.as_ptr()
                    ));
                }
            }
        }

        let inner = OnConflictInner {
            targets: tgs,
            action: Default::default(),
            target_where: None,
            action_where: None,
        };

        Ok(Self {
            inner: parking_lot::Mutex::new(inner),
        })
    }

    #[pyo3(signature=(*keys))]
    fn do_nothing<'a>(
        slf: pyo3::PyRef<'a, Self>,
        keys: &pyo3::Bound<'a, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if keys.is_empty() {
            let mut lock = slf.inner.lock();
            lock.action = OnConflictAction::DoNothing(Default::default());
            drop(lock);

            return Ok(slf);
        }

        let mut pks: Vec<String> = Vec::with_capacity(keys.len());

        for key in keys.iter() {
            unsafe {
                if pyo3::ffi::Py_TYPE(key.as_ptr()) == crate::typeref::COLUMN_TYPE {
                    let col = key.cast_into_unchecked::<crate::column::PyColumn>();
                    pks.push(col.get().inner.lock().name.clone());
                } else if pyo3::ffi::PyUnicode_CheckExact(key.as_ptr()) == 1 {
                    pks.push(key.extract::<String>().unwrap_unchecked());
                } else {
                    return Err(typeerror!(
                        "expected str or Column, got {:?}",
                        key.py(),
                        key.as_ptr()
                    ));
                }
            }
        }

        {
            let mut lock = slf.inner.lock();
            lock.action = OnConflictAction::DoNothing(pks);
        }

        Ok(slf)
    }

    #[pyo3(signature=(*args, **kwds))]
    fn do_update<'a>(
        slf: pyo3::PyRef<'a, Self>,
        args: &'a pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&'a pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if !PyTupleMethods::is_empty(args) && kwds.is_some() {
            return Err(typeerror!("cannot use both args and kwargs at the same time",));
        }

        if !PyTupleMethods::is_empty(args) {
            Self::update_from_tuple(slf, args)
        } else if kwds.is_some() {
            Self::update_from_dictionary(slf, kwds.unwrap())
        } else {
            Err(typeerror!("no arguments provided",))
        }
    }

    fn target_where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return Err(typeerror!(
                    "expected Expr, got {:?}",
                    condition.py(),
                    condition.as_ptr()
                ));
            }

            let mut lock = slf.inner.lock();
            lock.target_where = Some(
                condition
                    .clone()
                    .cast_into_unchecked::<crate::expression::PyExpr>()
                    .unbind()
                    .into_any(),
            );
        }

        Ok(slf)
    }

    fn action_where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: &pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        unsafe {
            if pyo3::ffi::Py_TYPE(condition.as_ptr()) != crate::typeref::EXPR_TYPE {
                return Err(typeerror!(
                    "expected Expr, got {:?}",
                    condition.py(),
                    condition.as_ptr()
                ));
            }

            let mut lock = slf.inner.lock();
            lock.action_where = Some(
                condition
                    .clone()
                    .cast_into_unchecked::<crate::expression::PyExpr>()
                    .unbind()
                    .into_any(),
            );
        }

        Ok(slf)
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let lock = self.inner.lock();
        let mut s = Vec::<u8>::with_capacity(30);

        write!(s, "<OnConflict targets=[").unwrap();

        let n = lock.targets.len();
        for (index, ix) in lock.targets.iter().enumerate() {
            if index + 1 == n {
                write!(s, "{ix}").unwrap();
            } else {
                write!(s, "{ix}, ").unwrap();
            }
        }
        write!(s, "]").unwrap();

        match &lock.action {
            OnConflictAction::DoNothing(_) => {
                write!(s, " (DO NOTHING)").unwrap();
            }
            OnConflictAction::DoUpdate(_) => {
                write!(s, " (DO UPDATE)").unwrap();
            }
            OnConflictAction::None => (),
        }

        if let Some(x) = &lock.target_where {
            write!(s, " target_where={x}").unwrap();
        }
        if let Some(x) = &lock.action_where {
            write!(s, " action_where={x}").unwrap();
        }

        write!(s, ">").unwrap();

        unsafe { String::from_utf8_unchecked(s) }
    }
}
