pub struct OrderClause {
    // Always is `PyExpr`
    pub target: pyo3::Py<pyo3::PyAny>,
    pub order: sea_query::Order,
    pub null_order: Option<sea_query::NullOrdering>,
}

impl OrderClause {
    #[inline]
    pub fn from_parameters(
        target: pyo3::Bound<'_, pyo3::PyAny>,
        mut order: String,
        null_order: Option<String>,
    ) -> pyo3::PyResult<Self> {
        let order = {
            order.make_ascii_lowercase();

            if order == "asc" {
                sea_query::Order::Asc
            } else if order == "desc" {
                sea_query::Order::Desc
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "invalid order value, expected 'asc' or 'desc'; got {order:?}"
                )));
            }
        };

        let null_order = unsafe {
            if let Some(mut n) = null_order {
                n.make_ascii_lowercase();

                if n == "first" {
                    Some(sea_query::NullOrdering::First)
                } else if n == "last" {
                    Some(sea_query::NullOrdering::Last)
                } else {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "invalid order value, expected 'first', 'last' or None; got {n:?}"
                    )));
                }
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
}

impl std::fmt::Display for OrderClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.order {
            sea_query::Order::Asc => {
                write!(f, "{} ASC", self.target)?;
            }
            sea_query::Order::Desc => {
                write!(f, "{} DESC", self.target)?;
            }
            _ => unreachable!(),
        }

        if let Some(null_order) = self.null_order {
            match null_order {
                sea_query::NullOrdering::First => {
                    write!(f, "NULLS FIRST")?;
                }
                sea_query::NullOrdering::Last => {
                    write!(f, "NULLS LAST")?;
                }
            }
        }

        Ok(())
    }
}
