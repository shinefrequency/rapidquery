from typing_extensions import Self
import datetime
import decimal
import typing
import uuid

class _AsteriskType:
    """
    Asterisk `"*"` - very useful for expression creating
    """

    ...

ASTERISK: typing.Final[_AsteriskType]

T = typing.TypeVar("T")

class ColumnTypeMeta(typing.Generic[T]):
    """
    Base class for all SQL column data types.

    This abstract base class represents SQL data types that can be used in
    column definitions. Each subclass implements a specific SQL data type
    with its particular characteristics, constraints, and backend-specific
    representations.
    """
    def __new__(cls) -> Self: ...
    def __eq__(self, other) -> bool: ...
    def __ne__(self, other) -> bool: ...
    def __repr__(self) -> str: ...

class _LengthColumnType(ColumnTypeMeta[T]):
    """
    Base class for column types that have a length parameter.

    This is an internal base class for column types like CHAR, VARCHAR,
    BINARY, and VARBINARY that specify a maximum length constraint.
    """

    length: typing.Optional[int]
    """The maximum length constraint for this column type."""

    def __new__(cls, length: typing.Optional[int] = ...) -> Self: ...

class _PrecisionScaleColumnType(ColumnTypeMeta[T]):
    """
    Base class for numeric column types with precision and scale parameters.

    This is an internal base class for numeric types like DECIMAL and NUMERIC
    that require both precision (total digits) and scale (decimal places) specification.
    """
    def __new__(cls, precision_scale: typing.Optional[typing.Tuple[int, int]] = ...) -> Self: ...
    @property
    def precision_scale(self) -> typing.Optional[typing.Tuple[int, int]]:
        """The total number of significant digits."""
        ...

class CharType(_LengthColumnType[str]):
    """
    Fixed-length character string column type (CHAR).

    Represents a fixed-length character string. Values shorter than the
    specified length are padded with spaces. Suitable for storing data
    with consistent, known lengths like country codes or status flags.
    """

    ...

class StringType(_LengthColumnType[str]):
    """
    Variable-length character string column type (VARCHAR).

    Represents a variable-length character string with a maximum length limit.
    This is the most common string type for storing text data of varying lengths
    like names, descriptions, or user input.
    """

    ...

class TextType(ColumnTypeMeta[str]):
    """
    Large text column type (TEXT).

    Represents a large text field capable of storing long strings without
    a predefined length limit. Suitable for storing articles, comments,
    descriptions, or any text content that may be very long.
    """

    ...

class TinyIntegerType(ColumnTypeMeta[int]):
    """
    Very small integer column type (TINYINT).

    Typically stores integers in the range -128 to 127 (signed) or 0 to 255
    (unsigned). Useful for flags, small counters, or enumerated values.
    """

    ...

class SmallIntegerType(ColumnTypeMeta[int]):
    """
    Small integer column type (SMALLINT).

    Typically stores integers in the range -32,768 to 32,767 (signed) or
    0 to 65,535 (unsigned). Good for moderate-sized counters or numeric codes.
    """

    ...

class IntegerType(ColumnTypeMeta[int]):
    """
    Standard integer column type (INTEGER/INT).

    The most common integer type, typically storing 32-bit integers in the
    range -2,147,483,648 to 2,147,483,647 (signed). Suitable for most
    numeric data including IDs, quantities, and counters.
    """

    ...

class BigIntegerType(ColumnTypeMeta[int]):
    """
    Large integer column type (BIGINT).

    Stores 64-bit integers for very large numeric values. Essential for
    high-volume systems, timestamps, large counters, or when integer
    overflow is a concern.
    """

    ...

class TinyUnsignedType(ColumnTypeMeta[int]):
    """
    Unsigned tiny integer column type.

    Stores small positive integers only, typically 0 to 255. Useful for
    small counters, percentages, or enumerated values that are always positive.
    """

    ...

class SmallUnsignedType(ColumnTypeMeta[int]):
    """
    Unsigned small integer column type.

    Stores moderate positive integers only, typically 0 to 65,535. Good for
    larger counters or numeric codes that are always positive.
    """

    ...

