#![feature(likely_unlikely)]
#![feature(optimize_attribute)]

/// Helper macros whole the crate
#[macro_use]
mod macros;

mod adaptation;
mod column;
mod typeref;

/// RapidQuery core module written in Rust
#[pyo3::pymodule(gil_used = false)]
mod _lib {
    use pyo3::types::PyModuleMethods;

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

    #[pymodule_export]
    use super::adaptation::PyAdaptedValue;

    #[pymodule_init]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        m.add("INTERVAL_YEAR", sea_query::PgInterval::Year as u8)?;
        m.add("INTERVAL_MONTH", sea_query::PgInterval::Month as u8)?;
        m.add("INTERVAL_DAY", sea_query::PgInterval::Day as u8)?;
        m.add("INTERVAL_HOUR", sea_query::PgInterval::Hour as u8)?;
        m.add("INTERVAL_MINUTE", sea_query::PgInterval::Minute as u8)?;
        m.add("INTERVAL_SECOND", sea_query::PgInterval::Second as u8)?;
        m.add("INTERVAL_YEAR_TO_MONTH", sea_query::PgInterval::YearToMonth as u8)?;
        m.add("INTERVAL_DAY_TO_HOUR", sea_query::PgInterval::DayToHour as u8)?;
        m.add("INTERVAL_DAY_TO_MINUTE", sea_query::PgInterval::DayToMinute as u8)?;
        m.add("INTERVAL_DAY_TO_SECOND", sea_query::PgInterval::DayToSecond as u8)?;
        m.add("INTERVAL_HOUR_TO_MINUTE", sea_query::PgInterval::HourToMinute as u8)?;
        m.add("INTERVAL_HOUR_TO_SECOND", sea_query::PgInterval::HourToSecond as u8)?;
        m.add("INTERVAL_MINUTE_TO_SECOND", sea_query::PgInterval::MinuteToSecond as u8)?;

        super::typeref::initialize_typeref(m.py());

        Ok(())
    }
}
