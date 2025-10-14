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

class BackendMeta:
    """
    Base class for database backend implementations.

    This abstract base class defines the interface for SQL query generation
    backends that support different database systems (SQLite, MySQL, PostgreSQL).
    Each backend handles the database-specific syntax and features for building
    SQL statements dynamically.

    Subclasses should implement database-specific query generation methods
    and handle dialect differences for their respective database systems.
    """

    ...

class SQLiteBackend(BackendMeta):
    """
    SQLite-specific query builder and schema builder backend.

    Implements SQL generation tailored for SQLite's syntax and capabilities.
    Handles SQLite-specific features like:
    - AUTOINCREMENT for primary keys
    - SQLite data type mapping
    - SQLite-specific index syntax
    - Constraint handling differences

    This backend generates SQL statements that are compatible with SQLite's
    dialect and feature set.
    """

    ...

class MySQLBackend(BackendMeta):
    """
    MySQL-specific query builder and schema builder backend.

    Implements SQL generation tailored for MySQL's syntax and capabilities.
    Handles MySQL-specific features like:
    - AUTO_INCREMENT for primary keys
    - MySQL data type mapping and engine specifications
    - MySQL index types (BTREE, HASH, FULLTEXT)
    - Character set and collation handling
    - MySQL-specific constraint syntax

    This backend generates SQL statements that are compatible with MySQL's
    dialect and feature set.
    """

    ...

class PostgreSQLBackend(BackendMeta):
    """
    PostgreSQL-specific query builder and schema builder backend.

    Implements SQL generation tailored for PostgreSQL's syntax and capabilities.
    Handles PostgreSQL-specific features like:
    - SERIAL/BIGSERIAL for auto-increment columns
    - Advanced PostgreSQL data types (JSONB, UUID, INET, CIDR, etc.)
    - PostgreSQL index types and advanced indexing features
    - Schema-qualified object names
    - PostgreSQL-specific constraint and extension syntax

    This backend generates SQL statements that are compatible with PostgreSQL's
    dialect and feature set.
    """

    ...

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

    def build(self, backend: BackendMeta) -> str:
        """
        Converts the adapted value to its SQL string representation.
        """
        ...

    def __repr__(self) -> str: ...

import typing
from typing import Self

class ColumnRef:
    """
    Represents a reference to a database column with optional table and schema qualification.

    This class is used to uniquely identify columns in SQL queries, supporting
    schema-qualified and table-qualified column references.

    Attributes:
        name: The column name
        table: The table name containing the column, if specified
        schema: The schema name containing the table, if specified

    Example:
        >>> ColumnRef("id")
        >>> ColumnRef("id", table="users")
        >>> ColumnRef("id", table="users", schema="public")
    """

    name: str
    table: typing.Optional[str]
    schema: typing.Optional[str]

    def __new__(
        cls,
        name: str,
        table: typing.Optional[str] = ...,
        schema: typing.Optional[str] = ...,
    ) -> Self:
        """
        Create a new ColumnRef instance.

        Args:
            name: The name of the column
            table: The table name containing the column
            schema: The schema name containing the table

        Returns:
            A new ColumnRef instance
        """
        ...

    @classmethod
    def parse(cls, string: str) -> "ColumnRef":
        """
        Parse a string representation of a column reference.

        Supports formats like:
        - "column_name"
        - "table.column_name"
        - "schema.table.column_name"

        Args:
            string: The string to parse

        Returns:
            A ColumnRef instance representing the parsed reference

        Raises:
            ValueError: If the string format is invalid
        """
        ...

    def __eq__(self, other: "ColumnRef") -> bool:
        """
        Check equality with another ColumnRef.

        Two ColumnRefs are equal if they have the same name, table, and schema.
        """
        ...

    def __ne__(self, other: "ColumnRef") -> bool:
        """
        Check inequality with another ColumnRef.
        """
        ...

    def __repr__(self) -> str:
        """
        Return a string representation of the ColumnRef.

        Returns:
            A string that could be used to recreate this ColumnRef
        """
        ...

_ExprValue = typing.Union[
    Self,
    AdaptedValue,
    ColumnRef,
    tuple,
    _AsteriskType,
    typing.Any,
]