class UnsignedType(ColumnTypeMeta[int]):
    """
    Unsigned integer column type.

    Stores positive integers only, typically 0 to 4,294,967,295. Doubles the
    positive range compared to signed integers, useful for IDs and counters
    that will never be negative.
    """

    ...

class BigUnsignedType(ColumnTypeMeta[int]):
    """
    Unsigned big integer column type.

    Stores very large positive integers only. Provides the maximum positive
    integer range for high-volume systems or when very large positive
    values are required.
    """

    ...

class FloatType(ColumnTypeMeta[float]):
    """
    Single-precision floating point column type (FLOAT).

    Stores approximate numeric values with single precision. Suitable for
    scientific calculations, measurements, or any numeric data where some
    precision loss is acceptable in exchange for storage efficiency.
    """

    ...

class DoubleType(ColumnTypeMeta[float]):
    """
    Double-precision floating point column type (DOUBLE).

    Stores approximate numeric values with double precision. Provides higher
    precision than FLOAT for scientific calculations or when more accuracy
    is required in floating-point operations.
    """

    ...

class DecimalType(_PrecisionScaleColumnType[decimal.Decimal]):
    """
    Exact numeric decimal column type (DECIMAL/NUMERIC).

    Stores exact numeric values with fixed precision and scale. Essential for
    financial calculations, currency values, or any situation where exact
    decimal representation is required without floating-point approximation.
    """

    ...

class DateTimeType(ColumnTypeMeta[datetime.datetime]):
    """
    Date and time column type (DATETIME).

    Stores both date and time information without timezone awareness.
    Suitable for recording timestamps, event times, or scheduling information
    when timezone handling is managed at the application level.
    """

    ...

class TimestampType(ColumnTypeMeta[datetime.datetime]):
    """
    Timestamp column type (TIMESTAMP).

    Stores timestamp values, often with automatic update capabilities.
    Behavior varies by database system - may include timezone handling
    or automatic updates on record changes.
    """

    ...

class TimestampWithTimeZoneType(ColumnTypeMeta[datetime.datetime]):
    """
    Timestamp with timezone column type (TIMESTAMPTZ).

    Stores timestamp values with timezone information. Essential for
    applications that need to handle dates and times across different
    timezones accurately.
    """

    ...

class TimeType(ColumnTypeMeta[datetime.time]):
    """
    Time-only column type (TIME).

    Stores time information without date component. Useful for storing
    daily schedules, opening hours, or any time-based data that repeats
    daily regardless of the specific date.
    """

    ...

class DateType(ColumnTypeMeta[datetime.date]):
    """
    Date-only column type (DATE).

    Stores date information without time component. Ideal for birth dates,
    deadlines, or any date-based data where time precision is not needed.
    """

    ...

class YearType(ColumnTypeMeta[int]):
    """
    Year-only column type (YEAR).

    Stores year values efficiently. Useful for storing birth years,
    academic years, or any year-based categorical data where full
    date precision is unnecessary.
    """

    ...

class BlobType(ColumnTypeMeta[bytes]):
    """
    Binary large object column type (BLOB).

    Stores large binary data such as images, documents, audio files, or
    any binary content. Size limits vary by database system.
    """

    ...

class BinaryType(_LengthColumnType[bytes]):
    """
    Fixed-length binary data column type (BINARY).

    Stores binary data of a fixed length. Values shorter than the specified
    length are padded. Useful for storing hashes, keys, or other binary
    data with consistent length.
    """

    ...

class VarBinaryType(_LengthColumnType[bytes]):
    """
    Variable-length binary data column type (VARBINARY).

    Stores binary data of variable length up to a specified maximum.
    More storage-efficient than BINARY for binary data of varying lengths.
    """

    ...

class BitType(_LengthColumnType[bytes]):
    """
    Fixed-length bit string column type (BIT).

    Stores a fixed number of bits. Useful for storing boolean flags efficiently
    or binary data where individual bits have meaning.
    """

    ...

class VarBitType(_LengthColumnType[bytes]):
    """
    Variable-length bit string column type (VARBIT).

    Stores a variable number of bits up to a specified maximum. More flexible
    than fixed BIT type for bit strings of varying lengths.
    """

    ...

