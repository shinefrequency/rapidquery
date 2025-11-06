use pyo3::types::PyAnyMethods;

/// A trait for conversation between ColumnType classes into
/// [`sea_query::ColumnType`]
pub trait AsColumnType {
    fn as_column_type<'a>(&'a self, py: pyo3::Python<'a>) -> sea_query::ColumnType;
}

/// Metaclass for all SQL column data types.
///
/// This abstract base class represents SQL data types that can be used in
/// column definitions. Each subclass implements a specific SQL data type
/// with its particular characteristics, constraints, and backend-specific
/// representations.
///
/// Don't try to use this class directly; **Use it only for type-hints.**
/// Also don't use this class as subclass.
#[pyo3::pyclass(
    module = "rapidquery._lib",
    name = "ColumnTypeMeta",
    frozen,
    subclass,
    generic
)]
#[derive(Default, Clone, Debug)]
pub struct PyColumnTypeMeta {}

macro_rules! impl_column_type {
    (
        $(
            $(#[$docs:meta])*
            $name:ident(name=$pyname:literal) => simple($init:expr),
        )+
    ) => {
        $(
            $(#[$docs])*
            #[pyo3::pyclass(module = "rapidquery._lib", name = $pyname, frozen, extends=PyColumnTypeMeta)]
            #[derive(Debug, Clone, Default)]
            pub struct $name {}

            #[pyo3::pymethods]
            impl $name {
                #[new]
                fn new() -> (Self, PyColumnTypeMeta) {
                    (Self::default(), PyColumnTypeMeta::default())
                }

                fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    if !other.bind(slf.py()).is_exact_instance_of::<Self>() {
                        Err(
                            typeerror!(
                                "'==' not supported between instances of {} and {}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr()
                            )
                        )
                    } else {
                        Ok(true)
                    }
                }

                fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    if !other.bind(slf.py()).is_exact_instance_of::<Self>() {
                        Err(
                            typeerror!(
                                "'!=' not supported between instances of {} and {}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr()
                            )
                        )
                    } else {
                        Ok(false)
                    }
                }

                fn __repr__(&self) -> String {
                    format!("<{} >", $pyname)
                }
            }

            impl AsColumnType for $name {
                #[inline]
                fn as_column_type<'a>(&'a self, _py: pyo3::Python<'a>) -> sea_query::ColumnType {
                    $init
                }
            }
        )+
    };

    (
        $(
            $(#[$docs:meta])*
            $name:ident(name=$pyname:literal) => length(|$length_param:ident| $init:expr),
        )+
    ) => {
        $(
            $(#[$docs])*
            #[pyo3::pyclass(module = "rapidquery._lib", name = $pyname, frozen, extends=PyColumnTypeMeta)]
            #[derive(Debug)]
            pub struct $name {
                pub(crate) length: std::sync::atomic::AtomicU32,
            }

            #[pyo3::pymethods]
            impl $name {
                #[new]
                #[pyo3(signature=(length=None))]
                fn new(length: Option<u32>) -> (Self, PyColumnTypeMeta) {
                    let value = length.unwrap_or(0);
                    (
                        Self {
                            length: std::sync::atomic::AtomicU32::new(value),
                        },
                        PyColumnTypeMeta::default()
                    )
                }

                #[getter]
                fn length(&self) -> Option<u32> {
                    let length = self.length.load(std::sync::atomic::Ordering::Relaxed);
                    if length == 0 { None } else { Some(length) }
                }

                #[setter]
                fn set_length(&self, val: Option<u32>) {
                    let new_val = val.unwrap_or(0);
                    self.length.store(new_val, std::sync::atomic::Ordering::Relaxed);
                }

                fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    if slf.as_ptr() == other.as_ptr() {
                        return Ok(true);
                    }

                    let other = other
                        .extract::<pyo3::PyRef<'_, Self>>(slf.py())
                        .map_err(
                            |_| typeerror!(
                                "'==' not supported between instances of {} and {}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr()
                            )
                        )?;

                    Ok(slf.length.load(std::sync::atomic::Ordering::Relaxed) == other.length.load(std::sync::atomic::Ordering::Relaxed))
                }

                fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    if slf.as_ptr() == other.as_ptr() {
                        return Ok(false);
                    }

                    let other = other
                        .extract::<pyo3::PyRef<'_, Self>>(slf.py())
                        .map_err(
                            |_| typeerror!(
                                "'!=' not supported between instances of {} and {}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr()
                            )
                        )?;

                    Ok(slf.length.load(std::sync::atomic::Ordering::Relaxed) != other.length.load(std::sync::atomic::Ordering::Relaxed))
                }

                fn __repr__(&self) -> String {
                    match self.length() {
                        Some(x) => format!("<{} length={:?}>", $pyname, x),
                        None => format!("<{} length=None>", $pyname),
                    }
                }
            }

            impl AsColumnType for $name {
                #[inline]
                fn as_column_type<'a>(&'a self, _py: pyo3::Python<'a>) -> sea_query::ColumnType {
                    let $length_param = self.length();
                    $init
                }
            }
        )+
    };

    (
        $(
            $(#[$docs:meta])*
            $name:ident(name=$pyname:literal) => precision_scale(|$p_param:ident, $s_param:ident| $init:expr),
        )+
    ) => {
        $(
            $(#[$docs])*
            #[pyo3::pyclass(module = "rapidquery._lib", name = $pyname, frozen, extends=PyColumnTypeMeta)]
            #[derive(Debug)]
            pub struct $name {
                precision: std::sync::atomic::AtomicU32,
                scale: std::sync::atomic::AtomicU32,
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        precision: std::sync::atomic::AtomicU32::new(0),
                        scale: std::sync::atomic::AtomicU32::new(0),
                    }
                }
            }

            #[pyo3::pymethods]
            impl $name {
                #[new]
                #[pyo3(signature=(precision_scale=None))]
                fn new(precision_scale: Option<(u32, u32)>) -> (Self, PyColumnTypeMeta) {
                    let (precision, scale) = precision_scale.unwrap_or((0, 0));
                    (
                        Self {
                            precision: std::sync::atomic::AtomicU32::new(precision),
                            scale: std::sync::atomic::AtomicU32::new(scale),
                        },
                        PyColumnTypeMeta::default()
                    )
                }

                #[getter]
                fn precision_scale(&self) -> Option<(u32, u32)> {
                    let precision = self.precision.load(std::sync::atomic::Ordering::Relaxed);
                    let scale = self.scale.load(std::sync::atomic::Ordering::Relaxed);

                    if precision == 0 || scale == 0 {
                        None
                    } else {
                        Some((precision, scale))
                    }
                }

                #[setter]
                fn set_precision_scale(&self, val: Option<(u32, u32)>) {
                    let (precision, scale) = val.unwrap_or((0, 0));
                    self.precision.store(precision, std::sync::atomic::Ordering::Relaxed);
                    self.scale.store(scale, std::sync::atomic::Ordering::Relaxed);
                }

                fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    if slf.as_ptr() == other.as_ptr() {
                        return Ok(true);
                    }

                    let other = other
                        .extract::<pyo3::PyRef<'_, Self>>(slf.py())
                        .map_err(
                            |_| typeerror!(
                                "'==' not supported between instances of {} and {}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr()
                            )
                        )?;

                    Ok(slf.precision_scale() == other.precision_scale())
                }

                fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    if slf.as_ptr() == other.as_ptr() {
                        return Ok(false);
                    }

                    let other = other
                        .extract::<pyo3::PyRef<'_, Self>>(slf.py())
                        .map_err(
                            |_| typeerror!(
                                "'!=' not supported between instances of {} and {}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr()
                            )
                        )?;

                    Ok(slf.precision_scale() != other.precision_scale())
                }

                fn __repr__(&self) -> String {
                    match self.precision_scale() {
                        Some(x) => format!("<{} precision_scale={:?}>", $pyname, x),
                        None => format!("<{} precision_scale=None>", $pyname),
                    }
                }
            }

            impl AsColumnType for $name {
                #[inline]
                fn as_column_type<'a>(&'a self, _py: pyo3::Python<'a>) -> sea_query::ColumnType {
                    let ($p_param, $s_param) = match self.precision_scale() {
                        Some((p, s)) => (Some(p), Some(s)),
                        None => (None, None),
                    };
                    $init
                }
            }
        )+
    };
}