class Expr:
    """
    Represents a SQL expression that can be built into SQL code.

    This class provides a fluent interface for constructing complex SQL expressions
    in a database-agnostic way. It supports arithmetic operations, comparisons,
    logical operations, and database-specific functions.

    The class automatically handles SQL injection protection and proper quoting
    when building the final SQL statement.

    Example::
        # Basic comparison
        e = Expr(1) > Expr(2)
        e.build(MySQLBackend())
        # Result: 1 > 2

        # IN clause with tuple
        e = Expr.col("id").in_(Expr((1, 2, 3)))
        e.build(MySQLBackend())
        # Result: "id" IN (1, 2, 3)

        # Complex expression with functions
        e = Expr.func(FunctionCall.upper(Expr.col("name"))) == "JOHN"
        e.build(PostgreSQLBackend())
        # Result: UPPER("name") = 'JOHN'
    """

    def __new__(cls, value: _ExprValue, /) -> Self:
        """
        Create a new expression from a value.

        Args:
            value: The value to convert to an expression. Can be a primitive
                  value, ColumnRef, AdaptedValue, or another Expr.

        Returns:
            A new Expr instance representing the value
        """
        ...

    @classmethod
    def val(cls, value: AdaptedValue) -> Self:
        """
        Create an expression from an AdaptedValue.

        AdaptedValues are values that have been adapted for safe use in SQL,
        such as properly escaped strings or formatted dates.

        Args:
            value: The adapted value to convert to an expression

        Returns:
            An Expr representing the adapted value
        """
        ...

    @classmethod
    def func(cls, value: FunctionCall) -> Self:
        """
        Create an expression from a FunctionCall.

        Args:
            value: The function call to convert to an expression

        Returns:
            An Expr representing the function call
        """
        ...

    @classmethod
    def col(cls, name: typing.Union[str, ColumnRef]) -> Self:
        """
        Create a column reference expression.

        Args:
            name: The column name or ColumnRef to reference

        Returns:
            An Expr representing the column reference
        """
        ...

    @classmethod
    def tuple(
        cls, values: typing.Union[typing.Set[Self], typing.List[Self], typing.Tuple[Self]]
    ) -> Self:
        """
        Create a tuple expression for tuple comparisons.

        Args:
            values: A collection of expressions to include in the tuple

        Returns:
            An Expr representing a SQL tuple

        Example:
            >>> Expr.tuple([Expr.col("id"), Expr.col("name")])
            # Can be used in: WHERE (id, name) IN ((1, 'a'), (2, 'b'))
        """
        ...

    @classmethod
    def asterisk(cls) -> Self:
        """
        Create a wildcard expression for SELECT * queries.

        Returns:
            An Expr representing the asterisk wildcard
        """
        ...

    @classmethod
    def custom(cls, value: str) -> Self:
        """
        Create an expression from a custom SQL string.

        Warning: This method does not escape the input, so it should only
        be used with trusted strings to avoid SQL injection vulnerabilities.

        Args:
            value: The raw SQL string to use as an expression

        Returns:
            An Expr representing the custom SQL
        """
        ...

    @classmethod
    def current_date(cls) -> Self:
        """
        Create an expression for the CURRENT_DATE SQL function.

        Returns:
            An Expr representing CURRENT_DATE
        """
        ...

    @classmethod
    def current_time(cls) -> Self:
        """
        Create an expression for the CURRENT_TIME SQL function.

        Returns:
            An Expr representing CURRENT_TIME
        """
        ...

    @classmethod
    def current_timestamp(cls) -> Self:
        """
        Create an expression for the CURRENT_TIMESTAMP SQL function.

        Returns:
            An Expr representing CURRENT_TIMESTAMP
        """
        ...

    @classmethod
    def null(cls) -> Self:
        """
        Create an expression representing the NULL value.

        Returns:
            An Expr representing NULL
        """
        ...

    def cast_as(self, value: str) -> Self:
        """
        Create a CAST expression to convert to a specific SQL type.

        Args:
            value: The target SQL type name (e.g., 'INTEGER', 'VARCHAR(255)')

        Returns:
            A new Expr representing the cast operation
        """
        ...

    def like(self, pattern: str, escape: typing.Optional[str] = ...) -> Self:
        """
        Create a LIKE pattern matching expression.

        Args:
            pattern: The pattern to match against
            escape: Optional escape character for wildcards in the pattern

        Returns:
            A new Expr representing the LIKE operation
        """
        ...

    def not_like(self, pattern: str, escape: typing.Optional[str] = ...) -> Self:
        """
        Create a NOT LIKE pattern matching expression.

        Args:
            pattern: The pattern that should not match
            escape: Optional escape character for wildcards in the pattern

        Returns:
            A new Expr representing the NOT LIKE operation
        """
        ...

    def __eq__(self, other: Self) -> Self:
        """
        Create an equality comparison expression.
        """
        ...

    def __ne__(self, other: Self) -> Self:
        """
        Create an inequality comparison expression.
        """
        ...

    def __gt__(self, other: Self) -> Self:
        """
        Create a greater-than comparison expression.
        """
        ...

    def __ge__(self, other: Self) -> Self:
        """
        Create a greater-than-or-equal comparison expression.
        """
        ...

    def __lt__(self, other: Self) -> Self:
        """
        Create a less-than comparison expression.
        """
        ...

    def __le__(self, other: Self) -> Self:
        """
        Create a less-than-or-equal comparison expression.
        """
        ...

    def __add__(self, other: Self) -> Self:
        """
        Create an addition expression.
        """
        ...

    def __sub__(self, other: Self) -> Self:
        """
        Create a subtraction expression.
        """
        ...

    def __and__(self, other: Self) -> Self:
        """
        Create a logical AND expression.
        """
        ...

    def __or__(self, other: Self) -> Self:
        """
        Create a logical OR expression.
        """
        ...

    def __truediv__(self, other: Self) -> Self:
        """
        Create a division expression.
        """
        ...

    def is_(self, other: Self) -> Self:
        """
        Create an IS comparison expression (for NULL comparisons).

        Typically used with NULL: column.is_(Expr.null())

        Args:
            other: The expression to compare with

        Returns:
            A new Expr representing the IS comparison
        """
        ...

    def sqlite_matches(self, other: Self) -> Self:
        """
        Create a SQLite MATCH expression for full-text search.

        Args:
            other: The expression to match against

        Returns:
            A new Expr representing the MATCH operation
        """
        ...

    def sqlite_glob(self, other: Self) -> Self:
        """
        Create a SQLite GLOB expression for pattern matching.

        Args:
            other: The glob pattern to match against

        Returns:
            A new Expr representing the GLOB operation
        """
        ...

    def pg_concat(self, other: Self) -> Self:
        """
        Create a PostgreSQL concatenation expression using || operator.

        Args:
            other: The expression to concatenate with

        Returns:
            A new Expr representing the concatenation
        """
        ...

    def pg_contained(self, other: Self) -> Self:
        """
        Create a PostgreSQL contained expression using <@ operator.

        Used for array and range containment checks.

        Args:
            other: The expression to check containment against

        Returns:
            A new Expr representing the contained operation
        """
        ...

    def cast_json_field(self, other: Self) -> Self:
        """
        Extract and cast a JSON field to appropriate SQL type using ->> operator.

        This operator returns the JSON field as text and can be cast to other types.

        Args:
            other: The JSON field path/name to extract

        Returns:
            A new Expr representing the JSON field extraction and casting
        """
        ...

    def get_json_field(self, other: Self) -> Self:
        """
        Extract a JSON field using -> operator (returns JSON type).

        Args:
            other: The JSON field path/name to extract

        Returns:
            A new Expr representing the JSON field extraction
        """
        ...

    def pg_contains(self, other: Self) -> Self:
        """
        Create a PostgreSQL contains expression using @> operator.

        Used for array and range containment checks.

        Args:
            other: The expression to check if it is contained

        Returns:
            A new Expr representing the contains operation
        """
        ...

    def pg_matches(self, other: Self) -> Self:
        """
        Create a PostgreSQL full-text search matches expression using @@ operator.

        Args:
            other: The full-text search query

        Returns:
            A new Expr representing the full-text match operation
        """
        ...

    def pg_ilike(self, other: Self) -> Self:
        """
        Create a PostgreSQL case-insensitive LIKE expression.

        Args:
            other: The pattern to match against

        Returns:
            A new Expr representing the ILIKE operation
        """
        ...

    def pg_not_ilike(self, other: Self) -> Self:
        """
        Create a PostgreSQL case-insensitive NOT LIKE expression.

        Args:
            other: The pattern that should not match

        Returns:
            A new Expr representing the NOT ILIKE operation
        """
        ...

    def is_not(self, other: Self) -> Self:
        """
        Create an IS NOT comparison expression.

        Args:
            other: The expression to compare with

        Returns:
            A new Expr representing the IS NOT comparison
        """
        ...

    def is_null(self) -> Self:
        """
        Create an IS NULL expression.

        Returns:
            A new Expr representing the IS NULL check
        """
        ...

    def is_not_null(self) -> Self:
        """
        Create an IS NOT NULL expression.

        Returns:
            A new Expr representing the IS NOT NULL check
        """
        ...

    def __lshift__(self, other: Self) -> Self:
        """
        Create a bitwise left shift expression.
        """
        ...

    def __rshift__(self, other: Self) -> Self:
        """
        Create a bitwise right shift expression.
        """
        ...

    def __mod__(self, other: Self) -> Self:
        """
        Create a modulo expression.
        """
        ...

    def __mul__(self, other: Self) -> Self:
        """
        Create a multiplication expression.
        """
        ...

    def between(self, a: Self, b: Self) -> Self:
        """
        Create a BETWEEN range comparison expression.

        Args:
            a: The lower bound of the range
            b: The upper bound of the range

        Returns:
            A new Expr representing the BETWEEN operation
        """
        ...

    def not_between(self, a: Self, b: Self) -> Self:
        """
        Create a NOT BETWEEN range comparison expression.

        Args:
            a: The lower bound of the range
            b: The upper bound of the range

        Returns:
            A new Expr representing the NOT BETWEEN operation
        """
        ...

    def in_(self, other: typing.Sequence[Self]) -> Self:
        """
        Create an IN membership expression.

        Args:
            other: A sequence of expressions to check membership against

        Returns:
            A new Expr representing the IN operation
        """
        ...

    def not_in(self, other: typing.Sequence[Self]) -> Self:
        """
        Create a NOT IN membership expression.

        Args:
            other: A sequence of expressions to check non-membership against

        Returns:
            A new Expr representing the NOT IN operation
        """
        ...

    def build(self, backend: BackendMeta) -> str:
        """
        Convert the expression to its SQL string representation.

        Args:
            backend: The database backend that determines SQL dialect and formatting

        Returns:
            A SQL string representation of the expression
        """
        ...

    def __repr__(self) -> str:
        """
        Return a developer-friendly string representation.

        Returns:
            A string that could be used to recreate this expression
        """
        ...

