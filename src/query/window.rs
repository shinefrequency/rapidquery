use pyo3::types::PyTupleMethods;
use sea_query::OverStatement;

#[derive(Debug, Clone, PartialEq)]
pub struct FrameClause {
    pub r#type: sea_query::FrameType,
    pub start: sea_query::Frame,
    pub end: Option<sea_query::Frame>,
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "WindowFrame", frozen)]
pub struct PyWindowFrame(sea_query::Frame);

#[pyo3::pymethods]
impl PyWindowFrame {
    #[classmethod]
    fn unbounded_preceding(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::UnboundedPreceding)
    }

    #[classmethod]
    fn current_row(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::CurrentRow)
    }

    #[classmethod]
    fn unbounded_following(_cls: &pyo3::Bound<'_, pyo3::types::PyType>) -> Self {
        Self(sea_query::Frame::UnboundedFollowing)
    }

    #[classmethod]
    fn following(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, val: u32) -> Self {
        Self(sea_query::Frame::Following(val))
    }

    #[classmethod]
    fn preceding(_cls: &pyo3::Bound<'_, pyo3::types::PyType>, val: u32) -> Self {
        Self(sea_query::Frame::Preceding(val))
    }
}

#[derive(Default)]
pub struct WindowInner {
    // Always is `Vec<PyExpr>`
    pub partition_by: Vec<pyo3::Py<pyo3::PyAny>>,
    pub orders: Vec<super::order::OrderClause>,
    pub frame: Option<FrameClause>,
}

impl WindowInner {
    #[inline]
    pub fn as_statement(&self, py: pyo3::Python) -> sea_query::WindowStatement {
        let mut stmt = sea_query::WindowStatement::new();

        for expr in &self.partition_by {
            let expr = unsafe { expr.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            stmt.add_partition_by(expr.get().inner.clone());
        }

        for order in self.orders.iter() {
            let target = unsafe { order.target.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            let target = target.get().inner.clone();

            if let Some(x) = order.null_order {
                stmt.order_by_expr_with_nulls(target, order.order.clone(), x);
            } else {
                stmt.order_by_expr(target, order.order.clone());
            }
        }

        if let Some(x) = &self.frame {
            stmt.frame(x.r#type.clone(), x.start.clone(), x.end.clone());
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "Window", frozen)]
pub struct PyWindow {
    pub inner: parking_lot::Mutex<WindowInner>,
}

#[pyo3::pymethods]
impl PyWindow {
    #[new]
    #[pyo3(signature=(*partition_by))]
    fn new(partition_by: &pyo3::Bound<'_, pyo3::types::PyTuple>) -> pyo3::PyResult<Self> {
        let mut partitions = Vec::<pyo3::Py<pyo3::PyAny>>::new();

        unsafe {
            for value in PyTupleMethods::iter(partition_by) {
                partitions.push(crate::expression::PyExpr::from_bound_into_any(value)?);
            }
        }

        Ok(Self {
            inner: parking_lot::Mutex::new(WindowInner {
                partition_by: partitions,
                orders: Vec::new(),
                frame: None,
            }),
        })
    }

    #[pyo3(signature=(*partition_by))]
    fn partition<'a>(
        slf: pyo3::PyRef<'a, Self>,
        partition_by: &pyo3::Bound<'a, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut partitions = Vec::<pyo3::Py<pyo3::PyAny>>::new();

        unsafe {
            for value in PyTupleMethods::iter(partition_by) {
                partitions.push(crate::expression::PyExpr::from_bound_into_any(value)?);
            }
        }

        {
            let mut lock = slf.inner.lock();
            lock.partition_by.append(&mut partitions);
        }

        Ok(slf)
    }

    #[pyo3(signature=(target, order, null_order=None))]
    fn order_by<'a>(
        slf: pyo3::PyRef<'a, Self>,
        target: pyo3::Bound<'_, pyo3::PyAny>,
        order: String,
        null_order: Option<String>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let order = super::order::OrderClause::from_parameters(target, order, null_order)?;

        {
            let mut lock = slf.inner.lock();
            lock.orders.push(order);
        }

        Ok(slf)
    }

    #[pyo3(signature=(r#type, start, end=None))]
    fn frame<'a>(
        slf: pyo3::PyRef<'a, Self>,
        mut r#type: String,
        start: pyo3::Bound<'_, pyo3::PyAny>,
        end: Option<pyo3::Bound<'_, pyo3::PyAny>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let r#type = {
            r#type.make_ascii_lowercase();

            if r#type == "rows" {
                sea_query::FrameType::Rows
            } else if r#type == "range" {
                sea_query::FrameType::Range
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "invalid frame type, expected 'rows' or 'range'; got {:?}",
                    r#type
                )));
            }
        };

        let start = {
            match start.cast::<PyWindowFrame>() {
                Ok(x) => x.get().0.clone(),
                Err(_) => {
                    return Err(typeerror!(
                        "expected WindowFrame, got {}",
                        slf.py(),
                        start.as_ptr()
                    ));
                }
            }
        };

        let end = {
            match end {
                None => None,
                Some(y) => match y.cast::<PyWindowFrame>() {
                    Ok(x) => Some(x.get().0.clone()),
                    Err(_) => {
                        return Err(typeerror!("expected WindowFrame, got {}", slf.py(), y.as_ptr()));
                    }
                },
            }
        };

        {
            let mut lock = slf.inner.lock();
            lock.frame = Some(FrameClause { r#type, start, end });
        }

        Ok(slf)
    }
}
