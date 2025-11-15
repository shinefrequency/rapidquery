use crate::backend::PyQueryStatement;
use pyo3::types::PyTupleMethods;
use sea_query::IntoIden;

#[pyo3::pyclass(module = "rapidquery._lib", name = "SelectExpr", frozen)]
pub struct PySelectExpr {
    // Always is `PyExpr`
    pub expr: pyo3::Py<pyo3::PyAny>,

    // Always is `PyExpr`
    pub alias: Option<String>,
    // TODO
    // pub window: pyo3::Py<pyo3::PyAny>,
}

impl PySelectExpr {
    pub fn clone_ref(&self, py: pyo3::Python) -> Self {
        Self {
            expr: self.expr.clone_ref(py),
            alias: self.alias.clone(),
        }
    }

    pub fn as_select_expr(&self, py: pyo3::Python) -> sea_query::SelectExpr {
        let expr = unsafe { self.expr.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
        let expr = expr.get().inner.clone();

        sea_query::SelectExpr {
            expr,
            alias: self.alias.as_ref().map(|x| sea_query::Alias::new(x).into_iden()),
            window: None,
        }
    }

    #[inline]
    #[optimize(speed)]
    pub fn from_bound_into_any(
        bound: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        use pyo3::PyTypeInfo;

        unsafe {
            if pyo3::ffi::Py_TYPE(bound.as_ptr()) == crate::typeref::EXPR_TYPE {
                let slf = Self {
                    expr: bound.clone().unbind(),
                    alias: None,
                };

                return pyo3::Py::new(bound.py(), slf).map(|x| x.into_any());
            }

            if PySelectExpr::is_exact_type_of(bound) {
                return Ok(bound.clone().unbind());
            }

            let expr = crate::expression::PyExpr::from_bound_into_any(bound.clone())?;
            let slf = Self { expr, alias: None };

            pyo3::Py::new(bound.py(), slf).map(|x| x.into_any())
        }
    }
}

#[pyo3::pymethods]
impl PySelectExpr {
    #[new]
    #[pyo3(signature=(expr, alias=None))]
    fn new(expr: &pyo3::Bound<'_, pyo3::PyAny>, alias: Option<String>) -> pyo3::PyResult<pyo3::Py<Self>> {
        use pyo3::PyTypeInfo;

        if PySelectExpr::is_exact_type_of(expr) {
            let slf = unsafe { expr.clone().cast_into_unchecked::<Self>() };

            if let Some(x) = alias {
                let expr = slf.get().expr.clone_ref(slf.py());

                let new_slf = Self { expr, alias: Some(x) };
                Ok(pyo3::Py::new(slf.py(), new_slf).unwrap())
            } else {
                Ok(slf.unbind())
            }
        } else {
            let py = expr.py();
            let expr = crate::expression::PyExpr::from_bound_into_any(expr.clone())?;
            let slf = Self { expr, alias };

            Ok(pyo3::Py::new(py, slf).unwrap())
        }
    }
}

#[derive(Debug, Default)]
pub enum SelectDistinct {
    #[default]
    None,
    Distinct,
    DistinctOn(
        // Always is `Vec<ColumnRef | String>`
        Vec<pyo3::Py<pyo3::PyAny>>,
    ),
}

pub struct SelectLock {
    pub r#type: sea_query::LockType,
    pub behavior: Option<sea_query::LockBehavior>,

    // Always is `Vec<TableName>`
    pub tables: Vec<pyo3::Py<pyo3::PyAny>>,
}

pub struct SelectJoin {
    pub r#type: sea_query::JoinType,

    // Always is `TableName | PySelect`
    pub table: pyo3::Py<pyo3::PyAny>,

