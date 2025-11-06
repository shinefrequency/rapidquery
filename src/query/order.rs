#[pyo3::pyclass(module = "rapidquery._lib", name = "Order", frozen)]
pub struct PyOrder {
    // Always is `PyExpr`
    pub target: pyo3::Py<pyo3::PyAny>,
    pub order: sea_query::Order,
    pub null_order: Option<sea_query::NullOrdering>,
}

#[pyo3::pymethods]
impl PyOrder {
    #[new]
    #[pyo3(signature=(target, order, null_order=None))]
    fn new(target: pyo3::Bound<'_, pyo3::PyAny>, order: u8, null_order: Option<u8>) -> pyo3::PyResult<Self> {
        let order = match order {
            0 => sea_query::Order::Asc,
            1 => sea_query::Order::Desc,
            _ => {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "invalid order value, use ORDER_ASC/ORDER_DESC. got {order}"
                )));
            }
        };

        let null_order = unsafe {
            if let Some(n) = null_order {
                if n > 1 {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "invalid order value, use NULL_ORDER_FIRST/NULL_ORDER_LAST. got {n}"
                    )));
                }

                Some(std::mem::transmute::<u8, sea_query::NullOrdering>(n))
            } else {
                None
            }
        };

        let target = crate::expression::PyExpr::from_bound_into_any(target)?;

        Ok(Self {
            target,
            order,
            null_order,
        })
    }

    #[getter]
    fn target(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        self.target.clone_ref(py)
    }

    #[getter]
    fn order(&self) -> u8 {
        match &self.order {
            sea_query::Order::Asc => 0,
            sea_query::Order::Desc => 1,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    #[getter]
    fn null_order(&self) -> Option<u8> {
        self.null_order.map(|x| x as u8)
    }

    fn __repr__(&self) -> String {
        use std::io::Write;

        let mut s = Vec::<u8>::with_capacity(30);

        write!(s, "<Order target={} order={:?}", self.target, self.order).unwrap();

        if let Some(x) = self.null_order {
            write!(s, " null_order={x:?}").unwrap();
        }

        write!(s, ">").unwrap();
        unsafe { String::from_utf8_unchecked(s) }
    }
}