class BooleanType(ColumnTypeMeta[bool]):
    """
    Boolean column type (BOOLEAN).

    Stores true/false values. The standard way to store boolean data,
    though implementation varies by database (some use TINYINT(1) or
    similar representations).
    """

    ...

class MoneyType(_PrecisionScaleColumnType[decimal.Decimal]):
    """
    Money/currency column type (MONEY).

    Specialized numeric type for storing monetary values with fixed precision.
    Optimized for currency calculations and formatting, though DECIMAL is
    often preferred for financial applications.
    """

    ...

class JsonType(ColumnTypeMeta[typing.Any]):
    """
    JSON data column type (JSON).

    Stores JSON documents with validation and indexing capabilities.
    Allows for flexible schema design and complex nested data structures
    while maintaining some query capabilities.
    """

    ...

class JsonBinaryType(ColumnTypeMeta[typing.Any]):
    """
    Binary JSON column type (JSONB).

    Stores JSON documents in a binary format for improved performance.
    Provides faster query and manipulation operations compared to text-based
    JSON storage, with additional indexing capabilities.
    """

    ...

class UuidType(ColumnTypeMeta[uuid.UUID]):
    """
    UUID column type (UUID).

    Stores universally unique identifiers. Ideal for distributed systems,
    primary keys, or any situation where globally unique identifiers are
    needed without central coordination.
    """

    ...

class VectorType(_LengthColumnType[list]):
    """
    Vector column type for storing mathematical vectors.

    Specialized type for storing vector data, often used in machine learning,
    similarity search, or mathematical applications. The length parameter
    typically specifies the vector dimension.
    """

    ...

class CidrType(ColumnTypeMeta[str]):
    """
    CIDR network address column type (CIDR).

    Stores IPv4 or IPv6 network addresses in CIDR notation (e.g., 192.168.1.0/24).
    Useful for network configuration, IP address management, and routing tables.
    """

    ...

class InetType(ColumnTypeMeta[str]):
    """
    Internet address column type (INET).

    Stores IPv4 or IPv6 addresses, with or without subnet specification.
    More flexible than CIDR type, allowing both host addresses and network ranges.
    """

    ...

class MacAddressType(ColumnTypeMeta[str]):
    """
    MAC address column type (MACADDR).

    Stores MAC (Media Access Control) addresses for network devices.
    Provides validation and formatting for 6-byte MAC addresses.
    """

    ...

class LTreeType(ColumnTypeMeta[str]):
    """
    Label tree column type (LTREE).

    Stores hierarchical tree-like structures as paths. Useful for representing
    organizational hierarchies, category trees, or any nested data structure
    that needs efficient tree operations.
    """

    ...

INTERVAL_YEAR: typing.Final[int]
INTERVAL_MONTH: typing.Final[int]
INTERVAL_DAY: typing.Final[int]
INTERVAL_HOUR: typing.Final[int]
INTERVAL_MINUTE: typing.Final[int]
INTERVAL_SECOND: typing.Final[int]
INTERVAL_YEAR_TO_MONTH: typing.Final[int]
INTERVAL_DAY_TO_HOUR: typing.Final[int]
INTERVAL_DAY_TO_MINUTE: typing.Final[int]
INTERVAL_DAY_TO_SECOND: typing.Final[int]
INTERVAL_HOUR_TO_MINUTE: typing.Final[int]
INTERVAL_HOUR_TO_SECOND: typing.Final[int]
INTERVAL_MINUTE_TO_SECOND: typing.Final[int]

class IntervalType(ColumnTypeMeta[datetime.timedelta]):
    """
    Time interval column type (INTERVAL).

    Stores time intervals (durations) with configurable precision and field
    restrictions. Can store periods like "3 days", "2 hours 30 minutes", etc.

    The fields parameter constrains which time units are stored (using
    PGINTERVAL_* constants), and precision controls fractional seconds.
    """

    fields: typing.Optional[int]
    """Bitmask specifying which interval fields to include (using PGINTERVAL_* constants)."""

    precision: typing.Optional[int]
    """Number of fractional digits for seconds."""

    def __new__(
        cls, fields: typing.Optional[int] = ..., precision: typing.Optional[int] = ...
    ) -> Self: ...