    // Always is `PyExpr`
    pub on: pyo3::Py<pyo3::PyAny>,
    pub lateral: Option<String>,
}

pub enum SelectTable {
    SubQuery(
        // Always is `PySelect`
        pyo3::Py<pyo3::PyAny>,
        String,
    ),
    FunctionCall(
        // Always is `PyFunctionCall`
        pyo3::Py<pyo3::PyAny>,
        String,
    ),
    TableName(
        // Always is `PyTableName`
        pyo3::Py<pyo3::PyAny>,
    ),
}

#[derive(Default)]
pub struct SelectInner {
    // TODO: support from_values
    pub tables: Vec<SelectTable>,

    // TODO: support subqueries
    // Always is `Option<SelectExpr>`
    pub cols: Vec<pyo3::Py<pyo3::PyAny>>,

    // Always is `Option<PyExpr>`
    pub r#where: Option<pyo3::Py<pyo3::PyAny>>,

    // Always is `Vec<PyExpr>`
    pub groups: Vec<pyo3::Py<pyo3::PyAny>>,

    // Always is `Vec<_, PySelect>`
    pub unions: Vec<(sea_query::UnionType, pyo3::Py<pyo3::PyAny>)>,

    // Always is `Option<PyExpr>`
    pub having: Option<pyo3::Py<pyo3::PyAny>>,

    pub orders: Vec<super::order::OrderClause>,

