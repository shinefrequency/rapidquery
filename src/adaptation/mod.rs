use pyo3::types::PyAnyMethods;
use std::ptr::NonNull;

mod common;
mod deserialize;
mod serialize;

pub use deserialize::DeserializedValue;
pub use serialize::SerializedValue;

/// A bridge between Python & [`sea_query::Value`]
#[derive(Debug, Clone)]
pub struct ReturnableValue {
    deserialized: Option<DeserializedValue>,
    serialized: Option<SerializedValue>,
}

unsafe impl Send for ReturnableValue {}

impl From<DeserializedValue> for ReturnableValue {
    #[inline]
    fn from(value: DeserializedValue) -> Self {
        Self {
            deserialized: Some(value),
            serialized: None,
        }
    }
}

impl From<SerializedValue> for ReturnableValue {
    #[inline]
    fn from(value: SerializedValue) -> Self {
        Self {
            deserialized: None,
            serialized: Some(value),
        }
    }
}

impl ReturnableValue {
    #[inline]
    pub fn with_specific_type(
        object: pyo3::Bound<'_, pyo3::PyAny>,
        r#type: std::sync::Arc<sea_query::ColumnType>,
    ) -> pyo3::PyResult<Self> {
        // Validate object depend on `type`
        // Note that we're only using BigInt and BigUnsigned for integers,
        // so there's no different between tiny, small, or normal integers for us.
        match &*r#type {
            sea_query::ColumnType::Boolean => unsafe {
                if pyo3::ffi::PyBool_Check(object.as_ptr()) == 0 {
                    return Err(typeerror!("expected bool, got {}", object.py(), object.as_ptr()));
                }

                Ok(Self::from(DeserializedValue::Bool(
                    pyo3::ffi::Py_True() == object.as_ptr(),
                )))
            },
            sea_query::ColumnType::TinyInteger
            | sea_query::ColumnType::SmallInteger
            | sea_query::ColumnType::Integer
            | sea_query::ColumnType::BigInteger
            | sea_query::ColumnType::Year => unsafe {
                let val = pyo3::ffi::PyLong_AsLongLong(object.as_ptr());
                if val == -1 && !pyo3::ffi::PyErr_Occurred().is_null() {
                    return Err(pyo3::PyErr::fetch(object.py()));
                }

                Ok(Self::from(DeserializedValue::BigInt(val)))
            },
            sea_query::ColumnType::TinyUnsigned
            | sea_query::ColumnType::SmallUnsigned
            | sea_query::ColumnType::Unsigned
            | sea_query::ColumnType::BigUnsigned => unsafe {
                let val = pyo3::ffi::PyLong_AsUnsignedLongLong(object.as_ptr());
                if val == u64::MAX && !pyo3::ffi::PyErr_Occurred().is_null() {
                    return Err(pyo3::PyErr::fetch(object.py()));
                }

                Ok(Self::from(DeserializedValue::BigUnsigned(val)))
            },
            sea_query::ColumnType::Char(_)
            | sea_query::ColumnType::String(_)
            | sea_query::ColumnType::Text
            | sea_query::ColumnType::Interval(_, _)
            | sea_query::ColumnType::Cidr
            | sea_query::ColumnType::Inet
            | sea_query::ColumnType::MacAddr
            | sea_query::ColumnType::LTree => unsafe {
                if pyo3::ffi::PyUnicode_CheckExact(object.as_ptr()) == 0 {
                    return Err(typeerror!("expected str, got {}", object.py(), object.as_ptr()));
                }

                Ok(Self::from(DeserializedValue::String(NonNull::new_unchecked(
                    object.into_ptr(),
                ))))
            },
            sea_query::ColumnType::Blob
            | sea_query::ColumnType::Binary(_)
            | sea_query::ColumnType::VarBinary(_)
            | sea_query::ColumnType::Bit(_)
            | sea_query::ColumnType::VarBit(_) => unsafe {
                if pyo3::ffi::PyBytes_CheckExact(object.as_ptr()) == 0 {
                    return Err(typeerror!("expected bytes, got {}", object.py(), object.as_ptr()));
                }

                Ok(Self::from(DeserializedValue::Bytes(NonNull::new_unchecked(
                    object.into_ptr(),
                ))))
            },
            sea_query::ColumnType::Float | sea_query::ColumnType::Double => unsafe {
                let val = pyo3::ffi::PyFloat_AsDouble(object.as_ptr());
                if val == -1.0 && !pyo3::ffi::PyErr_Occurred().is_null() {
                    return Err(pyo3::PyErr::fetch(object.py()));
                }

                Ok(Self::from(DeserializedValue::Double(val)))
            },
            sea_query::ColumnType::Decimal(_) | sea_query::ColumnType::Money(_) => unsafe {
                if pyo3::ffi::Py_IS_TYPE(object.as_ptr(), crate::typeref::STD_DECIMAL_TYPE) == 0 {
                    return Err(typeerror!(
                        "expected decimal.Decimal, got {}",
                        object.py(),
                        object.as_ptr()
                    ));
                }

                Ok(Self::from(DeserializedValue::Decimal(NonNull::new_unchecked(
                    object.into_ptr(),
                ))))
            },
            sea_query::ColumnType::DateTime | sea_query::ColumnType::Timestamp => unsafe {
                if pyo3::ffi::Py_IS_TYPE(object.as_ptr(), crate::typeref::STD_DATETIME_TYPE) == 0 {
                    return Err(typeerror!(
                        "expected datetime.datetime, got {}",
                        object.py(),
                        object.as_ptr()
                    ));
                }

                Ok(Self::from(DeserializedValue::ChronoDateTime(
                    NonNull::new_unchecked(object.into_ptr()),
                )))
            },
            sea_query::ColumnType::TimestampWithTimeZone => unsafe {
                if pyo3::ffi::Py_IS_TYPE(object.as_ptr(), crate::typeref::STD_DATETIME_TYPE) == 0 {
                    return Err(typeerror!(
                        "expected datetime.datetime, got {}",
                        object.py(),
                        object.as_ptr()
                    ));
                }

                Ok(Self::from(DeserializedValue::ChronoDateTime(
                    NonNull::new_unchecked(object.into_ptr()),
                )))
            },
            sea_query::ColumnType::Time => unsafe {
                if pyo3::ffi::Py_IS_TYPE(object.as_ptr(), crate::typeref::STD_TIME_TYPE) == 0 {
                    return Err(typeerror!(
                        "expected datetime.time, got {}",
                        object.py(),
                        object.as_ptr()
                    ));
                }

                Ok(Self::from(DeserializedValue::ChronoTime(NonNull::new_unchecked(
                    object.into_ptr(),
                ))))
            },
            sea_query::ColumnType::Date => unsafe {
                if pyo3::ffi::Py_IS_TYPE(object.as_ptr(), crate::typeref::STD_DATE_TYPE) == 0 {
                    return Err(typeerror!(
                        "expected datetime.date, got {}",
                        object.py(),
                        object.as_ptr()
                    ));
                }

                Ok(Self::from(DeserializedValue::ChronoDate(NonNull::new_unchecked(
                    object.into_ptr(),
                ))))
            },
            sea_query::ColumnType::Json | sea_query::ColumnType::JsonBinary => unsafe {
                common::_validate_json_object(object.py(), object.as_ptr())?;

                Ok(Self::from(DeserializedValue::Json(NonNull::new_unchecked(
                    object.into_ptr(),
                ))))
            },
            sea_query::ColumnType::Uuid => unsafe {
                if pyo3::ffi::Py_IS_TYPE(object.as_ptr(), crate::typeref::STD_UUID_TYPE) == 0 {
                    return Err(typeerror!(
                        "expected uuid.UUID, got {}",
                        object.py(),
                        object.as_ptr()
                    ));
                }

                Ok(Self::from(DeserializedValue::Uuid(NonNull::new_unchecked(
                    object.into_ptr(),
                ))))
            },
            sea_query::ColumnType::Custom(_) => unimplemented!(),
            sea_query::ColumnType::Enum { .. } => unsafe {
                // TODO: support enum.EnumMeta
                if pyo3::ffi::PyUnicode_CheckExact(object.as_ptr()) == 0 {
                    return Err(typeerror!("expected str, got {}", object.py(), object.as_ptr()));
                }

                Ok(Self::from(DeserializedValue::String(NonNull::new_unchecked(
                    object.into_ptr(),
                ))))
            },
            sea_query::ColumnType::Array(ty) => unsafe {
                use pyo3::types::PyListMethods;

                if pyo3::ffi::PyList_CheckExact(object.as_ptr()) == 0 {
                    return Err(typeerror!("expected list, got {}", object.py(), object.as_ptr()));
                }

                let list = object.cast_into_unchecked::<pyo3::types::PyList>();
                let mut values = Vec::with_capacity(list.len());

                for item in list.iter() {
                    let x = Self::with_specific_type(item, std::sync::Arc::clone(ty))?;
                    values.push(x.deserialized.unwrap());
                }

                Ok(Self::from(DeserializedValue::Array(values)))
            },
            sea_query::ColumnType::Vector(_) => unsafe {
                use pyo3::types::PyListMethods;

                if pyo3::ffi::PyList_CheckExact(object.as_ptr()) == 0 {
                    return Err(typeerror!(
                        "expected list of floats, got {}",
                        object.py(),
                        object.as_ptr()
                    ));
                }

                let list = object.cast_into_unchecked::<pyo3::types::PyList>();

                for item in list.iter() {
                    if pyo3::ffi::PyFloat_CheckExact(item.as_ptr()) == 0 {
                        return Err(typeerror!(
                            "expected list of floats, found an {:?} element",
                            item.py(),
                            item.as_ptr()
                        ));
                    }
                }

                Ok(Self::from(DeserializedValue::Vector(NonNull::new_unchecked(
                    list.into_ptr(),
                ))))
            },
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    #[inline]
    pub fn infer_pyobject_type(object: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::PyLong_CheckExact(object.as_ptr()) == 1 {
                return Self::with_specific_type(
                    object,
                    std::sync::Arc::new(sea_query::ColumnType::BigInteger),
                );
            }

            if pyo3::ffi::PyFloat_CheckExact(object.as_ptr()) == 1 {
                return Self::with_specific_type(object, std::sync::Arc::new(sea_query::ColumnType::Double));
            }

            if pyo3::ffi::PyUnicode_CheckExact(object.as_ptr()) == 1 {
                return Ok(Self::from(DeserializedValue::String(NonNull::new_unchecked(
                    object.into_ptr(),
                ))));
            }

            if pyo3::ffi::PyBool_Check(object.as_ptr()) == 1 {
                return Ok(Self::from(DeserializedValue::Bool(
                    pyo3::ffi::Py_True() == object.as_ptr(),
                )));
            }

            if pyo3::ffi::PyBytes_CheckExact(object.as_ptr()) == 1 {
                return Ok(Self::from(DeserializedValue::Bytes(NonNull::new_unchecked(
                    object.into_ptr(),
                ))));
            }

            if pyo3::ffi::PyDict_CheckExact(object.as_ptr()) == 1
                || pyo3::ffi::PyList_CheckExact(object.as_ptr()) == 1
            {
                common::_validate_json_object(object.py(), object.as_ptr())?;

                return Ok(Self::from(DeserializedValue::Json(NonNull::new_unchecked(
                    object.into_ptr(),
                ))));
            }

            if pyo3::ffi::Py_TYPE(object.as_ptr()) == crate::typeref::STD_DECIMAL_TYPE {
                return Ok(Self::from(DeserializedValue::Decimal(NonNull::new_unchecked(
                    object.into_ptr(),
                ))));
            }

            if pyo3::ffi::Py_TYPE(object.as_ptr()) == crate::typeref::STD_DATETIME_TYPE {
                return Ok(Self::from(DeserializedValue::ChronoDateTime(
                    NonNull::new_unchecked(object.into_ptr()),
                )));
            }

            if pyo3::ffi::Py_TYPE(object.as_ptr()) == crate::typeref::STD_DATE_TYPE {
                return Ok(Self::from(DeserializedValue::ChronoDate(NonNull::new_unchecked(
                    object.into_ptr(),
                ))));
            }

            if pyo3::ffi::Py_TYPE(object.as_ptr()) == crate::typeref::STD_TIME_TYPE {
                return Ok(Self::from(DeserializedValue::ChronoTime(NonNull::new_unchecked(
                    object.into_ptr(),
                ))));
            }

            if pyo3::ffi::Py_TYPE(object.as_ptr()) == crate::typeref::STD_UUID_TYPE {
                return Ok(Self::from(DeserializedValue::Uuid(NonNull::new_unchecked(
                    object.into_ptr(),
                ))));
            }
        }

        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "could not infer SQL type for {:?}",
            unsafe { crate::macros::get_type_name(object.py(), object.as_ptr()) }
        )))
    }

    #[inline]
    pub fn from_bound(
        object: pyo3::Bound<'_, pyo3::PyAny>,
        r#type: Option<&pyo3::Bound<'_, pyo3::PyAny>>,
    ) -> pyo3::PyResult<Self> {
        unsafe {
            if pyo3::ffi::Py_IsNone(object.as_ptr()) == 1 {
                return Ok(Self {
                    deserialized: Some(DeserializedValue::Null),
                    serialized: Some(SerializedValue::Null),
                });
            }
        }

        if r#type.is_none() {
            return Self::infer_pyobject_type(object);
        }

        let r#type = unsafe {
            crate::column::convert::convert_to_column_type(r#type.unwrap_unchecked()).ok_or_else(|| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "could not detect column type. are you sure you're using BaseColumnType instances?",
                )
            })?
        };

        Self::with_specific_type(object, std::sync::Arc::new(r#type))
    }

    #[inline]
    pub fn serialize(&mut self, py: pyo3::Python<'_>) -> &SerializedValue {
        unsafe {
            if self.serialized.is_none() {
                self.serialized = Some(
                    self.deserialized
                        .as_ref()
                        .unwrap_unchecked()
                        .serialize(py)
                        .unwrap(),
                );
            }

            self.serialized.as_ref().unwrap_unchecked()
        }
    }

    #[inline]
    pub fn deserialize(&mut self, py: pyo3::Python<'_>) -> &DeserializedValue {
        unsafe {
            if self.deserialized.is_none() {
                self.deserialized = Some(
                    self.serialized
                        .as_ref()
                        .unwrap_unchecked()
                        .deserialize(py)
                        .unwrap(),
                );
            }

            self.deserialized.as_ref().unwrap_unchecked()
        }
    }

    #[inline]
    pub fn create_simple_expr(&mut self, py: pyo3::Python<'_>) -> sea_query::SimpleExpr {
        let converted = self.serialize(py);
        sea_query::SimpleExpr::Value(converted.clone().into())
    }
}