class EnumType(ColumnTypeMeta[str]):
    """
    Enumeration column type (ENUM).

    Stores one value from a predefined set of allowed string values.
    Provides type safety and storage efficiency for categorical data
    with a fixed set of possible values.

    Examples:

        Enum("status", ["active", "inactive", "pending"])
        Enum("priority", ["low", "medium", "high", "critical"])
    """

    name: str
    """The name of the enumeration type."""

    variants: typing.Sequence[str]
    """The allowed values for this enumeration."""

    def __new__(cls, name: str, variants: typing.Sequence[str]) -> Self: ...

class ArrayType(ColumnTypeMeta[list]):
    """
    Array column type for storing arrays of elements.

    Represents a column that stores arrays of a specified element type.
    Useful in databases that support native array types (like PostgreSQL)
    for storing lists of values in a single column.
    """

    element: ColumnTypeMeta
    """The type of elements stored in the array."""

    def __new__(cls, element: ColumnTypeMeta) -> Self: ...

class AdaptedValue:
    """
    Bridges Python types, Rust types, and SQL types for seamless data conversion.

    This class handles validation, adaptation, and conversion between different
    type systems used in the application stack.

    NOTE: this class is immutable and frozen.
    """

    @typing.overload
    def __new__(cls, val: typing.Any, type: typing.Optional[ColumnTypeMeta] = None) -> Self:
        """
        Validates and adapts your value for Rust and SQL, then creates a new `AdaptedValue` instance.

        This method automatically detects the type of your value and selects appropriate Rust and SQL types.
        For example:
        - Python `int` becomes `BIGINT` SQL type (`BigIntegerType`)
        - Python `dict` or `list` becomes `JSON` SQL type (`JsonType`)
        - Python `float` becomes `DOUBLE` SQL type (`DoubleType`)

        However, for more accurate type selection, it's recommended to use the `type` parameter.

        Example::

            # Let the system detect types automatically
            AdaptedValue(1)                    # -> INTEGER SQL type
            AdaptedValue(1.4)                  # -> DOUBLE SQL type
            AdaptedValue("127.0.0.1")          # -> VARCHAR SQL type
            AdaptedValue({"key": "value"})     # -> JSON SQL type

            # Explicitly specify the type
            AdaptedValue(1, TinyUnsigned())      # -> TINYINT UNSIGNED SQL type
            AdaptedValue(1.4, Float())           # -> FLOAT SQL type
            AdaptedValue("127.0.0.1", Inet())    # -> INET SQL type (network address)
            AdaptedValue([4.3, 5.6], Vector())   # -> VECTOR SQL type (for AI embeddings)

        NOTE: this class is immutable and frozen.
        """
        ...

    @property
    def is_null(self) -> bool:
        """Returns True if the adapted value is NULL."""
        ...
    @property
    def is_integer(self) -> bool:
        """Returns True if the adapted value is an integer type."""
        ...
    @property
    def is_float(self) -> bool:
        """Returns True if the adapted value is a floating-point type."""
        ...
    @property
    def is_boolean(self) -> bool:
        """Returns True if the adapted value is a boolean type."""
        ...
    @property
    def is_string(self) -> bool:
        """Returns True if the adapted value is a string type."""
        ...
    @property
    def is_date(self) -> bool:
        """Returns True if the adapted value is a date type."""
        ...
    @property
    def is_datetime(self) -> bool:
        """Returns True if the adapted value is a datetime type."""
        ...
    @property
    def is_time(self) -> bool:
        """Returns True if the adapted value is a time type."""
        ...
    @property
    def is_uuid(self) -> bool:
        """Returns True if the adapted value is a UUID type."""
        ...
    @property
    def is_bytes(self) -> bool:
        """Returns True if the adapted value is a bytes/binary type."""
        ...
    @property
    def is_json(self) -> bool:
        """Returns True if the adapted value is a JSON type."""
        ...
    @property
    def is_decimal(self) -> bool:
        """Returns True if the adapted value is a decimal type."""
        ...
    @property
    def is_array(self) -> bool:
        """Returns True if the adapted value is an array type."""
        ...
    @property
    def is_vector(self) -> bool:
        """Returns True if the adapted value is a vector type."""
        ...

    @property
    def value(self) -> typing.Any:
        """
        Converts the adapted value back to a Python type.
        """
        ...

    def to_sql(self) -> str:
        """
        Converts the adapted value to its SQL string representation.
        """
        ...

    def __repr__(self) -> str: ...