    pub distinct: SelectDistinct,
    pub join: Vec<SelectJoin>,
    pub lock: Option<SelectLock>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    // TODO
    // pub window: Option<pyo3::Py<pyo3::PyAny>>,
    // pub with: Option<pyo3::Py<pyo3::PyAny>>,
    // pub table_sample: Option<pyo3::Py<pyo3::PyAny>>,
    // pub index_hint: Option<pyo3::Py<pyo3::PyAny>>,
}

impl SelectInner {
    #[inline]
    pub fn as_statement(&self, py: pyo3::Python) -> sea_query::SelectStatement {
        let mut stmt = sea_query::SelectStatement::new();

        match &self.distinct {
            SelectDistinct::None => (),
            SelectDistinct::Distinct => {
                stmt.distinct();
            }
            SelectDistinct::DistinctOn(cols) => {
                use sea_query::IntoColumnRef;

                stmt.distinct_on(cols.iter().map(|col| unsafe {
                    if pyo3::ffi::PyUnicode_Check(col.as_ptr()) == 1 {
                        let x = sea_query::Alias::new(col.extract::<String>(py).unwrap_unchecked());
                        x.into_column_ref()
                    } else {
                        let x = col.cast_bound_unchecked::<crate::common::PyColumnRef>(py).get();
                        x.clone().into_column_ref()
                    }
                }));
            }
        }

        for table in self.tables.iter() {
            match table {
                SelectTable::TableName(x) => unsafe {
                    let x = unsafe { x.cast_bound_unchecked::<crate::common::PyTableName>(py) };
                    stmt.from(x.get().clone());
                },
                SelectTable::FunctionCall(x, alias) => unsafe {
                    let x = unsafe { x.cast_bound_unchecked::<crate::expression::PyFunctionCall>(py) };
                    stmt.from_function(x.get().inner.lock().clone(), sea_query::Alias::new(alias));
                },
                SelectTable::SubQuery(x, alias) => unsafe {
                    let x = unsafe { x.cast_bound_unchecked::<PySelect>(py) };
                    let inner = x.get().inner.lock();

                    stmt.from_subquery(inner.as_statement(py), sea_query::Alias::new(alias));
                },
            }
        }

        if !self.cols.is_empty() {
            stmt.exprs(self.cols.iter().map(|x| unsafe {
                let expr = x.cast_bound_unchecked::<PySelectExpr>(py);
                expr.get().as_select_expr(py)
            }));
        }

        if !self.groups.is_empty() {
            stmt.add_group_by(self.groups.iter().map(|x| unsafe {
                let expr = x.cast_bound_unchecked::<crate::expression::PyExpr>(py);
                expr.get().inner.clone()
            }));
        }

        if let Some(x) = &self.r#where {
            let x = unsafe { x.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            stmt.and_where(x.get().inner.clone());
        }

        if let Some(x) = &self.having {
            let x = unsafe { x.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            stmt.and_having(x.get().inner.clone());
        }

        if let Some(n) = self.limit {
            stmt.limit(n);
        }

        if let Some(n) = self.offset {
            stmt.offset(n);
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

        if let Some(lock) = &self.lock {
            match (lock.behavior, lock.tables.is_empty()) {
                (Some(behavior), false) => {
                    stmt.lock_with_tables_behavior(
                        lock.r#type,
                        lock.tables.iter().map(|table| {
                            let x = unsafe { table.cast_bound_unchecked::<crate::common::PyTableName>(py) };
                            x.get().clone()
                        }),
                        behavior,
                    );
                }
                (Some(behavior), true) => {
                    stmt.lock_with_behavior(lock.r#type, behavior);
                }
                (None, false) => {
                    stmt.lock_with_tables(
                        lock.r#type,
                        lock.tables.iter().map(|table| {
                            let x = unsafe { table.cast_bound_unchecked::<crate::common::PyTableName>(py) };
                            x.get().clone()
                        }),
                    );
                }
                (None, true) => {
                    stmt.lock(lock.r#type);
                }
            }
        }

        stmt.unions(self.unions.iter().map(|(union_type, union_stmt)| {
            let union_stmt = unsafe { union_stmt.cast_bound_unchecked::<PySelect>(py) };

            let inner = union_stmt.get().inner.lock();
            (*union_type, inner.as_statement(py))
        }));

        for join in self.join.iter() {
            let condition = unsafe { join.on.cast_bound_unchecked::<crate::expression::PyExpr>(py) };
            let condition = condition.get().inner.clone();

            if let Some(lateral) = &join.lateral {
                let query = unsafe { join.table.cast_bound_unchecked::<PySelect>(py) };
                let query = query.get().inner.lock();

                stmt.join_lateral(
                    join.r#type,
                    query.as_statement(py),
                    sea_query::Alias::new(lateral),
                    condition,
                );
            } else {
                let table = unsafe { join.table.cast_bound_unchecked::<crate::common::PyTableName>(py) };
                stmt.join(join.r#type, table.get().clone(), condition);
            }
        }

        stmt
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "Select", frozen, extends=PyQueryStatement)]
pub struct PySelect {
    pub inner: parking_lot::Mutex<SelectInner>,
}

#[pyo3::pymethods]
impl PySelect {
    #[new]
    #[pyo3(signature=(*cols))]
    fn new(cols: &pyo3::Bound<'_, pyo3::types::PyTuple>) -> pyo3::PyResult<(Self, PyQueryStatement)> {
        let mut exprs = Vec::with_capacity(PyTupleMethods::len(cols));

        for expr in PyTupleMethods::iter(cols) {
            exprs.push(PySelectExpr::from_bound_into_any(&expr)?);
        }

        let slf = Self {
            inner: parking_lot::Mutex::new(SelectInner {
                cols: exprs,
                ..Default::default()
            }),
        };

        Ok((slf, PyQueryStatement))
    }

    #[pyo3(signature=(*on))]
    fn distinct<'a>(
        slf: pyo3::PyRef<'a, Self>,
        on: &'a pyo3::Bound<'a, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if PyTupleMethods::is_empty(on) {
            let mut lock = slf.inner.lock();
            lock.distinct = SelectDistinct::Distinct;
        } else {
            let mut cols = Vec::with_capacity(PyTupleMethods::len(on));

            for col in PyTupleMethods::iter(on) {
                unsafe {
                    let col_ptr = pyo3::ffi::Py_TYPE(col.as_ptr());

                    if col_ptr == crate::typeref::COLUMN_TYPE {
                        let col = col.cast_into_unchecked::<crate::column::PyColumn>();
                        let col_ref = col.get().inner.lock().as_column_ref(slf.py());
                        cols.push(
                            pyo3::Py::new(slf.py(), crate::common::PyColumnRef::from(col_ref))
                                .unwrap()
                                .into_any(),
                        );
                    } else if (col_ptr == crate::typeref::COLUMN_REF_TYPE)
                        || (pyo3::ffi::PyUnicode_Check(col.as_ptr()) == 1)
                    {
                        cols.push(col.unbind());
                    } else {
                        return Err(typeerror!(
                            "expected Column or ColumnRef or str, got {:?}",
                            col.py(),
                            col.as_ptr()
                        ));
                    }
                }
            }

            let mut lock = slf.inner.lock();
            lock.distinct = SelectDistinct::DistinctOn(cols);
        }

        Ok(slf)
    }

    #[pyo3(signature=(*cols))]
    fn columns<'a>(
        slf: pyo3::PyRef<'a, Self>,
        cols: &'a pyo3::Bound<'a, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut exprs = Vec::with_capacity(PyTupleMethods::len(cols));

        for expr in PyTupleMethods::iter(cols) {
            exprs.push(PySelectExpr::from_bound_into_any(&expr)?);
        }

        {
            let mut lock = slf.inner.lock();
            lock.cols = exprs;
        }

        Ok(slf)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_table<'a>(
        slf: pyo3::PyRef<'a, Self>,
        table: &'a pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let table = {
            if let Ok(x) = table.cast_exact::<crate::table::PyTable>() {
                let guard = x.get().inner.lock();
                guard.name.clone_ref(slf.py())
            } else if let Ok(x) = table.cast_exact::<crate::table::PyAliasedTable>() {
                x.get().name(slf.py())?
            } else {
                crate::common::PyTableName::from_pyobject(table)?
            }
        };

        {
            let mut lock = slf.inner.lock();
            lock.tables.push(SelectTable::TableName(table));
        }

        Ok(slf)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_subquery<'a>(
        slf: pyo3::PyRef<'a, Self>,
        subquery: &'a pyo3::Bound<'_, pyo3::PyAny>,
        alias: String,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if std::hint::unlikely(slf.as_ptr() == subquery.as_ptr()) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "A Select statement cannot select from itself",
            ));
        }

