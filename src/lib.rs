#![allow(unused_unsafe)]
#![allow(clippy::macro_metavars_in_unsafe)]
#![warn(clippy::print_stdout)]
#![warn(clippy::print_stderr)]
#![warn(clippy::dbg_macro)]
#![feature(likely_unlikely)]
#![feature(optimize_attribute)]

/// Helper macros and some utilitize functions
#[macro_use]
mod macros;

mod parameters;

mod adaptation;
mod column;
mod common;
mod expression;
mod typeref;

/// RapidQuery core module written in Rust
#[pyo3::pymodule(gil_used = false)]
mod _lib {
    use pyo3::types::PyModuleMethods;

    #[pymodule_export]
    use super::column::types::PyColumnTypeMeta;

    #[pymodule_export]
    use super::column::types::{
        PyArrayType, PyBigIntegerType, PyBigUnsignedType, PyBinaryType, PyBitType, PyBlobType,
        PyBooleanType, PyCharType, PyCidrType, PyDateTimeType, PyDateType, PyDecimalType,
        PyDoubleType, PyEnumType, PyFloatType, PyInetType, PyIntegerType, PyIntervalType,
        PyJsonBinaryType, PyJsonType, PyLTreeType, PyMacAddressType, PyMoneyType,
        PySmallIntegerType, PySmallUnsignedType, PyStringType, PyTextType, PyTimeType,
        PyTimestampType, PyTimestampWithTimeZoneType, PyTinyIntegerType, PyTinyUnsignedType,
        PyUnsignedType, PyUuidType, PyVarBinaryType, PyVarBitType, PyVectorType, PyYearType,
    };

    #[pymodule_export]
    use super::adaptation::PyAdaptedValue;

    #[pymodule_export]
    use super::common::{PyAsteriskType, PyColumnRef};

    #[pymodule_export]
    use super::expression::{all, any, PyExpr, PyFunctionCall};

    #[pymodule_export]
    use super::column::PyColumn;

    #[pymodule_init]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        m.add("INTERVAL_YEAR", sea_query::PgInterval::Year as u8)?;
        m.add("INTERVAL_MONTH", sea_query::PgInterval::Month as u8)?;
        m.add("INTERVAL_DAY", sea_query::PgInterval::Day as u8)?;
        m.add("INTERVAL_HOUR", sea_query::PgInterval::Hour as u8)?;
        m.add("INTERVAL_MINUTE", sea_query::PgInterval::Minute as u8)?;
        m.add("INTERVAL_SECOND", sea_query::PgInterval::Second as u8)?;
        m.add(
            "INTERVAL_YEAR_TO_MONTH",
            sea_query::PgInterval::YearToMonth as u8,
        )?;
        m.add(
            "INTERVAL_DAY_TO_HOUR",
            sea_query::PgInterval::DayToHour as u8,
        )?;
        m.add(
            "INTERVAL_DAY_TO_MINUTE",
            sea_query::PgInterval::DayToMinute as u8,
        )?;
        m.add(
            "INTERVAL_DAY_TO_SECOND",
            sea_query::PgInterval::DayToSecond as u8,
        )?;
        m.add(
            "INTERVAL_HOUR_TO_MINUTE",
            sea_query::PgInterval::HourToMinute as u8,
        )?;
        m.add(
            "INTERVAL_HOUR_TO_SECOND",
            sea_query::PgInterval::HourToSecond as u8,
        )?;
        m.add(
            "INTERVAL_MINUTE_TO_SECOND",
            sea_query::PgInterval::MinuteToSecond as u8,
        )?;

        m.add("ASTERISK", PyAsteriskType {})?;

        super::typeref::initialize_typeref(m.py());

        Ok(())
    }
}