class ColumnRef:
    name: str
    table: typing.Optional[str]
    schema: typing.Optional[str]

    def __init__(
        self,
        name: str,
        table: typing.Optional[str] = ...,
        schema: typing.Optional[str] = ...,
    ) -> None: ...
    @classmethod
    def parse(cls, string: str) -> "ColumnRef": ...
    def __eq__(self, other: "ColumnRef") -> bool: ...
    def __ne__(self, other: "ColumnRef") -> bool: ...
    def __repr__(self) -> str: ...

class Expr:
    """
    Represents a Simple Expression in SQL.
    """

    def __new__(
        cls,
        value: typing.Union[
            Self,
            AdaptedValue,
            ColumnRef,
            tuple,
            _AsteriskType,
            typing.Any,
        ],
        /,
    ) -> Self: ...
    @classmethod
    def val(cls, value: AdaptedValue) -> Self:
        """
        Express a `AdaptedValue`, returning a `Expr`.
        """
        ...

    @classmethod
    def func(cls, value: FunctionCall) -> Self:
        """
        Express a `FunctionCall`, returning a `Expr`.
        """
        ...

    @classmethod
    def col(cls, name: typing.Union[str, ColumnRef]) -> Self:
        """
        Express the target column without table prefix, returning a `Expr`.
        """
        ...

    @classmethod
    def tuple(
        cls, values: typing.Union[typing.Set[Self], typing.List[Self], typing.Tuple[Self]]
    ) -> Self:
        """
        Wraps tuple of `Expr`, can be used for tuple comparison
        """
        ...

    @classmethod
    def asterisk(cls) -> Self:
        """
        Shorthand for `Expr.col("*")`
        """
        ...

    @classmethod
    def custom(cls, value: str) -> Self:
        """
        Express any custom expression in `str`.
        """
        ...

    @classmethod
    def current_date(cls) -> Self:
        """
        Keyword `CURRENT_DATE`.
        """
        ...

    @classmethod
    def current_time(cls) -> Self:
        """
        Keyword `CURRENT_TIME`.
        """
        ...

    @classmethod
    def current_timestamp(cls) -> Self:
        """
        Keyword `CURRENT_TIMESTAMP`.
        """
        ...

    @classmethod
    def null(cls) -> Self:
        """
        Keyword `NULL`.
        """
        ...

    def cast_as(self, value: str) -> Self:
        """
        Express a `CAST AS` expression.
        """
        ...

    def like(self, pattern: str, escape: typing.Optional[str] = ...) -> Self:
        """
        Express a `LIKE` expression.
        """
        ...

    def not_like(self, pattern: str, escape: typing.Optional[str] = ...) -> Self:
        """
        Express a `NOT LIKE` expression.
        """
        ...

    def __eq__(self, other: Self) -> Self: ...
    def __ne__(self, other: Self) -> Self: ...
    def __gt__(self, other: Self) -> Self: ...
    def __ge__(self, other: Self) -> Self: ...
    def __lt__(self, other: Self) -> Self: ...
    def __le__(self, other: Self) -> Self: ...
    def __add__(self, other: Self) -> Self: ...
    def __sub__(self, other: Self) -> Self: ...
    def __and__(self, other: Self) -> Self: ...
    def __or__(self, other: Self) -> Self: ...
    def __truediv__(self, other: Self) -> Self: ...
    def is_(self, other: Self) -> Self:
        """
        Express a `IS` expression.
        """
        ...

    def sqlite_matches(self, other: Self) -> Self:
        """
        Express an sqlite `MATCH` operator.
        """
        ...

    def sqlite_glob(self, other: Self) -> Self:
        """
        Express an sqlite `GLOB` operator.
        """
        ...

    def pg_concat(self, other: Self) -> Self:
        """
        Express an postgres concatenate (`||`) expression.
        """
        ...

    def pg_contained(self, other: Self) -> Self:
        """
        Express an postgres fulltext search contained (`<@`) expression.
        """
        ...

    def cast_json_field(self, other: Self) -> Self:
        """
        Express a postgres/sqlite retrieves JSON field and casts it to an appropriate SQL type (`->>`).
        """
        ...

    def get_json_field(self, other: Self) -> Self:
        """
        Express a postgres/sqlite retrieves JSON field and casts it to an appropriate SQL type (`->`).
        """
        ...

    def pg_contains(self, other: Self) -> Self:
        """
        Express an postgres fulltext search contains (`@>`) expression.
        """
        ...

    def pg_matches(self, other: Self) -> Self:
        """
        Express an postgres fulltext search matches (`@@`) expression.
        """
        ...

    def pg_ilike(self, other: Self) -> Self:
        """
        Express an postgres `ILIKE` expression.
        """
        ...

    def pg_not_ilike(self, other: Self) -> Self:
        """
        Express an postgres `NOT ILIKE` expression.
        """
        ...

    def is_not(self, other: Self) -> Self:
        """
        Express a `IS NOT` expression.
        """
        ...
    def is_null(self) -> Self:
        """
        Express a `IS NULL` expression.
        """
        ...
    def is_not_null(self) -> Self:
        """
        Express a `IS NOT NULL` expression.
        """
        ...
    def __lshift__(self, other: Self) -> Self: ...
    def __rshift__(self, other: Self) -> Self: ...
    def __mod__(self, other: Self) -> Self: ...
    def __mul__(self, other: Self) -> Self: ...
    def between(self, a: Self, b: Self) -> Self:
        """
        Express a `BETWEEN` expression.
        """
        ...

    def not_between(self, a: Self, b: Self) -> Self:
        """
        Express a `NOT BETWEEN` expression.
        """
        ...

    def in_(self, expr: typing.Sequence[Self]) -> Self:
        """
        Express a `IN` expression.
        """
        ...

    def not_in(self, expr: typing.Sequence[Self]) -> Self:
        """
        Express a `NOT IN` expression.
        """
        ...

    def to_sql(self) -> str:
        """
        Converts the expression to its SQL string representation.
        """
        ...

    def __repr__(self) -> str: ...