class FunctionCall:
    """
    Represents a SQL function call that can be used in expressions.

    This class provides a type-safe way to construct SQL function calls
    with proper argument handling and database dialect support.
    """

    def __new__(cls, name: str) -> Self:
        """
        Create a new function call with the given name.

        Args:
            name: The name of the SQL function

        Returns:
            A new FunctionCall instance
        """
        ...

    def arg(self, arg: Expr) -> Self:
        """
        Add an argument to the function call.

        Args:
            arg: The expression to add as an argument

        Returns:
            Self for method chaining
        """
        ...

    @classmethod
    def min(cls, expr: Expr) -> Self:
        """
        Create a MIN aggregate function call.

        Args:
            expr: The expression to find the minimum of

        Returns:
            A FunctionCall representing MIN(expr)
        """
        ...

    @classmethod
    def max(cls, expr: Expr) -> Self:
        """
        Create a MAX aggregate function call.

        Args:
            expr: The expression to find the maximum of

        Returns:
            A FunctionCall representing MAX(expr)
        """
        ...

    @classmethod
    def abs(cls, expr: Expr) -> Self:
        """
        Create an ABS absolute value function call.

        Args:
            expr: The expression to get the absolute value of

        Returns:
            A FunctionCall representing ABS(expr)
        """
        ...

    @classmethod
    def avg(cls, expr: Expr) -> Self:
        """
        Create an AVG average function call.

        Args:
            expr: The expression to calculate the average of

        Returns:
            A FunctionCall representing AVG(expr)
        """
        ...

    @classmethod
    def count(cls, expr: Expr) -> Self:
        """
        Create a COUNT aggregate function call.

        Args:
            expr: The expression to count

        Returns:
            A FunctionCall representing COUNT(expr)
        """
        ...

    @classmethod
    def count_distinct(cls, expr: Expr) -> Self:
        """
        Create a COUNT(DISTINCT ...) aggregate function call.

        Args:
            expr: The expression to count distinct values of

        Returns:
            A FunctionCall representing COUNT(DISTINCT expr)
        """
        ...

    @classmethod
    def if_null(cls, expr: Expr) -> Self:
        """
        Create an IFNULL/COALESCE function call (database-dependent).

        Args:
            expr: The expression to check for NULL

        Returns:
            A FunctionCall representing the NULL-checking function
        """
        ...

    @classmethod
    def greatest(cls, exprs: typing.Sequence[Expr]) -> Self:
        """
        Create a GREATEST function call returning the largest value.

        Args:
            exprs: Sequence of expressions to compare

        Returns:
            A FunctionCall representing GREATEST(expr1, expr2, ...)
        """
        ...

    @classmethod
    def least(cls, exprs: typing.Sequence[Expr]) -> Self:
        """
        Create a LEAST function call returning the smallest value.

        Args:
            exprs: Sequence of expressions to compare

        Returns:
            A FunctionCall representing LEAST(expr1, expr2, ...)
        """
        ...

    @classmethod
    def char_length(cls, expr: Expr) -> Self:
        """
        Create a CHAR_LENGTH/LENGTH function call.

        Args:
            expr: The string expression to measure

        Returns:
            A FunctionCall representing CHAR_LENGTH(expr)
        """
        ...

    @classmethod
    def coalesce(cls, exprs: typing.Sequence[Expr]) -> Self:
        """
        Create a COALESCE function call returning first non-NULL value.

        Args:
            exprs: Sequence of expressions to check

        Returns:
            A FunctionCall representing COALESCE(expr1, expr2, ...)
        """
        ...

    @classmethod
    def lower(cls, expr: Expr) -> Self:
        """
        Create a LOWER case conversion function call.

        Args:
            expr: The string expression to convert to lowercase

        Returns:
            A FunctionCall representing LOWER(expr)
        """
        ...

    @classmethod
    def upper(cls, expr: Expr) -> Self:
        """
        Create an UPPER case conversion function call.

        Args:
            expr: The string expression to convert to uppercase

        Returns:
            A FunctionCall representing UPPER(expr)
        """
        ...

    @classmethod
    def bit_and(cls, expr: Expr) -> Self:
        """
        Create a BIT_AND aggregate function call.

        Args:
            expr: The expression for bitwise AND operation

        Returns:
            A FunctionCall representing BIT_AND(expr)
        """
        ...

    @classmethod
    def bit_or(cls, expr: Expr) -> Self:
        """
        Create a BIT_OR aggregate function call.

        Args:
            expr: The expression for bitwise OR operation

        Returns:
            A FunctionCall representing BIT_OR(expr)
        """
        ...

    @classmethod
    def random(cls) -> Self:
        """
        Create a RANDOM/RAND function call.

        Returns:
            A FunctionCall representing the random number function
        """
        ...

    @classmethod
    def round(cls, expr: Expr) -> Self:
        """
        Create a ROUND function call.

        Args:
            expr: The numeric expression to round

        Returns:
            A FunctionCall representing ROUND(expr)
        """
        ...

    @classmethod
    def md5(cls, expr: Expr) -> Self:
        """
        Create an MD5 hash function call.

        Args:
            expr: The expression to hash

        Returns:
            A FunctionCall representing MD5(expr)
        """
        ...

    def build(self, backend: BackendMeta) -> str:
        """
        Convert the function call to its SQL string representation.

        Args:
            backend: The database backend that determines SQL dialect

        Returns:
            A SQL string representation of the function call
        """
        ...

    def __repr__(self) -> str:
        """
        Return a developer-friendly string representation.

        Returns:
            A string that could be used to recreate this function call
        """
        ...