impl_column_type!(
    PyTinyIntegerType(name="TinyIntegerType") => simple(sea_query::ColumnType::TinyInteger),
    PySmallIntegerType(name="SmallIntegerType") => simple(sea_query::ColumnType::SmallInteger),
    PyIntegerType(name="IntegerType") => simple(sea_query::ColumnType::Integer),
    PyBigIntegerType(name="BigIntegerType") => simple(sea_query::ColumnType::BigInteger),
    PyTinyUnsignedType(name="TinyUnsignedType") => simple(sea_query::ColumnType::TinyUnsigned),
    PySmallUnsignedType(name="SmallUnsignedType") => simple(sea_query::ColumnType::SmallUnsigned),
    PyUnsignedType(name="UnsignedType") => simple(sea_query::ColumnType::Unsigned),
    PyBigUnsignedType(name="BigUnsignedType") => simple(sea_query::ColumnType::BigUnsigned),
    PyTextType(name="TextType") => simple(sea_query::ColumnType::Text),
    PyFloatType(name="FloatType") => simple(sea_query::ColumnType::Float),
    PyDoubleType(name="DoubleType") => simple(sea_query::ColumnType::Double),
    PyDateTimeType(name="DateTimeType") => simple(sea_query::ColumnType::DateTime),
    PyTimestampType(name="TimestampType") => simple(sea_query::ColumnType::Timestamp),
    PyTimestampWithTimeZoneType(name="TimestampWithTimeZoneType") => simple(sea_query::ColumnType::TimestampWithTimeZone),
    PyTimeType(name="TimeType") => simple(sea_query::ColumnType::Time),
    PyDateType(name="DateType") => simple(sea_query::ColumnType::Date),
    PyYearType(name="YearType") => simple(sea_query::ColumnType::Year),
    PyBlobType(name="BlobType") => simple(sea_query::ColumnType::Blob),
    PyBooleanType(name="BooleanType") => simple(sea_query::ColumnType::Boolean),
    PyJsonType(name="JsonType") => simple(sea_query::ColumnType::Json),
    PyJsonBinaryType(name="JsonBinaryType") => simple(sea_query::ColumnType::JsonBinary),
    PyUuidType(name="UuidType") => simple(sea_query::ColumnType::Uuid),
    PyCidrType(name="CidrType") => simple(sea_query::ColumnType::Cidr),
    PyInetType(name="InetType") => simple(sea_query::ColumnType::Inet),
    PyMacAddressType(name="MacAddressType") => simple(sea_query::ColumnType::MacAddr),
    PyLTreeType(name="LTreeType") => simple(sea_query::ColumnType::LTree),
);