        let subquery = unsafe {
            if std::hint::likely(
                pyo3::ffi::Py_TYPE(subquery.as_ptr()) == crate::typeref::SELECT_STATEMENT_TYPE,
            ) {
                subquery.clone().unbind()
            } else {
                return Err(typeerror!(
                    "expected Select, got {:?}",
                    subquery.py(),
                    subquery.as_ptr()
                ));
            }
        };

        {
            let mut lock = slf.inner.lock();
            lock.tables.push(SelectTable::SubQuery(subquery, alias));
        }

        Ok(slf)
    }

    #[allow(clippy::wrong_self_convention)]
    fn from_function<'a>(
        slf: pyo3::PyRef<'a, Self>,
        function: &'a pyo3::Bound<'_, pyo3::PyAny>,
        alias: String,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let function = unsafe {
            if std::hint::likely(pyo3::ffi::Py_TYPE(function.as_ptr()) == crate::typeref::FUNCTION_CALL_TYPE)
            {
                function.clone().unbind()
            } else {
                return Err(typeerror!(
                    "expected FunctionCall, got {:?}",
                    function.py(),
                    function.as_ptr()
                ));
            }
        };

        {
            let mut lock = slf.inner.lock();
            lock.tables.push(SelectTable::FunctionCall(function, alias));
        }