def all(arg1: Expr, *args: Expr) -> Expr:
    """
    Create a logical AND condition that is true only if all conditions are true.

    This is equivalent to SQL's AND operator applied to multiple expressions.

    Args:
        arg1: The first condition
        *args: Additional conditions to combine

    Returns:
        An Expr representing the logical AND of all input expressions

    Example:
        >>> all(Expr.col("age") > 18, Expr.col("status") == "active")
        # Equivalent to: age > 18 AND status = 'active'
    """
    ...

def any(arg1: Expr, *args: Expr) -> Expr:
    """
    Create a logical OR condition that is true if any condition is true.

    This is equivalent to SQL's OR operator applied to multiple expressions.

    Args:
        arg1: The first condition
        *args: Additional conditions to combine

    Returns:
        An Expr representing the logical OR of all input expressions

    Example:
        >>> any(Expr.col("status") == "pending", Expr.col("status") == "approved")
        # Equivalent to: status = 'pending' OR status = 'approved'
    """
    ...

class Column:
    """
    Defines a table column with its properties and constraints.

    Represents a complete column definition including:
    - Column name and data type
    - Constraints (primary key, unique, nullable)
    - Auto-increment behavior
    - Default values and generated columns
    - Comments and extra specifications

    This class is used within TableCreateStatement to specify the structure
    of table columns. It encapsulates all the properties that define how
    a column behaves and what data it can store.

    Example:
        >>> Column("id", Integer, primary_key=True, auto_increment=True)
        >>> Column("name", String(255), nullable=False, default="unknown")
        >>> Column("created_at", Timestamp, default=Expr.current_timestamp())
    """

    name: str
    """The name of the column."""

    type: ColumnTypeMeta
    """The data type of the column."""

    primary_key: bool
    """Whether this column is part of the primary key."""

    nullable: bool
    """Whether this column can contain NULL values."""

    unique: bool
    """Whether this column must contain unique values."""

    auto_increment: bool
    """Whether this column should auto-increment."""

    extra: typing.Optional[str]
    """Extra SQL specifications for this column."""

    default: typing.Optional[Expr]
    """Default value for this column."""

    generated: typing.Optional[Expr]
    """Expression for generated column values."""

    stored_generated: bool
    """Whether the generated column is STORED (vs VIRTUAL)."""

    comment: typing.Optional[str]
    """Comment describing this column."""

    def __new__(
        cls,
        name: str,
        type: ColumnTypeMeta,
        primary_key: bool = ...,
        unique: bool = ...,
        nullable: bool = ...,
        auto_increment: bool = ...,
        extra: typing.Optional[str] = ...,
        comment: typing.Optional[str] = ...,
        default: _ExprValue = ...,
        generated: _ExprValue = ...,
        stored_generated: bool = ...,
    ) -> Self:
        """
        Create a new Column definition.

        Args:
            name: The column name
            type: The column data type
            primary_key: Whether this is a primary key column
            unique: Whether this column has a unique constraint
            nullable: Whether NULL values are allowed
            auto_increment: Whether the column auto-increments
            extra: Additional column specifications
            comment: Column description comment
            default: Default value expression
            generated: Generation expression for computed columns
            stored_generated: Whether computed column is stored physically

        Returns:
            A new Column instance
        """
        ...

    def to_column_ref(self) -> ColumnRef:
        """
        Convert this column definition to a ColumnRef.

        Returns:
            A ColumnRef referencing this column
        """
        ...

    def to_expr(self) -> Expr:
        """
        Convert this column to an expression.

        Shorthand for `Expr(self.to_column_ref())`

        Returns:
            An Expr representing this column
        """
        ...

    def cast_as(self, value: str) -> Expr:
        """
        Create a CAST expression for this column.

        Args:
            value: The target SQL type name

        Returns:
            An Expr representing CAST(column AS type)
        """
        ...

    def like(self, pattern: str, escape: typing.Optional[str] = ...) -> Expr:
        """
        Create a LIKE expression using this column.

        Args:
            pattern: The pattern to match against
            escape: Optional escape character for wildcards

        Returns:
            An Expr representing column LIKE pattern
        """
        ...

    def not_like(self, pattern: str, escape: typing.Optional[str] = ...) -> Expr:
        """
        Create a NOT LIKE expression using this column.

        Args:
            pattern: The pattern that should not match
            escape: Optional escape character for wildcards

        Returns:
            An Expr representing column NOT LIKE pattern
        """
        ...

    def __eq__(self, other: Expr) -> Expr:
        """
        Create equality comparison with this column.
        """
        ...

    def __ne__(self, other: Expr) -> Expr:
        """
        Create inequality comparison with this column.
        """
        ...

    def __gt__(self, other: Expr) -> Expr:
        """
        Create greater-than comparison with this column.
        """
        ...

    def __ge__(self, other: Expr) -> Expr:
        """
        Create greater-than-or-equal comparison with this column.
        """
        ...

    def __lt__(self, other: Expr) -> Expr:
        """
        Create less-than comparison with this column.
        """
        ...

    def __le__(self, other: Expr) -> Expr:
        """
        Create less-than-or-equal comparison with this column.
        """
        ...

    def __add__(self, other: Expr) -> Expr:
        """
        Create addition expression with this column.
        """
        ...

    def __sub__(self, other: Expr) -> Expr:
        """
        Create subtraction expression with this column.
        """
        ...

    def __and__(self, other: Expr) -> Expr:
        """
        Create logical AND expression with this column.
        """
        ...

    def __or__(self, other: Expr) -> Expr:
        """
        Create logical OR expression with this column.
        """
        ...

    def __truediv__(self, other: Expr) -> Expr:
        """
        Create division expression with this column.
        """
        ...

    def is_(self, other: Expr) -> Expr:
        """
        Create IS comparison with this column.
        """
        ...

    def sqlite_matches(self, other: Expr) -> Expr:
        """
        Create SQLite MATCH expression with this column.
        """
        ...

    def sqlite_glob(self, other: Expr) -> Expr:
        """
        Create SQLite GLOB expression with this column.
        """
        ...

    def pg_concat(self, other: Expr) -> Expr:
        """
        Create PostgreSQL concatenation with this column.
        """
        ...

    def pg_contained(self, other: Expr) -> Expr:
        """
        Create PostgreSQL contained expression with this column.
        """
        ...

    def cast_json_field(self, other: Expr) -> Expr:
        """
        Extract and cast JSON field from this column.
        """
        ...

    def get_json_field(self, other: Expr) -> Expr:
        """
        Extract JSON field from this column.
        """
        ...

    def pg_contains(self, other: Expr) -> Expr:
        """
        Create PostgreSQL contains expression with this column.
        """
        ...

    def pg_matches(self, other: Expr) -> Expr:
        """
        Create PostgreSQL full-text match with this column.
        """
        ...

    def pg_ilike(self, other: Expr) -> Expr:
        """
        Create PostgreSQL ILIKE expression with this column.
        """
        ...

    def pg_not_ilike(self, other: Expr) -> Expr:
        """
        Create PostgreSQL NOT ILIKE expression with this column.
        """
        ...

    def is_not(self, other: Expr) -> Expr:
        """
        Create IS NOT comparison with this column.
        """
        ...

    def is_null(self) -> Expr:
        """
        Create IS NULL check for this column.
        """
        ...

    def is_not_null(self) -> Expr:
        """
        Create IS NOT NULL check for this column.
        """
        ...

    def __lshift__(self, other: Expr) -> Expr:
        """
        Create bitwise left shift with this column.
        """
        ...

    def __rshift__(self, other: Expr) -> Expr:
        """
        Create bitwise right shift with this column.
        """
        ...

    def __mod__(self, other: Expr) -> Expr:
        """
        Create modulo operation with this column.
        """
        ...

    def __mul__(self, other: Expr) -> Expr:
        """
        Create multiplication with this column.
        """
        ...

    def between(self, a: Expr, b: Expr) -> Expr:
        """
        Create BETWEEN expression with this column.
        """
        ...

    def not_between(self, a: Expr, b: Expr) -> Expr:
        """
        Create NOT BETWEEN expression with this column.
        """
        ...

    def in_(self, other: typing.Sequence[Expr]) -> Expr:
        """
        Create IN expression with this column.
        """
        ...

    def not_in(self, other: typing.Sequence[Expr]) -> Expr:
        """
        Create NOT IN expression with this column.
        """
        ...
    
    def __copy__(self) -> Self:
        """
        Create a shallow copy of this Column.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this Column.

        Returns:
            A new Column instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a developer-friendly string representation.

        Returns:
            A string showing the column definition
        """
        ...

