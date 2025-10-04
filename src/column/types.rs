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
#[pyo3::pyclass(module = "rapidquery._lib", name = "ColumnTypeMeta", frozen, subclass, generic)]
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
                                "'==' not supported between instances of {:?} and {:?}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr(),
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
                                "'!=' not supported between instances of {:?} and {:?}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr(),
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
                    let value = length.unwrap_or(u32::MAX);
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
                    if length == u32::MAX { None } else { Some(length) }
                }

                #[setter]
                fn set_length(&self, val: Option<u32>) {
                    let new_val = val.unwrap_or(u32::MAX);
                    self.length.store(new_val, std::sync::atomic::Ordering::Relaxed);
                }

                fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    let other = other
                        .extract::<pyo3::PyRef<'_, Self>>(slf.py())
                        .map_err(
                            |_| typeerror!(
                                "'==' not supported between instances of {:?} and {:?}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr(),
                            )
                        )?;

                    Ok(slf.as_ptr() == other.as_ptr()
                        || slf.length.load(std::sync::atomic::Ordering::Relaxed) == other.length.load(std::sync::atomic::Ordering::Relaxed))
                }

                fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    let other = other
                        .extract::<pyo3::PyRef<'_, Self>>(slf.py())
                        .map_err(
                            |_| typeerror!(
                                "'!=' not supported between instances of {:?} and {:?}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr(),
                            )
                        )?;

                    Ok(slf.as_ptr() == other.as_ptr()
                        && slf.length.load(std::sync::atomic::Ordering::Relaxed) != other.length.load(std::sync::atomic::Ordering::Relaxed))
                }

                fn __repr__(&self) -> String {
                    format!("<{} length={:?}>", $pyname, self.length.load(std::sync::atomic::Ordering::Relaxed))
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
                        precision: std::sync::atomic::AtomicU32::new(u32::MAX),
                        scale: std::sync::atomic::AtomicU32::new(u32::MAX),
                    }
                }
            }

            #[pyo3::pymethods]
            impl $name {
                #[new]
                #[pyo3(signature=(precision_scale=None))]
                fn new(precision_scale: Option<(u32, u32)>) -> (Self, PyColumnTypeMeta) {
                    let (precision, scale) = precision_scale.unwrap_or((u32::MAX, u32::MAX));
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

                    if precision == u32::MAX || scale == u32::MAX {
                        None
                    } else {
                        Some((precision, scale))
                    }
                }

                #[setter]
                fn set_precision_scale(&self, val: Option<(u32, u32)>) {
                    let (precision, scale) = val.unwrap_or((u32::MAX, u32::MAX));
                    self.precision.store(precision, std::sync::atomic::Ordering::Relaxed);
                    self.scale.store(scale, std::sync::atomic::Ordering::Relaxed);
                }

                fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    let other = other
                        .extract::<pyo3::PyRef<'_, Self>>(slf.py())
                        .map_err(
                            |_| typeerror!(
                                "'==' not supported between instances of {:?} and {:?}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr(),
                            )
                        )?;

                    Ok(slf.as_ptr() == other.as_ptr() || slf.precision_scale() == other.precision_scale())
                }

                fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::Py<pyo3::PyAny>) -> pyo3::PyResult<bool> {
                    let other = other
                        .extract::<pyo3::PyRef<'_, Self>>(slf.py())
                        .map_err(
                            |_| typeerror!(
                                "'!=' not supported between instances of {:?} and {:?}",
                                slf.py(),
                                slf.as_ptr(),
                                other.as_ptr(),
                            )
                        )?;

                    Ok(slf.as_ptr() == other.as_ptr() && slf.precision_scale() != other.precision_scale())
                }

                fn __repr__(&self) -> String {
                    format!("<{} precision_scale={:?}>", $pyname, self.precision_scale())
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
