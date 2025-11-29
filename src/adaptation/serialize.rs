use std::ptr::NonNull;

#[derive(Clone, Debug, PartialEq)]
pub enum RustValue {
    Null,
    Bool(bool),
    BigInt(i64),
    BigUnsigned(u64),
    Double(f64),
    String(Vec<u8>),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
    ChronoDate(chrono::NaiveDate),
    ChronoTime(chrono::NaiveTime),
    ChronoDateTime(chrono::NaiveDateTime),
    ChronoDateTimeWithTimeZone(chrono::DateTime<chrono::FixedOffset>),
    Uuid(uuid::Uuid),
    Decimal(rust_decimal::Decimal),
    Array(Vec<RustValue>),
    Vector(Vec<f32>),
}

impl RustValue {
    pub fn deserialize(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<super::deserialize::PythonValue> {
        use chrono::{Datelike, Timelike};
        use pyo3::IntoPyObject;

        unsafe {
            match self {
                Self::Null => Ok(super::deserialize::PythonValue::Null),
                Self::Bool(x) => Ok(super::deserialize::PythonValue::Bool(*x)),
                Self::BigInt(x) => Ok(super::deserialize::PythonValue::BigInt(*x)),
                Self::BigUnsigned(x) => Ok(super::deserialize::PythonValue::BigUnsigned(*x)),
                Self::Double(x) => Ok(super::deserialize::PythonValue::Double(*x)),
                Self::String(x) => {
                    let val = pyo3::types::PyString::intern(py, std::str::from_utf8_unchecked(x));

                    Ok(super::deserialize::PythonValue::String(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
                Self::Bytes(x) => {
                    let val = pyo3::types::PyBytes::new(py, x);

                    Ok(super::deserialize::PythonValue::Bytes(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
                Self::Json(x) => {
                    let encoded = serde_json::to_vec(x)
                        .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string()))?;
                    let val = pyo3::types::PyString::intern(py, std::str::from_utf8_unchecked(&encoded));

                    let val = super::common::_deserialize_object_with_pyjson(py, val.as_ptr())?;
                    Ok(super::deserialize::PythonValue::Json(
                        NonNull::new_unchecked(val),
                    ))
                }
                Self::ChronoDate(x) => {
                    let val =
                        pyo3::types::PyDate::new(py, x.year(), (x.month0() + 1) as u8, (x.day0() + 1) as u8)?;

                    Ok(super::deserialize::PythonValue::ChronoDate(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
                Self::ChronoTime(x) => {
                    let val = pyo3::types::PyTime::new(
                        py,
                        x.hour() as u8,
                        x.minute() as u8,
                        x.second() as u8,
                        0,
                        None,
                    )
                    .unwrap();

                    Ok(super::deserialize::PythonValue::Bytes(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
                Self::ChronoDateTime(x) => {
                    let val = x.into_pyobject(py)?;

                    Ok(super::deserialize::PythonValue::ChronoDateTime(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
                Self::ChronoDateTimeWithTimeZone(x) => {
                    let val = x.into_pyobject(py)?;

                    Ok(super::deserialize::PythonValue::ChronoDateTime(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
                Self::Uuid(x) => {
                    let val = x.into_pyobject(py)?;

                    Ok(super::deserialize::PythonValue::Uuid(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
                Self::Decimal(x) => {
                    let val = x.into_pyobject(py)?;

                    Ok(super::deserialize::PythonValue::Decimal(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
                Self::Array(x) => Ok(super::deserialize::PythonValue::Array(
                    x.iter().map(|x| x.deserialize(py).unwrap()).collect(),
                )),
                Self::Vector(x) => {
                    let val = x.into_pyobject(py)?;

                    Ok(super::deserialize::PythonValue::Vector(
                        NonNull::new_unchecked(val.into_ptr()),
                    ))
                }
            }
        }
    }
}

impl From<RustValue> for sea_query::Value {
    #[inline]
    fn from(value: RustValue) -> Self {
        match value {
            RustValue::Null => Self::BigInt(None),
            RustValue::Bool(x) => Self::Bool(Some(x)),
            RustValue::BigInt(x) => Self::BigInt(Some(x)),
            RustValue::BigUnsigned(x) => Self::BigUnsigned(Some(x)),
            RustValue::Double(x) => Self::Double(Some(x)),
            RustValue::String(x) => {
                Self::String(Some(Box::new(unsafe { String::from_utf8_unchecked(x) })))
            }
            RustValue::Bytes(x) => Self::Bytes(Some(Box::new(x.to_vec()))),
            RustValue::Json(x) => Self::Json(Some(Box::new(x))),
            RustValue::ChronoDate(x) => Self::ChronoDate(Some(Box::new(x))),
            RustValue::ChronoTime(x) => Self::ChronoTime(Some(Box::new(x))),
            RustValue::ChronoDateTime(x) => Self::ChronoDateTime(Some(Box::new(x))),
            RustValue::ChronoDateTimeWithTimeZone(x) => {
                Self::ChronoDateTimeWithTimeZone(Some(Box::new(x)))
            }
            RustValue::Uuid(x) => Self::Uuid(Some(Box::new(x))),
            RustValue::Decimal(x) => Self::Decimal(Some(Box::new(x))),
            RustValue::Array(x) => {
                Self::Array(
                    /* this parameter is unusable and not important */
                    sea_query::ArrayType::BigInt,
                    Some(Box::new(x.into_iter().map(|x| x.into()).collect())),
                )
            }
            RustValue::Vector(x) => Self::Vector(Some(Box::new(pgvector::Vector::from(x)))),
        }
    }
}

impl From<sea_query::Value> for RustValue {
    #[inline]
    fn from(value: sea_query::Value) -> Self {
        match value {
            sea_query::Value::TinyInt(None)
            | sea_query::Value::SmallInt(None)
            | sea_query::Value::Int(None)
            | sea_query::Value::BigInt(None)
            | sea_query::Value::TinyUnsigned(None)
            | sea_query::Value::SmallUnsigned(None)
            | sea_query::Value::Unsigned(None)
            | sea_query::Value::BigUnsigned(None)
            | sea_query::Value::Float(None)
            | sea_query::Value::Double(None)
            | sea_query::Value::String(None)
            | sea_query::Value::Char(None)
            | sea_query::Value::Bytes(None)
            | sea_query::Value::Bool(None)
            | sea_query::Value::Json(None)
            | sea_query::Value::ChronoDate(None)
            | sea_query::Value::ChronoTime(None)
            | sea_query::Value::ChronoDateTime(None)
            | sea_query::Value::ChronoDateTimeLocal(None)
            | sea_query::Value::ChronoDateTimeUtc(None)
            | sea_query::Value::ChronoDateTimeWithTimeZone(None)
            | sea_query::Value::Uuid(None)
            | sea_query::Value::Decimal(None)
            | sea_query::Value::Array(_, None)
            | sea_query::Value::Vector(None)
            | sea_query::Value::IpNetwork(None)
            | sea_query::Value::MacAddress(None) => Self::Null,
            sea_query::Value::Bool(Some(x)) => Self::Bool(x),
            sea_query::Value::BigInt(Some(x)) => Self::BigInt(x),
            sea_query::Value::BigUnsigned(Some(x)) => Self::BigUnsigned(x),
            sea_query::Value::Double(Some(x)) => Self::Double(x),
            sea_query::Value::String(Some(x)) => Self::String((*x).into()),
            sea_query::Value::Bytes(Some(x)) => Self::Bytes((*x).clone()),
            sea_query::Value::Json(Some(x)) => Self::Json(*x),
            sea_query::Value::ChronoDate(Some(x)) => Self::ChronoDate(*x),
            sea_query::Value::ChronoTime(Some(x)) => Self::ChronoTime(*x),
            sea_query::Value::ChronoDateTime(Some(x)) => Self::ChronoDateTime(*x),
            sea_query::Value::ChronoDateTimeWithTimeZone(Some(x)) => Self::ChronoDateTimeWithTimeZone(*x),
            sea_query::Value::Uuid(Some(x)) => Self::Uuid(*x),
            sea_query::Value::Decimal(Some(x)) => Self::Decimal(*x),
            sea_query::Value::Array(_, Some(x)) => Self::Array(x.into_iter().map(|x| x.into()).collect()),
            sea_query::Value::Vector(Some(x)) => Self::Vector((*x).into()),
            _ => unreachable!(),
        }
    }
}