class TableName:
    """
    Represents a table name reference with optional schema, database, and alias.

    This class encapsulates a table name that can include:
    - The base table name
    - Optional schema/namespace qualification
    - Optional database qualification (for systems that support it)

    The class provides parsing capabilities for string representations
    and supports comparison operations.

    Examples:
        >>> TableName("users")                           # Simple table name
        >>> TableName("users", schema="public")          # Schema-qualified table
        >>> TableName("users", schema="hr", database="company")  # Fully qualified
    """

    name: str
    """The base name of the table."""

    schema: typing.Optional[str]
    """The schema/namespace containing the table, if specified."""

    database: typing.Optional[str]
    """The database containing the table, if specified."""

    def __new__(
        cls,
        name: str,
        schema: typing.Optional[str] = ...,
        database: typing.Optional[str] = ...,
    ) -> Self:
        """
        Create a new TableName instance.

        Args:
            name: The table name
            schema: The schema containing the table
            database: The database containing the table

        Returns:
            A new TableName instance
        """
        ...

    @classmethod
    def parse(cls, string: str) -> Self:
        """
        Parse a string representation of a table name.

        Supports formats like:
        - "table_name"
        - "schema.table_name"
        - "database.schema.table_name"

        Args:
            string: The string to parse

        Returns:
            A TableName instance representing the parsed name

        Raises:
            ValueError: If the string format is invalid
        """
        ...

    def __eq__(self, other: Self) -> bool:
        """
        Check equality with another TableName.
        """
        ...

    def __ne__(self, other: Self) -> bool:
        """
        Check inequality with another TableName.
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this TableName.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this TableName.

        Returns:
            A new TableName instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a string representation of the TableName.
        """
        ...

