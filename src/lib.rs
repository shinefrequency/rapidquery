/// Helper macros whole the crate
#[macro_use]
mod macros;

mod column;

/// RapidQuery core module written in Rust
#[pyo3::pymodule(gil_used = false)]
mod _lib {
    #[pymodule_export]
    use super::column::types::PyColumnTypeMeta;

    #[pymodule_export]
    use super::column::types::{
        PyBigIntegerType, PyBigUnsignedType, PyBinaryType, PyBitType, PyBlobType, PyBooleanType, PyCharType,
        PyCidrType, PyDateTimeType, PyDateType, PyDecimalType, PyDoubleType, PyFloatType, PyInetType, PyIntegerType,
        PyJsonBinaryType, PyJsonType, PyLTreeType, PyMacAddressType, PyMoneyType, PySmallIntegerType,
        PySmallUnsignedType, PyStringType, PyTextType, PyTimeType, PyTimestampType, PyTimestampWithTimeZoneType,
        PyTinyIntegerType, PyTinyUnsignedType, PyUnsignedType, PyUuidType, PyVarBinaryType, PyVarBitType, PyVectorType,
        PyYearType,
    };
}
