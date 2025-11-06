use pyo3::types::PyAnyMethods;

unsafe fn extract_and_convert<T>(py: pyo3::Python, obj_ptr: *mut pyo3::ffi::PyObject) -> sea_query::ColumnType
where
    T: pyo3::PyClass<Frozen = pyo3::pyclass::boolean_struct::True> + std::marker::Sync,
    T: super::types::AsColumnType,
{
    let val = pyo3::Bound::from_borrowed_ptr(py, obj_ptr);
    let val = val.cast_unchecked::<T>();

    val.get().as_column_type(py)
}

macro_rules! try_convert_column_type {
    ($py:expr, $ptr:expr, $type_ptr:expr, $($constant:expr => $py_type:ty),*) => {
        $(
            if $type_ptr == $constant {
                return Some(extract_and_convert::<$py_type>($py, $ptr));
            }
        )*
    };
}

pub fn convert_to_column_type(obj: &pyo3::Bound<'_, pyo3::PyAny>) -> Option<sea_query::ColumnType> {
    if std::hint::unlikely(!obj.is_instance_of::<super::types::PyColumnTypeMeta>()) {
        return None;
    }

    unsafe {
        let ptr = obj.as_ptr();
        let type_ptr = pyo3::ffi::Py_TYPE(ptr);

        try_convert_column_type!(
            obj.py(),
            ptr,
            type_ptr,
            crate::typeref::CHAR_COLUMN_TYPE => super::types::PyCharType,
            crate::typeref::STRING_COLUMN_TYPE => super::types::PyStringType,
            crate::typeref::TEXT_COLUMN_TYPE => super::types::PyTextType,
            crate::typeref::TINY_INTEGER_COLUMN_TYPE => super::types::PyTinyIntegerType,
            crate::typeref::SMALL_INTEGER_COLUMN_TYPE => super::types::PySmallIntegerType,
            crate::typeref::INTEGER_COLUMN_TYPE => super::types::PyIntegerType,
            crate::typeref::BIG_INTEGER_COLUMN_TYPE => super::types::PyBigIntegerType,
            crate::typeref::TINY_UNSIGNED_COLUMN_TYPE => super::types::PyTinyUnsignedType,
            crate::typeref::SMALL_UNSIGNED_COLUMN_TYPE => super::types::PySmallUnsignedType,
            crate::typeref::UNSIGNED_COLUMN_TYPE => super::types::PyUnsignedType,
            crate::typeref::BIG_UNSIGNED_COLUMN_TYPE => super::types::PyBigUnsignedType,
            crate::typeref::FLOAT_COLUMN_TYPE => super::types::PyFloatType,
            crate::typeref::DOUBLE_COLUMN_TYPE => super::types::PyDoubleType,
            crate::typeref::DECIMAL_COLUMN_TYPE => super::types::PyDecimalType,
            crate::typeref::DATETIME_COLUMN_TYPE => super::types::PyDateTimeType,
            crate::typeref::TIMESTAMP_COLUMN_TYPE => super::types::PyTimestampType,
            crate::typeref::TIMESTAMP_WITH_TIMEZONE_COLUMN_TYPE => super::types::PyTimestampWithTimeZoneType,
            crate::typeref::TIME_COLUMN_TYPE => super::types::PyTimeType,
            crate::typeref::DATE_COLUMN_TYPE => super::types::PyDateType,
            crate::typeref::YEAR_COLUMN_TYPE => super::types::PyYearType,
            crate::typeref::BLOB_COLUMN_TYPE => super::types::PyBlobType,
            crate::typeref::BINARY_COLUMN_TYPE => super::types::PyBinaryType,
            crate::typeref::VAR_BINARY_COLUMN_TYPE => super::types::PyVarBinaryType,
            crate::typeref::BIT_COLUMN_TYPE => super::types::PyBitType,
            crate::typeref::VAR_BIT_COLUMN_TYPE => super::types::PyVarBitType,
            crate::typeref::BOOLEAN_COLUMN_TYPE => super::types::PyBooleanType,
            crate::typeref::MONEY_COLUMN_TYPE => super::types::PyMoneyType,
            crate::typeref::JSON_COLUMN_TYPE => super::types::PyJsonType,
            crate::typeref::JSON_BINARY_COLUMN_TYPE => super::types::PyJsonBinaryType,
            crate::typeref::UUID_COLUMN_TYPE => super::types::PyUuidType,
            crate::typeref::VECTOR_COLUMN_TYPE => super::types::PyVectorType,
            crate::typeref::CIDR_COLUMN_TYPE => super::types::PyCidrType,
            crate::typeref::INET_COLUMN_TYPE => super::types::PyInetType,
            crate::typeref::MAC_ADDR_COLUMN_TYPE => super::types::PyMacAddressType,
            crate::typeref::LTREE_COLUMN_TYPE => super::types::PyLTreeType,
            crate::typeref::INTERVAL_COLUMN_TYPE => super::types::PyIntervalType,
            crate::typeref::ENUM_COLUMN_TYPE => super::types::PyEnumType,
            crate::typeref::ARRAY_COLUMN_TYPE => super::types::PyArrayType
        );
    }

    unreachable!()
}