_ForeignKeyActions = typing.Literal["CASCADE", "NO ACTION", "RESTRICT", "SET DEFAULT", "SET NULL"]

class ForeignKeySpec:
    """
    Specifies a foreign key relationship between tables.

    Defines referential integrity constraints including:
    - Source columns (in the child table)
    - Target columns (in the parent table)
    - Actions for updates and deletes (CASCADE, RESTRICT, SET NULL, etc.)
    - Optional naming for the constraint

    Foreign keys ensure data consistency by requiring that values in the
    child table's columns match existing values in the parent table's columns.

    Example:
        >>> ForeignKeySpec(
        ...     from_columns=["user_id"],
        ...     to_columns=["id"],
        ...     to_table="users",
        ...     on_delete="CASCADE",
        ...     on_update="RESTRICT"
        ... )
    """

    from_columns: typing.List[str]
    """
    The column names in the child table that reference the parent.
    
    Note: This attribute is immutable. To modify it, create a new list:
    
        fk.from_columns.append("file_id")  # Wrong ❌
        fk.from_columns = ["id", "name"]   # Correct ✅
    """

    to_columns: typing.List[str]
    """
    The column names in the parent table being referenced.
    
    Note: This attribute is immutable. To modify it, create a new list:
    
        fk.to_columns.append("file_id")  # Wrong ❌  
        fk.to_columns = ["id", "name"]   # Correct ✅
    """

    to_table: TableName
    """The parent table being referenced."""

    from_table: typing.Optional[TableName]
    """The child table containing the foreign key (optional if inferred)."""

    name: typing.Optional[str]
    """The name of the foreign key constraint."""

    on_delete: typing.Optional[_ForeignKeyActions]
    """Action to take when referenced row is deleted."""

    on_update: typing.Optional[_ForeignKeyActions]
    """Action to take when referenced row is updated."""

    def __new__(
        cls,
        from_columns: typing.Sequence[str],
        to_columns: typing.Sequence[str],
        to_table: typing.Union[TableName, str],
        from_table: typing.Union[TableName, str, None] = ...,
        name: typing.Optional[str] = ...,
        on_delete: typing.Optional[_ForeignKeyActions] = ...,
        on_update: typing.Optional[_ForeignKeyActions] = ...,
    ) -> None:
        """
        Create a new ForeignKeySpec.

        Args:
            from_columns: Columns in the child/referencing table
            to_columns: Columns in the parent/referenced table
            to_table: The parent table being referenced
            from_table: The child table (optional, often inferred from context)
            name: Constraint name (optional)
            on_delete: Action on parent row deletion
            on_update: Action on parent row update

        Returns:
            A new ForeignKeySpec instance
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this ForeignKeySpec.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this ForeignKeySpec.

        Returns:
            A new ForeignKeySpec instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a string representation of the ForeignKeySpec.
        """
        ...