class FunctionCall:
    def __new__(cls, name: str) -> Self: ...
    def arg(self, arg: Expr) -> Self: ...
    @classmethod
    def min(cls, expr: Expr) -> Self: ...
    @classmethod
    def max(cls, expr: Expr) -> Self: ...
    @classmethod
    def abs(cls, expr: Expr) -> Self: ...
    @classmethod
    def avg(cls, expr: Expr) -> Self: ...
    @classmethod
    def count(cls, expr: Expr) -> Self: ...
    @classmethod
    def count_distinct(cls, expr: Expr) -> Self: ...
    @classmethod
    def if_null(cls, expr: Expr) -> Self: ...
    @classmethod
    def greatest(cls, exprs: typing.Sequence[Expr]) -> Self: ...
    @classmethod
    def least(cls, exprs: typing.Sequence[Expr]) -> Self: ...
    @classmethod
    def char_length(cls, expr: Expr) -> Self: ...
    @classmethod
    def coalesce(cls, exprs: typing.Sequence[Expr]) -> Self: ...
    @classmethod
    def lower(cls, expr: Expr) -> Self: ...
    @classmethod
    def upper(cls, expr: Expr) -> Self: ...
    @classmethod
    def bit_and(cls, expr: Expr) -> Self: ...
    @classmethod
    def bit_or(cls, expr: Expr) -> Self: ...
    @classmethod
    def random(cls) -> Self: ...
    @classmethod
    def round(cls, expr: Expr) -> Self: ...
    @classmethod
    def md5(cls, expr: Expr) -> Self: ...
    def to_sql(self) -> str: ...
    def __repr__(self) -> str: ...

def all(arg1: Expr, *args: Expr) -> Expr:
    """
    Create a condition that is false if any of the conditions is false.
    """
    ...

def any(arg1: Expr, *args: Expr) -> Expr:
    """
    Create a condition that is true if any of the conditions is true.
    """
    ...