impl_column_type!(
    PyCharType(name="CharType") => length(|length| sea_query::ColumnType::Char(length)),
    PyStringType(name="StringType") => length(|length| {
        sea_query::ColumnType::String(
            length.map(sea_query::StringLen::N).unwrap_or(sea_query::StringLen::None)
        )
    }),
    PyBinaryType(name="BinaryType") => length(|length| {
        sea_query::ColumnType::Binary(length.unwrap_or(1))
    }),
    PyVarBinaryType(name="VarBinaryType") => length(|length| {
        sea_query::ColumnType::VarBinary(
            length.map(sea_query::StringLen::N).unwrap_or(sea_query::StringLen::None)
        )
    }),
    PyBitType(name="BitType") => length(|length| sea_query::ColumnType::Bit(length)),
    PyVarBitType(name="VarBitType") => length(|length| {
        sea_query::ColumnType::VarBit(length.unwrap_or(1))
    }),
    PyVectorType(name="VectorType") => length(|length| sea_query::ColumnType::Vector(length)),
);

impl_column_type!(
    PyDecimalType(name="DecimalType") => precision_scale(|precision, scale| {
        sea_query::ColumnType::Decimal(
            precision.and_then(|p| scale.map(|s| (p, s)))
        )
    }),
    PyMoneyType(name="MoneyType") => precision_scale(|precision, scale| {
        sea_query::ColumnType::Money(
            precision.and_then(|p| scale.map(|s| (p, s)))
        )
    }),
);