INDEX_ORDER_ASC: typing.Final[int]
"""Constant representing ascending index order."""
INDEX_ORDER_DESC: typing.Final[int]
"""Constant representing descending index order."""

class IndexColumn:
    """
    Defines a column within an index specification.

    Represents a single column's participation in an index, including:
    - The column name
    - Optional prefix length (for partial indexing)
    - Sort order (ascending or descending)

    Used within Index to specify which columns are indexed
    and how they should be ordered.

    Example:

        >>> IndexColumn("name")  # Simple column
        >>> IndexColumn("email", order=INDEX_ORDER_DESC)  # Descending order
        >>> IndexColumn("content", prefix=100)  # Prefix indexing for long text
    """

    name: str
    """The name of the column to include in the index."""

    prefix: typing.Optional[int]
    """Number of characters to index for string columns (prefix indexing)."""

    order: typing.Optional[int]
    """Sort order for this column (INDEX_ORDER_ASC or INDEX_ORDER_DESC)."""

    def __new__(
        cls, name: str, prefix: typing.Optional[int] = ..., order: typing.Optional[int] = ...
    ) -> Self:
        """
        Create a new IndexColumn.

        Args:
            name: The column name
            prefix: Prefix length for string columns
            order: Sort order (INDEX_ORDER_ASC or INDEX_ORDER_DESC)

        Returns:
            A new IndexColumn instance
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this IndexColumn.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this IndexColumn.

        Returns:
            A new IndexColumn instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a string representation of the IndexColumn.
        """
        ...