        Ok(slf)
    }

    fn limit(slf: pyo3::PyRef<'_, Self>, n: u64) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();
            lock.limit = Some(n);
        }

        slf
    }

    fn offset(slf: pyo3::PyRef<'_, Self>, n: u64) -> pyo3::PyRef<'_, Self> {
        {
            let mut lock = slf.inner.lock();
            lock.offset = Some(n);
        }

        slf
    }

    fn r#where<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let condition = crate::expression::PyExpr::from_bound_into_any(condition)?;

        {
            let mut lock = slf.inner.lock();
            lock.r#where = Some(condition);
        }

        Ok(slf)
    }

    fn having<'a>(
        slf: pyo3::PyRef<'a, Self>,
        condition: pyo3::Bound<'a, pyo3::PyAny>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let condition = crate::expression::PyExpr::from_bound_into_any(condition)?;

        {
            let mut lock = slf.inner.lock();
            lock.having = Some(condition);
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

    #[pyo3(signature=(r#type=String::from("exclusive"), behavior=None, tables=Vec::new()))]
    fn lock(
        slf: pyo3::PyRef<'_, Self>,
        mut r#type: String,
        mut behavior: Option<String>,
        tables: Vec<pyo3::Py<pyo3::PyAny>>,
    ) -> pyo3::PyResult<pyo3::PyRef<'_, Self>> {
        let r#type = {
            r#type.make_ascii_lowercase();

            if r#type == "exclusive" {
                sea_query::LockType::Update
            } else if r#type == "shared" {
                sea_query::LockType::Share
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "acceptable lock type are: 'exclusive' and 'shared'. got invalid type",
                ));
            }
        };

        let behavior = match &mut behavior {
            None => None,
            Some(x) => {
                x.make_ascii_lowercase();

                if x == "nowait" {
                    Some(sea_query::LockBehavior::Nowait)
                } else if x == "skip" {
                    Some(sea_query::LockBehavior::SkipLocked)
                } else {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "acceptable lock behavior are: 'nowait' and 'skip'. got invalid behavior",
                    ));
                }
            }
        };

        let mut tbs = Vec::with_capacity(tables.len());

        for tb in tables.into_iter() {
            let bound = tb.bind(slf.py());

            if let Ok(x) = bound.cast_exact::<crate::table::PyTable>() {
                let guard = x.get().inner.lock();
                tbs.push(guard.name.clone_ref(slf.py()));
            } else {
                tbs.push(crate::common::PyTableName::from_pyobject(bound)?);
            }
        }

        {
            let mut lock = slf.inner.lock();
            lock.lock = Some(SelectLock {
                r#type,
                behavior,
                tables: tbs,
            });
        }

        Ok(slf)
    }

    #[pyo3(signature=(*cols))]
    fn group_by<'a>(
        slf: pyo3::PyRef<'a, Self>,
        cols: &'a pyo3::Bound<'a, pyo3::types::PyTuple>,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let mut exprs = Vec::with_capacity(PyTupleMethods::len(cols));

        for expr in PyTupleMethods::iter(cols) {
            exprs.push(crate::expression::PyExpr::from_bound_into_any(expr)?);
        }

        {
            let mut lock = slf.inner.lock();
            lock.groups = exprs;
        }

        Ok(slf)
    }

    #[pyo3(signature=(statement, r#type=String::from("distinct")))]
    fn union<'a>(
        slf: pyo3::PyRef<'a, Self>,
        statement: &'a pyo3::Bound<'a, pyo3::PyAny>,
        mut r#type: String,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        if std::hint::unlikely(slf.as_ptr() == statement.as_ptr()) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "A Select statement cannot union itself",
            ));
        }

        unsafe {
            if pyo3::ffi::Py_TYPE(statement.as_ptr()) != crate::typeref::SELECT_STATEMENT_TYPE {
                return Err(typeerror!(
                    "expected Select, got {:?}",
                    statement.py(),
                    statement.as_ptr()
                ));
            }
        }

        let r#type = {
            r#type.make_ascii_lowercase();

            if r#type == "all" {
                sea_query::UnionType::All
            } else if r#type == "intersect" {
                sea_query::UnionType::Intersect
            } else if r#type == "distinct" {
                sea_query::UnionType::Distinct
            } else if r#type == "except" {
                sea_query::UnionType::Except
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "acceptable union types are: 'all', 'distinct', 'except', and 'intersect'. got invalid type",
                ));
            }
        };

        {
            let mut lock = slf.inner.lock();
            lock.unions.push((r#type, statement.clone().unbind()));
        }

        Ok(slf)
    }

    #[pyo3(signature=(table, on, r#type=String::new()))]
    fn join<'a>(
        slf: pyo3::PyRef<'a, Self>,
        table: &'a pyo3::Bound<'a, pyo3::PyAny>,
        on: &'a pyo3::Bound<'a, pyo3::PyAny>,
        mut r#type: String,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let r#type = {
            r#type.make_ascii_lowercase();

            if r#type.is_empty() {
                sea_query::JoinType::Join
            } else if r#type == "cross" {
                sea_query::JoinType::CrossJoin
            } else if r#type == "full" {
                sea_query::JoinType::FullOuterJoin
            } else if r#type == "inner" {
                sea_query::JoinType::InnerJoin
            } else if r#type == "left" {
                sea_query::JoinType::LeftJoin
            } else if r#type == "right" {
                sea_query::JoinType::RightJoin
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "acceptable join types are: '', 'cross', 'full', 'left', 'right', and 'inner'. got invalid type",
                ));
            }
        };

        let table = {
            if let Ok(x) = table.cast_exact::<crate::table::PyTable>() {
                let guard = x.get().inner.lock();
                guard.name.clone_ref(slf.py())
            } else if let Ok(x) = table.cast_exact::<crate::table::PyAliasedTable>() {
                x.get().name(slf.py())?
            } else {
                crate::common::PyTableName::from_pyobject(table)?
            }
        };

        let expr = crate::expression::PyExpr::from_bound_into_any(on.clone())?;

        let join_expr = SelectJoin {
            r#type,
            table,
            on: expr,
            lateral: None,
        };

        {
            let mut lock = slf.inner.lock();
            lock.join.push(join_expr);
        }

        Ok(slf)
    }

    #[pyo3(signature=(query, alias, on, r#type=String::new()))]
    fn join_lateral<'a>(
        slf: pyo3::PyRef<'a, Self>,
        query: &'a pyo3::Bound<'a, pyo3::PyAny>,
        alias: String,
        on: &'a pyo3::Bound<'a, pyo3::PyAny>,
        mut r#type: String,
    ) -> pyo3::PyResult<pyo3::PyRef<'a, Self>> {
        let r#type = {
            r#type.make_ascii_lowercase();

            if r#type.is_empty() {
                sea_query::JoinType::Join
            } else if r#type == "cross" {
                sea_query::JoinType::CrossJoin
            } else if r#type == "full" {
                sea_query::JoinType::FullOuterJoin
            } else if r#type == "inner" {
                sea_query::JoinType::InnerJoin
            } else if r#type == "left" {
                sea_query::JoinType::LeftJoin
            } else if r#type == "right" {
                sea_query::JoinType::RightJoin
            } else {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "acceptable join types are: '', 'cross', 'full', 'left', 'right', and 'inner'. got invalid type",
                ));
            }
        };

        unsafe {
            if pyo3::ffi::Py_TYPE(query.as_ptr()) != crate::typeref::SELECT_STATEMENT_TYPE {
                return Err(typeerror!(
                    "expected Select, got {:?}",
                    query.py(),
                    query.as_ptr()
                ));
            }
        }

        let expr = crate::expression::PyExpr::from_bound_into_any(on.clone())?;

        let join_expr = SelectJoin {
            r#type,
            table: query.clone().unbind(),
            on: expr,
            lateral: Some(alias),
        };

        {
            let mut lock = slf.inner.lock();
            lock.join.push(join_expr);
        }

        Ok(slf)
    }

    fn build(
        &self,
        backend: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<(String, pyo3::Py<pyo3::PyAny>)> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_query_parts!(backend => build_collect_any_into(stmt))
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let lock = self.inner.lock();
        let stmt = lock.as_statement(backend.py());
        drop(lock);

        build_query_string!(backend => build_collect_any_into(stmt))
    }
}