fn into_pginterval(constant: u8) -> Result<sea_query::PgInterval, ()> {
    if std::hint::unlikely(constant > 12) {
        Err(())
    } else {
        Ok(unsafe { std::mem::transmute::<u8, sea_query::PgInterval>(constant) })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct IntervalTypeFields {
    pub(super) fields: Option<sea_query::PgInterval>,
    pub(super) precision: Option<u32>,
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "IntervalType", frozen, extends=PyColumnTypeMeta)]
pub struct PyIntervalType {
    pub(super) inner: parking_lot::Mutex<IntervalTypeFields>,
}

#[pyo3::pymethods]
impl PyIntervalType {
    #[new]
    #[pyo3(signature=(fields=None, precision=None))]
    fn new(fields: Option<u8>, precision: Option<u32>) -> pyo3::PyResult<(Self, PyColumnTypeMeta)> {
        let fields = match fields {
            Some(x) => Some(into_pginterval(x).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>("expected INTERVAL_* constants")
            })?),
            None => None,
        };

        let slf = Self {
            inner: parking_lot::Mutex::new(IntervalTypeFields { fields, precision }),
        };

        Ok((slf, PyColumnTypeMeta::default()))
    }

    #[getter]
    fn fields(&self) -> Option<u8> {
        let lock = self.inner.lock();
        lock.fields.clone().map(|x| x as u8)
    }

    #[setter]
    fn set_fields(&self, val: Option<u8>) -> pyo3::PyResult<()> {
        let val = match val {
            Some(x) => Some(into_pginterval(x).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>("expected INTERVAL_* constants")
            })?),
            None => None,
        };

        let mut lock = self.inner.lock();
        lock.fields = val;
        Ok(())
    }

    #[getter]
    fn precision(&self) -> Option<u32> {
        let lock = self.inner.lock();
        lock.precision
    }

    #[setter]
    fn set_precision(&self, val: Option<u32>) {
        let mut lock = self.inner.lock();
        lock.precision = val;
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(true);
        }

        let other = other.extract::<pyo3::PyRef<'_, Self>>(slf.py()).map_err(|_| {
            typeerror!(
                "'==' not supported between instances of {} and {}",
                slf.py(),
                slf.as_ptr(),
                other.as_ptr()
            )
        })?;

        let x = other.inner.lock();
        Ok(slf.inner.lock().eq(&x))
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(false);
        }

        let other = other.extract::<pyo3::PyRef<'_, Self>>(slf.py()).map_err(|_| {
            typeerror!(
                "'==' not supported between instances of {} and {}",
                slf.py(),
                slf.as_ptr(),
                other.as_ptr()
            )
        })?;

        let x = other.inner.lock();
        Ok(slf.inner.lock().ne(&x))
    }

    fn __repr__(&self) -> String {
        let inner = self.inner.lock();

        format!(
            "<IntervalColumnType fields={:?} precision={:?}>",
            inner.fields, inner.precision,
        )
    }
}

impl AsColumnType for PyIntervalType {
    #[inline]
    fn as_column_type<'a>(&'a self, _py: pyo3::Python<'a>) -> sea_query::ColumnType {
        let n = self.inner.lock();
        sea_query::ColumnType::Interval(n.fields.clone(), n.precision)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct EnumTypeFields {
    pub(super) name: String,
    pub(super) variants: Vec<String>,
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "EnumType", frozen, extends=PyColumnTypeMeta)]
pub struct PyEnumType {
    pub(super) inner: parking_lot::Mutex<EnumTypeFields>,
}

#[pyo3::pymethods]
impl PyEnumType {
    #[new]
    fn new(name: String, mut variants: Vec<String>) -> pyo3::PyResult<(Self, PyColumnTypeMeta)> {
        if variants.is_empty() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "variants cannot be empty",
            ));
        }

        variants.sort_unstable();
        variants.dedup();

        let slf = Self {
            inner: parking_lot::Mutex::new(EnumTypeFields { name, variants }),
        };

        Ok((slf, PyColumnTypeMeta::default()))
    }

    #[getter]
    fn name(&self) -> String {
        let lock = self.inner.lock();
        lock.name.clone()
    }

    #[setter]
    fn set_name(&self, val: String) {
        let mut lock = self.inner.lock();
        lock.name = val;
    }

    #[getter]
    fn variants(&self) -> Vec<String> {
        let lock = self.inner.lock();
        lock.variants.clone()
    }

    #[setter]
    fn set_variants(&self, val: Vec<String>) {
        let mut lock = self.inner.lock();
        lock.variants = val;
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(true);
        }

        let other = other.extract::<pyo3::PyRef<'_, Self>>(slf.py()).map_err(|_| {
            typeerror!(
                "'==' not supported between instances of {} and {}",
                slf.py(),
                slf.as_ptr(),
                other.as_ptr()
            )
        })?;

        let x = other.inner.lock();
        Ok(slf.inner.lock().eq(&x))
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(false);
        }

        let other = other.extract::<pyo3::PyRef<'_, Self>>(slf.py()).map_err(|_| {
            typeerror!(
                "'==' not supported between instances of {} and {}",
                slf.py(),
                slf.as_ptr(),
                other.as_ptr()
            )
        })?;

        let x = other.inner.lock();
        Ok(slf.inner.lock().ne(&x))
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let inner = slf.inner.lock();

        format!(
            "<EnumColumnType name={:?} variants={:?}>",
            inner.name, inner.variants
        )
    }
}