/// Bridges Python types, Rust types, and SQL types for seamless data
/// conversion.
///
/// This class handles validation, adaptation, and conversion
/// between different type systems used in the application stack.
#[pyo3::pyclass(module = "rapidquery._lib", name = "AdaptedValue", frozen, generic)]
pub struct PyAdaptedValue {
    pub(crate) inner: parking_lot::Mutex<ReturnableValue>,
}

impl From<ReturnableValue> for PyAdaptedValue {
    fn from(value: ReturnableValue) -> Self {
        Self {
            inner: parking_lot::Mutex::new(value),
        }
    }
}

#[pyo3::pymethods]
impl PyAdaptedValue {
    #[new]
    #[pyo3(signature=(value, r#type=None))]
    pub fn new(
        value: pyo3::Bound<'_, pyo3::PyAny>,
        r#type: Option<pyo3::Bound<'_, pyo3::PyAny>>,
    ) -> pyo3::PyResult<pyo3::PyClassInitializer<Self>> {
        if value.is_instance_of::<Self>() {
            return Ok(pyo3::PyClassInitializer::from(unsafe {
                value.cast_into_unchecked::<Self>()
            }));
        }

        let result = ReturnableValue::from_bound(value, r#type.as_ref())?;
        let slf = Self {
            inner: parking_lot::Mutex::new(result),
        };

        Ok(pyo3::PyClassInitializer::from(slf))
    }

    #[getter]
    fn is_null(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Null))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Null))
    }

    #[getter]
    fn is_integer(&self) -> bool {
        let lock = self.inner.lock();

        matches!(
            lock.deserialized.as_ref(),
            Some(DeserializedValue::BigInt(_)) | Some(DeserializedValue::BigUnsigned(_))
        ) || matches!(
            lock.serialized.as_ref(),
            Some(SerializedValue::BigInt(_)) | Some(SerializedValue::BigUnsigned(_))
        )
    }

    #[getter]
    fn is_float(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Double(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Double(_)))
    }

    #[getter]
    fn is_boolean(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Bool(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Bool(_)))
    }

    #[getter]
    fn is_string(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::String(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::String(_)))
    }

    #[getter]
    fn is_date(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::ChronoDate(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::ChronoDate(_)))
    }

    #[getter]
    fn is_datetime(&self) -> bool {
        let lock = self.inner.lock();

        matches!(
            lock.deserialized.as_ref(),
            Some(DeserializedValue::ChronoDateTime(_))
        ) || matches!(
            lock.serialized.as_ref(),
            Some(SerializedValue::ChronoDateTime(_)) | Some(SerializedValue::ChronoDateTimeWithTimeZone(_))
        )
    }

    #[getter]
    fn is_time(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::ChronoTime(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::ChronoTime(_)))
    }

    #[getter]
    fn is_uuid(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Uuid(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Uuid(_)))
    }

    #[getter]
    fn is_bytes(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Bytes(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Bytes(_)))
    }

    #[getter]
    fn is_json(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Json(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Json(_)))
    }

    #[getter]
    fn is_decimal(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Decimal(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Decimal(_)))
    }

    #[getter]
    fn is_array(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Array(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Array(_)))
    }

    #[getter]
    fn is_vector(&self) -> bool {
        let lock = self.inner.lock();

        matches!(lock.deserialized.as_ref(), Some(DeserializedValue::Vector(_)))
            || matches!(lock.serialized.as_ref(), Some(SerializedValue::Vector(_)))
    }

    #[getter]
    fn value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        let mut lock = self.inner.lock();
        let obj = lock.deserialize(py);

        unsafe { pyo3::Py::from_borrowed_ptr_or_err(py, obj.as_pyobject()) }
    }

    fn __hash__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<isize> {
        let obj = unsafe {
            let mut lock = self.inner.lock();
            lock.deserialize(py).as_pyobject()
        };

        unsafe {
            let hash = pyo3::ffi::PyObject_Hash(obj);
            pyo3::ffi::Py_DECREF(obj);

            if hash == -1 {
                Err(pyo3::PyErr::fetch(py))
            } else {
                Ok(hash)
            }
        }
    }

    fn __eq__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(true);
        }

        let mut inner1 = slf.inner.lock();
        let mut inner2 = other.inner.lock();

        if let (Some(x), Some(y)) = (&inner1.serialized, &inner2.serialized) {
            return Ok(x == y);
        }

        unsafe {
            let obj1 = inner1.deserialize(slf.py()).as_pyobject();
            let obj2 = inner2.deserialize(slf.py()).as_pyobject();

            let result = pyo3::ffi::PyObject_RichCompareBool(obj1, obj2, pyo3::ffi::Py_EQ);
            pyo3::ffi::Py_DECREF(obj1);
            pyo3::ffi::Py_DECREF(obj2);

            if result == -1 {
                Err(pyo3::PyErr::fetch(slf.py()))
            } else {
                Ok(result == 1)
            }
        }
    }

    fn __ne__(slf: pyo3::PyRef<'_, Self>, other: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<bool> {
        if slf.as_ptr() == other.as_ptr() {
            return Ok(false);
        }

        let mut inner1 = slf.inner.lock();
        let mut inner2 = other.inner.lock();

        if let (Some(x), Some(y)) = (&inner1.serialized, &inner2.serialized) {
            return Ok(x != y);
        }

        unsafe {
            let obj1 = inner1.deserialize(slf.py()).as_pyobject();
            let obj2 = inner2.deserialize(slf.py()).as_pyobject();

            let result = pyo3::ffi::PyObject_RichCompareBool(obj1, obj2, pyo3::ffi::Py_EQ);
            pyo3::ffi::Py_DECREF(obj1);
            pyo3::ffi::Py_DECREF(obj2);

            if result == -1 {
                Err(pyo3::PyErr::fetch(slf.py()))
            } else {
                Ok(result == 0)
            }
        }
    }

    fn __copy__(&self) -> Self {
        Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone()),
        }
    }

    fn copy(&self) -> Self {
        Self {
            inner: parking_lot::Mutex::new(self.inner.lock().clone()),
        }
    }

    fn to_sql(&self, backend: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<String> {
        let mut lock = self.inner.lock();
        let expr = lock.create_simple_expr(backend.py());

        let mut sql = String::new();

        prepare_sql!(
            crate::backend::into_query_builder => backend => prepare_simple_expr(&expr, &mut sql)
        )?;

        Ok(sql)
    }

    fn __repr__(&self) -> String {
        let lock = self.inner.lock();

        if let Some(x) = &lock.serialized {
            format!("<AdaptedValue[adapted] {x:?}>")
        } else if let Some(x) = &lock.deserialized {
            format!("<AdaptedValue[inferred] {x:?}>")
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }
}
