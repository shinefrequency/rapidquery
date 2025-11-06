use std::ptr::NonNull;

#[derive(Debug, Default)]
pub enum DeserializedValue {
    #[default]
    Null,
    Bool(bool),
    BigInt(i64),
    BigUnsigned(u64),
    Double(f64),
    String(NonNull<pyo3::ffi::PyObject>),
    Bytes(NonNull<pyo3::ffi::PyObject>),
    Json(NonNull<pyo3::ffi::PyObject>),
    ChronoDate(NonNull<pyo3::ffi::PyObject>),
    ChronoTime(NonNull<pyo3::ffi::PyObject>),
    ChronoDateTime(
        // May have tzinfo
        NonNull<pyo3::ffi::PyObject>,
    ),
    Uuid(NonNull<pyo3::ffi::PyObject>),
    Decimal(NonNull<pyo3::ffi::PyObject>),
    Array(Vec<DeserializedValue>),
    Vector(NonNull<pyo3::ffi::PyObject>),
}

impl Clone for DeserializedValue {
    fn clone(&self) -> Self {
        unsafe {
            match self {
                Self::Null => Self::Null,
                Self::Bool(x) => Self::Bool(*x),
                Self::BigInt(x) => Self::BigInt(*x),
                Self::BigUnsigned(x) => Self::BigUnsigned(*x),
                Self::Double(x) => Self::Double(*x),
                Self::String(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::String(*x)
                }
                Self::Bytes(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::Bytes(*x)
                }
                Self::Json(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::Json(*x)
                }
                Self::ChronoDate(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::ChronoDate(*x)
                }
                Self::ChronoTime(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::ChronoTime(*x)
                }
                Self::ChronoDateTime(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::ChronoDateTime(*x)
                }
                Self::Uuid(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::Uuid(*x)
                }
                Self::Decimal(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::Decimal(*x)
                }
                Self::Array(x) => Self::Array(x.clone()),
                Self::Vector(x) => {
                    pyo3::ffi::Py_INCREF(x.as_ptr());
                    Self::Vector(*x)
                }
            }
        }
    }
}