_IndexType = typing.Literal["BTREE", "FULL TEXT", "HASH"]
"""Supported index types."""

class Index:
    """
    Represents a database index specification.

    This class defines the structure and properties of a database index,
    including column definitions, uniqueness constraints, index type,
    and partial indexing conditions.

    You can use it to generate `CREATE INDEX` SQL expressions.

    Example:

        >>> Index(
        ...     columns=["id", IndexColumn("name", order=INDEX_ORDER_DESC)],
        ...     name="idx_user_name",
        ...     unique=True
        ... )
    """

    name: str
    """The name of the index."""

    table: typing.Optional[TableName]
    """The table on which to create the index."""

    if_not_exists: bool
    """Whether to use IF NOT EXISTS clause."""

    primary: bool
    """Whether this is a primary key constraint."""

    unique: bool
    """Whether this is a unique constraint."""

    nulls_not_distinct: bool
    """Whether NULL values should be considered equal for uniqueness."""

    include: typing.Sequence[str]
    """Additional columns to include in the index for covering queries."""

    columns: typing.Sequence[typing.Union[IndexColumn, str]]
    """The columns that make up this index."""

    index_type: typing.Optional[typing.Union[str, _IndexType]]
    """The type/algorithm for this index."""

    where: typing.Optional[Expr]
    """Condition for partial indexing."""

    def __new__(
        cls,
        columns: typing.Sequence[typing.Union[IndexColumn, str]],
        name: typing.Optional[str] = ...,
        table: typing.Union[str, TableName] = ...,
        if_not_exists: bool = ...,
        primary: bool = ...,
        unique: bool = ...,
        nulls_not_distinct: bool = ...,
        include: typing.Sequence[str] = ...,
        index_type: typing.Union[str, _IndexType] = ...,
        where: typing.Optional[Expr] = ...,
    ) -> Self:
        """
        Create a new Index specification.

        Args:
            columns: The columns to include in the index
            name: The index name (optional)
            table: The table to index (optional)
            if_not_exists: Whether to use IF NOT EXISTS
            primary: Whether this is a primary key
            unique: Whether to enforce uniqueness
            nulls_not_distinct: Whether NULLs are distinct for uniqueness
            include: Additional included columns
            index_type: The index algorithm type
            where: Condition for partial indexing

        Returns:
            A new Index instance
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this Index.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this Index.

        Returns:
            A new Index instance with the same values
        """
        ...


    def build(self, backend: BackendMeta) -> str:
        """
        Build a CREATE INDEX SQL string representation.

        Args:
            backend: The database backend that determines SQL dialect and formatting

        Returns:
            A SQL string representation of the expression
        """
        ...

    def __repr__(self) -> str:
        """
        Return a string representation of the Index.
        """
        ...