impl AsColumnType for PyEnumType {
    #[inline]
    fn as_column_type<'a>(&'a self, _py: pyo3::Python<'a>) -> sea_query::ColumnType {
        use sea_query::IntoIden;

        let inner = self.inner.lock();

        sea_query::ColumnType::Enum {
            name: sea_query::Alias::new(inner.name.clone()).into_iden(),
            variants: inner
                .variants
                .iter()
                .cloned()
                .map(|x| sea_query::Alias::new(x).into_iden())
                .collect(),
        }
    }
}

#[pyo3::pyclass(module = "rapidquery._lib", name = "ArrayType", frozen, extends=PyColumnTypeMeta)]
pub struct PyArrayType {
    pub(crate) inner: parking_lot::Mutex<pyo3::Py<pyo3::PyAny>>,
}

#[pyo3::pymethods]
impl PyArrayType {
    #[new]
    fn new(py: pyo3::Python, element: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<(Self, PyColumnTypeMeta)> {
        if !element.bind(py).is_instance_of::<PyColumnTypeMeta>() {
            Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "element must be an instance of Base",
            ))
        } else {
            let slf = Self {
                inner: parking_lot::Mutex::new(element),
            };

            Ok((slf, PyColumnTypeMeta::default()))
        }
    }

    #[getter]
    fn element(&self, py: pyo3::Python) -> pyo3::Py<pyo3::PyAny> {
        let lock = self.inner.lock();
        (*lock).clone_ref(py)
    }

    #[setter]
    fn set_element(&self, py: pyo3::Python, val: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<()> {
        if !val.bind(py).is_instance_of::<PyColumnTypeMeta>() {
            Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "element must be an instance of Base",
            ))
        } else {
            let mut lock = self.inner.lock();
            *lock = val;
            Ok(())
        }
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(true);
        }

        let other = other.extract::<pyo3::PyRef<'_, Self>>(slf.py()).map_err(|_| {
            typeerror!(
                "'==' not supported between instances of {} and {}",
                slf.py(),
                slf.as_ptr(),
                other.as_ptr()
            )
        })?;

        unsafe {
            let inner1 = slf.inner.lock();
            let inner2 = other.inner.lock();

            let result =
                pyo3::ffi::PyObject_RichCompareBool((*inner1).as_ptr(), (*inner2).as_ptr(), pyo3::ffi::Py_EQ);

            if result == -1 {
                Err(pyo3::PyErr::fetch(slf.py()))
            } else {
                Ok(result == 1)
            }
        }
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(false);
        }

        let other = other.extract::<pyo3::PyRef<'_, Self>>(slf.py()).map_err(|_| {
            typeerror!(
                "'==' not supported between instances of {} and {}",
                slf.py(),
                slf.as_ptr(),
                other.as_ptr()
            )
        })?;

        unsafe {
            let inner1 = slf.inner.lock();
            let inner2 = other.inner.lock();

            let result =
                pyo3::ffi::PyObject_RichCompareBool((*inner1).as_ptr(), (*inner2).as_ptr(), pyo3::ffi::Py_NE);

            if result == -1 {
                Err(pyo3::PyErr::fetch(slf.py()))
            } else {
                Ok(result == 1)
            }
        }
    }

    fn __repr__(slf: pyo3::PyRef<'_, Self>) -> String {
        let inner = slf.inner.lock();
        format!("<ArrayColumnType element={}>", inner)
    }
}

impl AsColumnType for PyArrayType {
    #[inline]
    fn as_column_type<'a>(&'a self, py: pyo3::Python<'a>) -> sea_query::ColumnType {
        let inner = self.inner.lock();
        let col = unsafe { super::convert::convert_to_column_type(inner.bind(py)).unwrap_unchecked() };

        sea_query::ColumnType::Array(sea_query::RcOrArc::new(col))
    }
}