impl Drop for DeserializedValue {
    fn drop(&mut self) {
        unsafe {
            match self {
                Self::Null => (),
                Self::Bool(_) => (),
                Self::BigInt(_) => (),
                Self::BigUnsigned(_) => (),
                Self::Double(_) => (),
                Self::String(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
                Self::Bytes(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
                Self::Json(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
                Self::ChronoDate(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
                Self::ChronoTime(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
                Self::ChronoDateTime(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
                Self::Uuid(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
                Self::Decimal(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
                Self::Array(_) => (),
                Self::Vector(x) => pyo3::ffi::Py_DECREF(x.as_ptr()),
            }
        }
    }
}

impl DeserializedValue {
    pub unsafe fn as_pyobject(&self) -> *mut pyo3::ffi::PyObject {
        match self {
            Self::Null => pyo3::ffi::Py_None(),
            Self::Bool(x) => {
                if *x {
                    pyo3::ffi::Py_True()
                } else {
                    pyo3::ffi::Py_False()
                }
            }
            Self::BigInt(x) => pyo3::ffi::PyLong_FromLongLong(*x),
            Self::BigUnsigned(x) => pyo3::ffi::PyLong_FromUnsignedLongLong(*x),
            Self::Double(x) => pyo3::ffi::PyFloat_FromDouble(*x),
            Self::String(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
            Self::Bytes(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
            Self::Json(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
            Self::ChronoDate(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
            Self::ChronoTime(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
            Self::ChronoDateTime(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
            Self::Uuid(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
            Self::Decimal(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
            Self::Array(x) => {
                let arr = pyo3::ffi::PyList_New(x.len() as isize);

                for (index, item) in x.iter().enumerate() {
                    let item = item.as_pyobject();
                    if pyo3::ffi::PyList_SetItem(arr, index as isize, item) == 0 {
                        pyo3::ffi::Py_DECREF(item);
                        pyo3::ffi::Py_DECREF(arr);
                        return std::ptr::null_mut();
                    }
                }

                arr
            }
            Self::Vector(x) => {
                let x = x.as_ptr();
                pyo3::ffi::Py_INCREF(x);
                x
            }
        }
    }

    pub fn serialize(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<super::serialize::SerializedValue> {
        use pyo3::types::PyAnyMethods;

        unsafe {
            match self {
                Self::Null => Ok(super::serialize::SerializedValue::Null),
                Self::Bool(op) => Ok(super::serialize::SerializedValue::Bool(*op)),
                Self::BigInt(op) => Ok(super::serialize::SerializedValue::BigInt(*op)),
                Self::BigUnsigned(op) => Ok(super::serialize::SerializedValue::BigUnsigned(*op)),
                Self::Double(op) => Ok(super::serialize::SerializedValue::Double(*op)),
                Self::String(op) => {
                    let mut size: pyo3::ffi::Py_ssize_t = 0;
                    let c_str = pyo3::ffi::PyUnicode_AsUTF8AndSize(op.as_ptr(), &mut size);

                    if c_str.is_null() || size < 0 {
                        Err(pyo3::PyErr::fetch(py))
                    } else {
                        let val = std::ffi::CStr::from_ptr(c_str);
                        Ok(super::serialize::SerializedValue::String(val.to_bytes().to_vec()))
                    }
                }
                Self::Bytes(op) => {
                    let bytes =
                        pyo3::Py::<pyo3::PyAny>::from_borrowed_ptr(py, op.as_ptr()).extract::<Vec<u8>>(py)?;

                    Ok(super::serialize::SerializedValue::Bytes(bytes))
                }
                Self::Json(op) => {
                    let serialized = super::common::_serialize_object_with_pyjson(py, op.as_ptr())?;

                    let mut size: pyo3::ffi::Py_ssize_t = 0;
                    let c_str = pyo3::ffi::PyUnicode_AsUTF8AndSize(serialized, &mut size);

                    if c_str.is_null() || size < 0 {
                        pyo3::ffi::Py_DECREF(serialized);
                        Err(pyo3::PyErr::fetch(py))
                    } else {
                        let val = std::ffi::CStr::from_ptr(c_str);
                        let val = serde_json::from_slice::<serde_json::Value>(val.to_bytes());

                        pyo3::ffi::Py_DECREF(serialized);

                        let val = val.map_err(|x| {
                            pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string())
                        })?;

                        Ok(super::serialize::SerializedValue::Json(val))
                    }
                }
                Self::ChronoDate(op) => {
                    let val: pyo3::Bound<'_, pyo3::types::PyDate> =
                        pyo3::Bound::from_borrowed_ptr(py, op.as_ptr()).cast_into()?;

                    Ok(super::serialize::SerializedValue::ChronoDate(val.extract()?))
                }
                Self::ChronoTime(op) => {
                    let val: pyo3::Bound<'_, pyo3::types::PyTime> =
                        pyo3::Bound::from_borrowed_ptr(py, op.as_ptr()).cast_into()?;

                    Ok(super::serialize::SerializedValue::ChronoTime(val.extract()?))
                }
                Self::ChronoDateTime(op) => {
                    let val: pyo3::Bound<'_, pyo3::types::PyDateTime> =
                        pyo3::Bound::from_borrowed_ptr(py, op.as_ptr()).cast_into()?;

                    let tzinfo =
                        pyo3::ffi::PyObject_GetAttr(val.as_ptr(), pyo3::intern!(py, "tzinfo").as_ptr());

                    debug_assert!(!tzinfo.is_null());

                    if pyo3::ffi::Py_IsNone(tzinfo) == 1 {
                        Ok(super::serialize::SerializedValue::ChronoDateTime(val.extract()?))
                    } else {
                        Ok(super::serialize::SerializedValue::ChronoDateTimeWithTimeZone(
                            val.extract()?,
                        ))
                    }
                }
                Self::Uuid(op) => {
                    let val: uuid::Uuid = pyo3::Bound::from_borrowed_ptr(py, op.as_ptr()).extract()?;

                    Ok(super::serialize::SerializedValue::Uuid(val))
                }
                Self::Decimal(op) => {
                    let val: rust_decimal::Decimal =
                        pyo3::Bound::from_borrowed_ptr(py, op.as_ptr()).extract()?;

                    Ok(super::serialize::SerializedValue::Decimal(val))
                }
                Self::Array(op) => {
                    let mut values: Vec<super::serialize::SerializedValue> = Vec::with_capacity(op.len());

                    for item in op {
                        let item = item.serialize(py)?;
                        values.push(item);
                    }

                    Ok(super::serialize::SerializedValue::Array(values))
                }
                Self::Vector(op) => {
                    let mut values: Vec<f32> = Vec::new();

                    let iterator = pyo3::Bound::from_borrowed_ptr(py, op.as_ptr());
                    for item in iterator.try_iter()? {
                        let item = item?;
                        values.push(item.extract::<f32>()?);
                    }

                    Ok(super::serialize::SerializedValue::Vector(values))
                }
            }
        }
    }
}
