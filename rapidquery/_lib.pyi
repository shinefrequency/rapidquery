from typing_extensions import Self
import datetime
import decimal
import typing
import uuid

_Backends = typing.Literal["sqlite", "mysql", "postgresql", "postgres"]

class _AsteriskType:
    """
    Asterisk `"*"` - very useful for expression creating
    """

    ...

ASTERISK: typing.Final[_AsteriskType]

class SchemaStatement:
    def to_sql(self, backend: _Backends) -> str:
        """
        Build a SQL string representation.

        Args:
            backend: The database backend that determines SQL dialect and formatting

        Returns:
            A SQL string representation of the expression
        """
        ...

class QueryStatement:
    def build(self, backend: _Backends) -> typing.Tuple[str, typing.Tuple[AdaptedValue, ...]]:
        """
        Build the SQL statement with parameter values.

        Args:
            backend: The database backend that determines SQL dialect

        Returns:
            A tuple of (SQL string, parameter values)
        """
        ...

    def to_sql(self, backend: _Backends) -> str:
        """
        Build a SQL string representation.

        **This method is unsafe and can cause SQL injection.** use `.build()` method instead.

        Args:
            backend: The database backend that determines SQL dialect and formatting

        Returns:
            A SQL string representation of the expression
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

class AdaptedValue(typing.Generic[T]):
    """
    Bridges Python types, Rust types, and SQL types for seamless data conversion.

    This class handles validation, adaptation, and conversion between different
    type systems used in the application stack.

    NOTE: this class is immutable and frozen.
    """

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
    def value(self) -> T:
        """
        Converts the adapted value back to a Python type.
        """
        ...

    # `AdaptedValue` is not a child of SchemaStatement, but we used
    # `to_sql` name for this method to make compatible with others
    def to_sql(self, backend: _Backends) -> str:
        """
        Converts the adapted value to SQL.
        """
        ...

    def __hash__(self) -> int: ...
    def __eq__(self, other: Self) -> bool: ...
    def __ne__(self, other: Self) -> bool: ...
    def __repr__(self) -> str: ...

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

    @property
    def name(self) -> str: ...
    @property
    def table(self) -> typing.Optional[str]: ...
    @property
    def schema(self) -> typing.Optional[str]: ...
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

    def copy(self) -> Self: ...
    def __copy__(self) -> Self: ...
    def copy_with(
        self,
        *,
        name: typing.Optional[str] = ...,
        table: typing.Optional[str] = ...,
        schema: typing.Optional[str] = ...,
    ) -> Self: ...
    def __repr__(self) -> str: ...

_ExprValue = typing.Union[
    Expr,
    AdaptedValue,
    ColumnRef,
    Column,
    tuple,
    _AsteriskType,
    typing.Any,
    Select,
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
        e = Expr(1) > 2
        e.to_sql("mysql")
        # Result: 1 > 2

        # IN clause with tuple
        e = Expr.col("id").in_((1, 2, 3))
        e.to_sql("mysql")
        # Result: "id" IN (1, 2, 3)

        # Complex expression with functions
        e = FunctionCall.upper(Expr.col("name")).to_expr() == "JOHN"
        e.to_sql("postgres")
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
    def exists(cls, stmt: Select) -> Self: ...
    @classmethod
    def any(cls, stmt: Select) -> Self: ...
    @classmethod
    def some(cls, stmt: Select) -> Self: ...
    @classmethod
    def all(cls, stmt: Select) -> Self: ...
    @classmethod
    def tuple(
        cls,
        values: typing.Union[typing.Set[Self], typing.List[Self], typing.Tuple[Self]],
    ) -> Self:
        """
        Create a tuple expression for tuple comparisons.

        Args:
            values: A collection of expressions to include in the tuple

        Returns:
            An Expr representing a SQL tuple

        Example:
            >>> Expr.tuple(["id", "name"])
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

    def __eq__(self, other: _ExprValue) -> Self:
        """
        Create an equality comparison expression.
        """
        ...

    def __ne__(self, other: _ExprValue) -> Self:
        """
        Create an inequality comparison expression.
        """
        ...

    def __gt__(self, other: _ExprValue) -> Self:
        """
        Create a greater-than comparison expression.
        """
        ...

    def __ge__(self, other: _ExprValue) -> Self:
        """
        Create a greater-than-or-equal comparison expression.
        """
        ...

    def __lt__(self, other: _ExprValue) -> Self:
        """
        Create a less-than comparison expression.
        """
        ...

    def __le__(self, other: _ExprValue) -> Self:
        """
        Create a less-than-or-equal comparison expression.
        """
        ...

    def __add__(self, other: _ExprValue) -> Self:
        """
        Create an addition expression.
        """
        ...

    def __sub__(self, other: _ExprValue) -> Self:
        """
        Create a subtraction expression.
        """
        ...

    def __and__(self, other: _ExprValue) -> Self:
        """
        Create a logical AND expression.
        """
        ...

    def __or__(self, other: _ExprValue) -> Self:
        """
        Create a logical OR expression.
        """
        ...

    def bit_and(self, other: _ExprValue) -> Self: ...
    def bit_or(self, other: _ExprValue) -> Self: ...
    def __truediv__(self, other: _ExprValue) -> Self:
        """
        Create a division expression.
        """
        ...

    def is_(self, other: _ExprValue) -> Self:
        """
        Create an IS comparison expression (for NULL comparisons).

        Typically used with NULL: column.is_(Expr.null())

        Args:
            other: The expression to compare with

        Returns:
            A new Expr representing the IS comparison
        """
        ...

    def sqlite_matches(self, other: _ExprValue) -> Self:
        """
        Create a SQLite MATCH expression for full-text search.

        Args:
            other: The expression to match against

        Returns:
            A new Expr representing the MATCH operation
        """
        ...

    def sqlite_glob(self, other: _ExprValue) -> Self:
        """
        Create a SQLite GLOB expression for pattern matching.

        Args:
            other: The glob pattern to match against

        Returns:
            A new Expr representing the GLOB operation
        """
        ...

    def pg_concat(self, other: _ExprValue) -> Self:
        """
        Create a PostgreSQL concatenation expression using || operator.

        Args:
            other: The expression to concatenate with

        Returns:
            A new Expr representing the concatenation
        """
        ...

    def pg_contained(self, other: _ExprValue) -> Self:
        """
        Create a PostgreSQL contained expression using <@ operator.

        Used for array and range containment checks.

        Args:
            other: The expression to check containment against

        Returns:
            A new Expr representing the contained operation
        """
        ...

    def sqlite_cast_json_field(self, other: _ExprValue) -> Self:
        """
        Extract and cast a JSON field to appropriate SQL type using ->> operator.

        This operator returns the JSON field as text and can be cast to other types.

        Args:
            other: The JSON field path/name to extract

        Returns:
            A new Expr representing the JSON field extraction and casting
        """
        ...

    def sqlite_get_json_field(self, other: _ExprValue) -> Self:
        """
        Extract a JSON field using -> operator (returns JSON type).

        Args:
            other: The JSON field path/name to extract

        Returns:
            A new Expr representing the JSON field extraction
        """
        ...

    def pg_cast_json_field(self, other: _ExprValue) -> Self:
        """
        Extract and cast a JSON field to appropriate SQL type using ->> operator.

        This operator returns the JSON field as text and can be cast to other types.

        Args:
            other: The JSON field path/name to extract

        Returns:
            A new Expr representing the JSON field extraction and casting
        """
        ...

    def pg_get_json_field(self, other: _ExprValue) -> Self:
        """
        Extract a JSON field using -> operator (returns JSON type).

        Args:
            other: The JSON field path/name to extract

        Returns:
            A new Expr representing the JSON field extraction
        """
        ...

    def pg_contains(self, other: _ExprValue) -> Self:
        """
        Create a PostgreSQL contains expression using @> operator.

        Used for array and range containment checks.

        Args:
            other: The expression to check if it is contained

        Returns:
            A new Expr representing the contains operation
        """
        ...

    def pg_matches(self, other: _ExprValue) -> Self:
        """
        Create a PostgreSQL full-text search matches expression using @@ operator.

        Args:
            other: The full-text search query

        Returns:
            A new Expr representing the full-text match operation
        """
        ...

    def pg_ilike(self, other: _ExprValue) -> Self:
        """
        Create a PostgreSQL case-insensitive LIKE expression.

        Args:
            other: The pattern to match against

        Returns:
            A new Expr representing the ILIKE operation
        """
        ...

    def pg_not_ilike(self, other: _ExprValue) -> Self:
        """
        Create a PostgreSQL case-insensitive NOT LIKE expression.

        Args:
            other: The pattern that should not match

        Returns:
            A new Expr representing the NOT ILIKE operation
        """
        ...

    def is_not(self, other: _ExprValue) -> Self:
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

    def __lshift__(self, other: _ExprValue) -> Self:
        """
        Create a bitwise left shift expression.
        """
        ...

    def __rshift__(self, other: _ExprValue) -> Self:
        """
        Create a bitwise right shift expression.
        """
        ...

    def __mod__(self, other: _ExprValue) -> Self:
        """
        Create a modulo expression.
        """
        ...

    def __mul__(self, other: _ExprValue) -> Self:
        """
        Create a multiplication expression.
        """
        ...

    def between(self, a: _ExprValue, b: _ExprValue) -> Self:
        """
        Create a BETWEEN range comparison expression.

        Args:
            a: The lower bound of the range
            b: The upper bound of the range

        Returns:
            A new Expr representing the BETWEEN operation
        """
        ...

    def not_between(self, a: _ExprValue, b: _ExprValue) -> Self:
        """
        Create a NOT BETWEEN range comparison expression.

        Args:
            a: The lower bound of the range
            b: The upper bound of the range

        Returns:
            A new Expr representing the NOT BETWEEN operation
        """
        ...

    def in_subquery(self, stmt: Select) -> Self: ...
    def not_in_subquery(self, stmt: Select) -> Self: ...
    def in_(self, other: typing.Sequence[_ExprValue]) -> Self:
        """
        Create an IN membership expression.

        Args:
            other: A sequence of expressions to check membership against

        Returns:
            A new Expr representing the IN operation
        """
        ...

    def not_in(self, other: typing.Sequence[_ExprValue]) -> Self:
        """
        Create a NOT IN membership expression.

        Args:
            other: A sequence of expressions to check non-membership against

        Returns:
            A new Expr representing the NOT IN operation
        """
        ...

    # `Expr` is not a child of SchemaStatement, but we used
    # `to_sql` name for this method to make compatible with others
    def to_sql(self, backend: _Backends) -> str:
        """
        Converts the adapted value to SQL.
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

    def arg(self, arg: _ExprValue) -> Self:
        """
        Add an argument to the function call.

        Args:
            arg: The expression to add as an argument

        Returns:
            Self for method chaining
        """
        ...

    def to_expr(self) -> Expr:
        """
        Convert this function call to an expression.

        Shorthand for `Expr(self)`

        Returns:
            An Expr representing this column
        """
        ...

    @classmethod
    def sum(cls, expr: _ExprValue) -> Self: ...
    @classmethod
    def now(cls) -> Self: ...
    @classmethod
    def min(cls, expr: _ExprValue) -> Self:
        """
        Create a MIN aggregate function call.

        Args:
            expr: The expression to find the minimum of

        Returns:
            A FunctionCall representing MIN(expr)
        """
        ...

    @classmethod
    def max(cls, expr: _ExprValue) -> Self:
        """
        Create a MAX aggregate function call.

        Args:
            expr: The expression to find the maximum of

        Returns:
            A FunctionCall representing MAX(expr)
        """
        ...

    @classmethod
    def abs(cls, expr: _ExprValue) -> Self:
        """
        Create an ABS absolute value function call.

        Args:
            expr: The expression to get the absolute value of

        Returns:
            A FunctionCall representing ABS(expr)
        """
        ...

    @classmethod
    def avg(cls, expr: _ExprValue) -> Self:
        """
        Create an AVG average function call.

        Args:
            expr: The expression to calculate the average of

        Returns:
            A FunctionCall representing AVG(expr)
        """
        ...

    @classmethod
    def count(cls, expr: _ExprValue) -> Self:
        """
        Create a COUNT aggregate function call.

        Args:
            expr: The expression to count

        Returns:
            A FunctionCall representing COUNT(expr)
        """
        ...

    @classmethod
    def count_distinct(cls, expr: _ExprValue) -> Self:
        """
        Create a COUNT(DISTINCT ...) aggregate function call.

        Args:
            expr: The expression to count distinct values of

        Returns:
            A FunctionCall representing COUNT(DISTINCT expr)
        """
        ...

    @classmethod
    def if_null(cls, expr: _ExprValue) -> Self:
        """
        Create an IFNULL/COALESCE function call (database-dependent).

        Args:
            expr: The expression to check for NULL

        Returns:
            A FunctionCall representing the NULL-checking function
        """
        ...

    @classmethod
    def greatest(cls, *exprs: _ExprValue) -> Self:
        """
        Create a GREATEST function call returning the largest value.

        Args:
            exprs: Sequence of expressions to compare

        Returns:
            A FunctionCall representing GREATEST(expr1, expr2, ...)
        """
        ...

    @classmethod
    def least(cls, *exprs: _ExprValue) -> Self:
        """
        Create a LEAST function call returning the smallest value.

        Args:
            exprs: Sequence of expressions to compare

        Returns:
            A FunctionCall representing LEAST(expr1, expr2, ...)
        """
        ...

    @classmethod
    def char_length(cls, expr: _ExprValue) -> Self:
        """
        Create a CHAR_LENGTH/LENGTH function call.

        Args:
            expr: The string expression to measure

        Returns:
            A FunctionCall representing CHAR_LENGTH(expr)
        """
        ...

    @classmethod
    def coalesce(cls, *exprs: _ExprValue) -> Self:
        """
        Create a COALESCE function call returning first non-NULL value.

        Args:
            exprs: Sequence of expressions to check

        Returns:
            A FunctionCall representing COALESCE(expr1, expr2, ...)
        """
        ...

    @classmethod
    def lower(cls, expr: _ExprValue) -> Self:
        """
        Create a LOWER case conversion function call.

        Args:
            expr: The string expression to convert to lowercase

        Returns:
            A FunctionCall representing LOWER(expr)
        """
        ...

    @classmethod
    def upper(cls, expr: _ExprValue) -> Self:
        """
        Create an UPPER case conversion function call.

        Args:
            expr: The string expression to convert to uppercase

        Returns:
            A FunctionCall representing UPPER(expr)
        """
        ...

    @classmethod
    def bit_and(cls, expr: _ExprValue) -> Self:
        """
        Create a BIT_AND aggregate function call.

        Args:
            expr: The expression for bitwise AND operation

        Returns:
            A FunctionCall representing BIT_AND(expr)
        """
        ...

    @classmethod
    def bit_or(cls, expr: _ExprValue) -> Self:
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
    def round(cls, expr: _ExprValue) -> Self:
        """
        Create a ROUND function call.

        Args:
            expr: The numeric expression to round

        Returns:
            A FunctionCall representing ROUND(expr)
        """
        ...

    @classmethod
    def round_with_precision(cls, a: _ExprValue, b: _ExprValue) -> Self:
        """
        Call ROUND function with the precision.

        Args:
            a: The numeric expression to round
            b: The numeric expression to round

        Returns:
            A FunctionCall representing ROUND(a, b)
        """
        ...

    @classmethod
    def md5(cls, expr: _ExprValue) -> Self:
        """
        Create an MD5 hash function call.

        Args:
            expr: The expression to hash

        Returns:
            A FunctionCall representing MD5(expr)
        """
        ...

    # `FunctionCall` is not a child of SchemaStatement, but we used
    # `to_sql` name for this method to make compatible with others
    def to_sql(self, backend: _Backends) -> str:
        """
        Converts the adapted value to SQL.
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

def not_(arg1: Expr) -> Expr:
    """
    Create a logical NOT.

    Example:
        >>> not_(Expr.col("status") == "pending", Expr.col("status"))
        # Equivalent to: NOT status = 'pending'
    """
    ...

class Column(typing.Generic[T]):
    """
    Defines a table column with its properties and constraints.

    Represents a complete column definition including:
    - Column name and data type
    - Constraints (primary key, unique, nullable)
    - Auto-increment behavior
    - Default values and generated columns
    - Comments and extra specifications

    This class is used within Table to specify the structure
    of table columns. It encapsulates all the properties that define how
    a column behaves and what data it can store.

    Example:
        >>> Column("id", Integer(), primary_key=True, auto_increment=True)
        >>> Column("name", String(255), nullable=False, default="unknown")
        >>> Column("created_at", Timestamp(), default=Expr.current_timestamp())
    """

    name: str
    """The name of the column."""

    type: ColumnTypeMeta[T]
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
        type: ColumnTypeMeta[T],
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

    def adapt(self, value: T) -> AdaptedValue[T]:
        """
        Shorthand for `AdaptedValue(value, type=self.type)`
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

    def __new__(
        cls,
        name: str,
        schema: typing.Optional[str] = ...,
        database: typing.Optional[str] = ...,
        alias: typing.Optional[str] = ...,
    ) -> Self:
        """
        Create a new TableName instance.
        """
        ...

    @property
    def name(self) -> str: ...
    @property
    def schema(self) -> typing.Optional[str]: ...
    @property
    def database(self) -> typing.Optional[str]: ...
    @property
    def alias(self) -> typing.Optional[str]: ...
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

    def copy_with(
        self,
        *,
        name: typing.Optional[str] = ...,
        schema: typing.Optional[str] = ...,
        database: typing.Optional[str] = ...,
        alias: typing.Optional[str] = ...,
    ) -> Self: ...
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

class ForeignKey:
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
        >>> ForeignKey(
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
        Create a new ForeignKey.

        Args:
            from_columns: Columns in the child/referencing table
            to_columns: Columns in the parent/referenced table
            to_table: The parent table being referenced
            from_table: The child table (optional, often inferred from context)
            name: Constraint name (optional)
            on_delete: Action on parent row deletion
            on_update: Action on parent row update

        Returns:
            A new ForeignKey instance
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this ForeignKey.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this ForeignKey.

        Returns:
            A new ForeignKey instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a string representation of the ForeignKey.
        """
        ...

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
        >>> IndexColumn("email", order="desc")  # Descending order
        >>> IndexColumn("content", prefix=100)  # Prefix indexing for long text
    """

    name: str
    """The name of the column to include in the index."""

    prefix: typing.Optional[int]
    """Number of characters to index for string columns (prefix indexing)."""

    order: typing.Optional[typing.Literal["asc", "desc"]]
    """Sort order for this column ("asc" or "desc")."""

    def __new__(
        cls,
        name: str,
        prefix: typing.Optional[int] = ...,
        order: typing.Optional[typing.Literal["asc", "desc"]] = ...,
    ) -> Self:
        """
        Create a new IndexColumn.

        Args:
            name: The column name
            prefix: Prefix length for string columns
            order: Sort order ("asc" or "desc")

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

class Index(SchemaStatement):
    """
    Represents a database index specification.

    This class defines the structure and properties of a database index,
    including column definitions, uniqueness constraints, index type,
    and partial indexing conditions.

    You can use it to generate `CREATE INDEX` SQL expressions.

    Example:

        >>> Index(
        ...     columns=["id", IndexColumn("name", order="desc")],
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

    def __repr__(self) -> str:
        """
        Return a string representation of the Index.
        """
        ...

class DropIndex(SchemaStatement):
    """
    Represents a DROP INDEX SQL statement.

    Builds index deletion statements with support for:
    - Conditional deletion (IF EXISTS)
    - Table-specific index dropping (for databases that require it)
    - Proper error handling for non-existent indexes

    Example:

        >>> DropIndex(
        ...     name="idx_user_name",
        ...     if_exists=True
        ... )
    """

    name: str
    """The name of the index to drop."""

    table: typing.Optional[TableName]
    """The table from which to drop the index."""

    if_exists: bool
    """Whether to use IF EXISTS clause to avoid errors."""

    def __new__(
        self,
        name: str = ...,
        table: typing.Optional[TableName] = ...,
        if_exists: bool = ...,
    ) -> Self:
        """
        Create a new DropIndex.

        Args:
            name: The index name
            table: The table from which to drop the index (optional)
            if_exists: Whether to use IF EXISTS

        Returns:
            A new Index instance
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this DropIndex.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this DropIndex.

        Returns:
            A new DropIndex instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a string representation of the DropIndex.
        """
        ...

class _TableColumnsSequence:
    def __getattr__(self, name: str) -> Column: ...
    def get(self, name: str) -> Column: ...
    def append(self, col: Column) -> None: ...
    def remove(self, name: str) -> None: ...
    def to_list(self) -> typing.Sequence[Column]: ...
    def clear(self) -> None: ...
    def __len__(self) -> int: ...

class Table(SchemaStatement):
    """
    Represents a complete database table definition.

    This class encapsulates all aspects of a table structure including:
    - Column definitions with their types and constraints
    - Indexes for query optimization
    - Foreign key relationships for referential integrity
    - Check constraints for data validation
    - Table-level options like engine, collation, and character set

    Used to generate CREATE TABLE SQL statements with full schema specifications.

    Example:
        >>> Table(
        ...     "users",
        ...     columns=[
        ...         Column("id", Integer, primary_key=True, auto_increment=True),
        ...         Column("email", String(255), unique=True, nullable=False)
        ...     ],
        ...     indexes=[Index(["email"])],
        ...     if_not_exists=True
        ... )
    """

    indexes: typing.Sequence[Index]
    """Indexes defined on this table for query optimization."""

    foreign_keys: typing.Sequence[ForeignKey]
    """Foreign key constraints defining relationships with other tables."""

    checks: typing.Sequence[Expr]
    """Check constraints for validating data integrity."""

    if_not_exists: bool
    """Whether to use IF NOT EXISTS clause to avoid errors if table exists."""

    temporary: bool
    """Whether this is a temporary table that exists only for the session."""

    comment: typing.Optional[str]
    """Comment describing the purpose of this table."""

    engine: typing.Optional[str]
    """Storage engine for the table (e.g., InnoDB, MyISAM for MySQL)."""

    collate: typing.Optional[str]
    """Collation for string comparisons and sorting in this table."""

    character_set: typing.Optional[str]
    """Character set encoding for text data in this table."""

    extra: typing.Optional[str]
    """Additional table-specific options for the CREATE TABLE statement."""

    def __new__(
        cls,
        name: typing.Union[str, TableName],
        columns: typing.Sequence[Column],
        indexes: typing.Sequence[Index] = ...,
        foreign_keys: typing.Sequence[ForeignKey] = ...,
        checks: typing.Sequence[Expr] = ...,
        if_not_exists: bool = ...,
        temporary: bool = ...,
        comment: typing.Optional[str] = ...,
        engine: typing.Optional[str] = ...,
        collate: typing.Optional[str] = ...,
        character_set: typing.Optional[str] = ...,
        extra: typing.Optional[str] = ...,
    ) -> Self:
        """
        Create a new Table definition.

        Args:
            name: The table name, optionally schema-qualified
            columns: List of column definitions
            indexes: List of index definitions
            foreign_keys: List of foreign key constraints
            checks: List of check constraint expressions
            if_not_exists: Whether to use IF NOT EXISTS clause
            temporary: Whether to create a temporary table
            comment: Table description comment
            engine: Storage engine specification
            collate: Collation specification
            character_set: Character set specification
            extra: Additional SQL specifications

        Returns:
            A new Table instance
        """
        ...

    @property
    def name(self) -> TableName:
        """The name of this table."""
        ...

    @property
    def columns(self) -> _TableColumnsSequence:
        """Returns columns as `_TableColumnsSequence`"""
        ...

    @property
    def c(self) -> _TableColumnsSequence:
        """Returns columns as `_TableColumnsSequence`. It is an alias for `self.columns`"""
        ...

    def __repr__(self) -> str: ...

class _AliasedTableColumnsSequence:
    def __getattr__(self, name: str) -> ColumnRef: ...
    def get(self, name: str) -> ColumnRef: ...
    def __len__(self) -> int: ...

class AliasedTable:
    def __new__(cls, table: typing.Union[Table, Self], alias: str) -> Self: ...
    @property
    def name(self) -> TableName:
        """The name of aliased table."""
        ...

    @property
    def columns(self) -> _AliasedTableColumnsSequence:
        """Returns columns as `_AliasedTableColumnsSequence`"""
        ...

    @property
    def c(self) -> _AliasedTableColumnsSequence:
        """Returns columns as `_AliasedTableColumnsSequence`. It is an alias for `self.columns`"""
        ...

    def __repr__(self) -> str: ...

class DropTable(SchemaStatement):
    """
    Represents a DROP TABLE SQL statement.

    Builds table deletion statements with support for:
    - Conditional deletion (IF EXISTS) to avoid errors
    - CASCADE to drop dependent objects
    - RESTRICT to prevent deletion if dependencies exist

    Example:
        >>> DropTable("old_users", if_exists=True, cascade=True)
    """

    name: TableName
    """The name of the table to drop."""

    if_exists: bool
    """Whether to use IF EXISTS clause to avoid errors if table doesn't exist."""

    restrict: bool
    """Whether to use RESTRICT to prevent dropping if dependencies exist."""

    cascade: bool
    """Whether to use CASCADE to drop dependent objects automatically."""

    def __new__(
        cls,
        name: typing.Union[str, TableName],
        if_exists: bool = ...,
        restrict: bool = ...,
        cascade: bool = ...,
    ) -> Self:
        """
        Create a new DropTable statement.

        Args:
            name: The table name to drop
            if_exists: Whether to use IF EXISTS clause
            restrict: Whether to use RESTRICT mode
            cascade: Whether to use CASCADE mode

        Returns:
            A new DropTable instance
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this DropTable.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this DropTable.

        Returns:
            A new DropTable instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a developer-friendly string representation.

        Returns:
            A string showing the drop table statement
        """
        ...

class RenameTable(SchemaStatement):
    """
    Represents a RENAME TABLE SQL statement.

    Changes the name of an existing table to a new name. Both names can be
    schema-qualified if needed.

    Example:
        >>> RenameTable("old_users", "users")
        >>> RenameTable("public.old_users", "archive.users")
    """

    from_name: TableName
    """The current name of the table."""

    to_name: TableName
    """The new name for the table."""

    def __new__(
        cls,
        from_name: typing.Union[str, TableName],
        to_name: typing.Union[str, TableName],
    ) -> Self:
        """
        Create a new RenameTable statement.

        Args:
            from_name: The current table name
            to_name: The new table name

        Returns:
            A new RenameTable instance
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this RenameTable.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this RenameTable.

        Returns:
            A new RenameTable instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a developer-friendly string representation.

        Returns:
            A string showing the rename table statement
        """
        ...

class TruncateTable(SchemaStatement):
    """
    Represents a TRUNCATE TABLE SQL statement.

    Quickly removes all rows from a table, typically faster than DELETE
    and with different transaction and trigger behavior depending on the
    database system.

    Example:
        >>> TruncateTable("temp_data")
    """

    name: TableName
    """The name of the table to truncate."""

    def __new__(
        cls,
        name: typing.Union[str, TableName],
    ) -> Self:
        """
        Create a new TruncateTable statement.

        Args:
            name: The table name to truncate

        Returns:
            A new TruncateTable instance
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this TruncateTable.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this TruncateTable.

        Returns:
            A new TruncateTable instance with the same values
        """
        ...

    def __repr__(self) -> str:
        """
        Return a developer-friendly string representation.

        Returns:
            A string showing the truncate table statement
        """
        ...

class AlterTableOptionMeta:
    """
    Base class for all ALTER TABLE operation types.

    This abstract base class represents the different types of modifications
    that can be made to an existing table structure, such as adding/dropping
    columns, modifying column definitions, or managing foreign keys.
    """

    def __new__(cls): ...
    def __repr__(self) -> str:
        """
        Return a developer-friendly string representation.
        """
        ...

class AlterTableAddColumnOption(AlterTableOptionMeta):
    """
    ALTER TABLE operation to add a new column.

    Adds a column to an existing table with optional IF NOT EXISTS clause
    to prevent errors if the column already exists.

    Example:
        >>> AlterTableAddColumnOption(
        ...     Column("created_at", Timestamp, nullable=False),
        ...     if_not_exists=True
        ... )
    """

    def __new__(cls, column: Column, if_not_exists: bool) -> Self: ...
    @property
    def column(self) -> Column: ...
    @property
    def if_not_exists(self) -> bool: ...

class AlterTableAddForeignKeyOption(AlterTableOptionMeta):
    """
    ALTER TABLE operation to add a foreign key constraint.

    Adds referential integrity between tables by creating a foreign key
    relationship on an existing table.

    Example:
        >>> AlterTableAddForeignKeyOption(
        ...     ForeignKey(
        ...         from_columns=["user_id"],
        ...         to_columns=["id"],
        ...         to_table="users",
        ...         on_delete="CASCADE"
        ...     )
        ... )
    """

    def __new__(cls, foreign_key: ForeignKey) -> Self: ...
    @property
    def foreign_key(self) -> ForeignKey: ...
    def __repr__(self) -> str: ...

class AlterTableDropColumnOption(AlterTableOptionMeta):
    """
    ALTER TABLE operation to drop an existing column.

    Removes a column from the table. This operation may fail if the column
    is referenced by other database objects.

    Example:
        >>> AlterTableDropColumnOption("deprecated_field")
    """

    def __new__(cls, name: str) -> Self: ...
    @property
    def name(self) -> str: ...
    def __repr__(self) -> str: ...

class AlterTableDropForeignKeyOption(AlterTableOptionMeta):
    """
    ALTER TABLE operation to drop a foreign key constraint.

    Removes a foreign key relationship by its constraint name.

    Example:
        >>> AlterTableDropForeignKeyOption("fk_user_posts")
    """

    def __new__(cls, name: str) -> Self: ...
    @property
    def name(self) -> str: ...
    def __repr__(self) -> str: ...

class AlterTableModifyColumnOption(AlterTableOptionMeta):
    """
    ALTER TABLE operation to modify a column definition.

    Changes properties of an existing column such as type, nullability,
    default value, or other constraints.

    Example:
        >>> AlterTableModifyColumnOption(
        ...     Column("email", String(512), nullable=False)
        ... )
    """

    def __new__(cls, column: Column) -> Self: ...
    @property
    def column(self) -> Column: ...
    def __repr__(self) -> str: ...

class AlterTableRenameColumnOption(AlterTableOptionMeta):
    """
    ALTER TABLE operation to rename a column.

    Changes the name of an existing column without modifying its type
    or constraints.

    Example:
        >>> AlterTableRenameColumnOption("old_name", "new_name")
    """

    def __new__(cls, from_name: str, to_name: str) -> Self: ...
    @property
    def from_name(self) -> str: ...
    @property
    def to_name(self) -> str: ...
    def __repr__(self) -> str: ...

class AlterTable(SchemaStatement):
    """
    Represents an ALTER TABLE SQL statement.

    Provides a flexible way to modify existing table structures by applying
    one or more alteration operations such as adding/dropping columns,
    modifying column definitions, or managing constraints.

    Multiple operations can be batched together in a single ALTER TABLE
    statement for efficiency.

    Example:
        >>> AlterTable(
        ...     "users",
        ...     options=[
        ...         AlterTableAddColumnOption(Column("status", String(20))),
        ...         AlterTableModifyColumnOption(Column("email", String(512)))
        ...     ]
        ... )
    """

    name: TableName
    """The name of the table to alter."""

    options: typing.Sequence[AlterTableOptionMeta]
    """The list of alteration operations to apply."""

    def __new__(
        cls, name: typing.Union[str, TableName], options: typing.Sequence[AlterTableOptionMeta]
    ) -> Self:
        """
        Create a new AlterTable statement.

        Args:
            name: The table name to alter
            options: List of alteration operations

        Returns:
            A new AlterTable instance
        """
        ...

    def add_option(self, option: AlterTableOptionMeta) -> None:
        """
        Add an alteration operation to this ALTER TABLE statement.

        Args:
            option: The alteration operation to add
        """
        ...

    def __copy__(self) -> Self:
        """
        Create a shallow copy of this AlterTable.
        """
        ...

    def copy(self) -> Self:
        """
        Create a copy of this AlterTable.

        Returns:
            A new AlterTable instance with the same values
        """
        ...

    def __repr__(self) -> str: ...

class OnConflict:
    """
    Specifies conflict resolution behavior for INSERT statements.

    Handles situations where an INSERT would violate a unique constraint
    or primary key. Supports various strategies:
    - DO NOTHING: Skip the conflicting row
    - DO UPDATE: Update the existing row with new values

    This corresponds to INSERT ... ON CONFLICT in PostgreSQL and
    INSERT ... ON DUPLICATE KEY UPDATE in MySQL.

    Example:
        >>> OnConflict("email").do_update("name")
        >>> OnConflict("id", "version").do_nothing()
    """

    def __new__(cls, *targets: typing.Union[str, Column]) -> Self:
        """
        Create a new conflict resolution specification.

        Args:
            *targets: Column names or Column objects that define the conflict constraint

        Returns:
            A new OnConflict instance
        """
        ...

    def do_nothing(self, *keys: typing.Union[str, Column]) -> Self:
        """
        Specify DO NOTHING action for conflicts.

        When a conflict occurs, the conflicting row will be skipped.

        Args:
            *keys: Provide primary keys if you are using MySQL, for MySQL specific polyfill.

        Returns:
            Self for method chaining
        """
        ...

    @typing.overload
    def do_update(self, *keys: typing.Union[str, Column]) -> Self:
        """
        Specify DO UPDATE action for conflicts using column names.

        Args:
            *keys: Columns to update on conflict

        Returns:
            Self for method chaining
        """
        ...

    @typing.overload
    def do_update(self, **kwds: _ExprValue) -> Self:
        """
        Specify DO UPDATE action for conflicts with explicit values.

        Args:
            **kwds: Column names and their new values

        Returns:
            Self for method chaining
        """
        ...

    def target_where(self, condition: Expr) -> Self:
        """
        Add a WHERE clause to the conflict target (partial unique index).

        Args:
            condition: The condition that must match for the conflict to apply

        Returns:
            Self for method chaining
        """
        ...

    def action_where(self, condition: Expr) -> Self:
        """
        Add a WHERE clause to the conflict action (conditional update).

        Args:
            condition: The condition that must be true for the update to occur

        Returns:
            Self for method chaining
        """
        ...

    def __repr__(self) -> str: ...

class Insert(QueryStatement):
    """
    Builds INSERT SQL statements with a fluent interface.

    Provides a chainable API for constructing INSERT queries with support for:
    - Single or multiple row insertion
    - Conflict resolution (UPSERT)
    - RETURNING clauses
    - REPLACE functionality
    - Default values

    Example:
        >>> Insert().into("users").columns("name", "email").values(
        ...     "John", "john@example.com"
        ... ).returning_all()
        >>> Insert().into("users").values(name="Jane", email="jane@example.com")
    """

    def __new__(cls) -> Self:
        """
        Create a new INSERT statement builder.

        Returns:
            A new Insert instance
        """
        ...

    def replace(self) -> Self:
        """
        Convert this INSERT to a REPLACE statement.

        REPLACE will delete existing rows that conflict with the new row
        before inserting.

        Returns:
            Self for method chaining
        """
        ...

    def into(self, table: typing.Union[str, Table, TableName]) -> Self:
        """
        Specify the target table for insertion.

        Args:
            table: The table name, Table object, or TableName to insert into

        Returns:
            Self for method chaining
        """
        ...

    def columns(self, *args: typing.Union[Column, str]) -> Self:
        """
        Specify the columns for insertion.

        Args:
            *args: Column names or Column objects

        Returns:
            Self for method chaining
        """
        ...

    @typing.overload
    def values(self, **kwds: _ExprValue) -> Self:
        """
        Specify values to insert using keyword arguments.

        Args:
            **kwds: Column names and their values

        Returns:
            Self for method chaining
        """
        ...

    @typing.overload
    def values(self, *args: _ExprValue) -> Self:
        """
        Specify values to insert using positional arguments.

        Args:
            *args: Values in the same order as columns

        Returns:
            Self for method chaining
        """
        ...

    def or_default_values(self, rows: int = ...) -> Self:
        """
        Use DEFAULT VALUES if no values were specified.

        Args:
            rows: Number of rows to insert with default values

        Returns:
            Self for method chaining
        """
        ...

    def on_conflict(self, action: OnConflict) -> Self:
        """
        Specify conflict resolution behavior (UPSERT).

        Args:
            action: The OnConflict specification

        Returns:
            Self for method chaining
        """
        ...

    def returning(self, *args: typing.Union[Column, str]) -> Self:
        """
        Specify columns to return from the inserted rows.

        Args:
            *args: Column names or Column objects to return

        Returns:
            Self for method chaining
        """
        ...

    def returning_all(self) -> Self:
        """
        Return all columns from the inserted rows.

        Returns:
            Self for method chaining
        """
        ...

    def __repr__(self) -> str: ...

class Delete(QueryStatement):
    """
    Builds DELETE SQL statements with a fluent interface.

    Provides a chainable API for constructing DELETE queries with support for:
    - WHERE conditions for filtering
    - LIMIT for restricting deletion count
    - ORDER BY for determining deletion order
    - RETURNING clauses for getting deleted data
    """

    def __new__(cls) -> Self:
        """
        Create a new DELETE statement builder.

        Returns:
            A new Delete instance
        """
        ...

    def from_table(self, table: typing.Union[str, Table, TableName]) -> Self:
        """
        Specify the table to delete from.

        Args:
            table: The table name, Table object, or TableName

        Returns:
            Self for method chaining
        """
        ...

    def limit(self, n: int) -> Self:
        """
        Limit the number of rows to delete.

        Args:
            n: Maximum number of rows to delete

        Returns:
            Self for method chaining
        """
        ...

    def returning(self, *args: typing.Union[Column, str]) -> Self:
        """
        Specify columns to return from the deleted rows.

        Args:
            *args: Column names or Column objects to return

        Returns:
            Self for method chaining
        """
        ...

    def returning_all(self) -> Self:
        """
        Return all columns from the deleted rows.

        Returns:
            Self for method chaining
        """
        ...

    def where(self, condition: _ExprValue) -> Self:
        """
        Add a WHERE condition to filter rows to delete.

        Args:
            condition: The filter condition expression

        Returns:
            Self for method chaining
        """
        ...

    def order_by(
        self,
        target: _ExprValue,
        order: typing.Literal["asc", "desc"],
        null_order: typing.Optional[typing.Literal["first", "last"]] = ...,
    ) -> Self:
        """
        Specify the order in which to delete rows.

        Typically used with LIMIT to delete specific rows.

        Returns:
            Self for method chaining
        """
        ...

    def __repr__(self) -> str: ...

class Update(QueryStatement):
    """
    Builds UPDATE SQL statements with a fluent interface.

    Provides a chainable API for constructing UPDATE queries with support for:
    - Setting column values
    - WHERE conditions for filtering
    - LIMIT for restricting update count
    - ORDER BY for determining update order
    - RETURNING clauses for getting updated data

    Example:
        >>> Update().table("users").values(
        ...     status="active", last_updated=datetime.now()
        ... ).where(Expr.col("id") == 123).returning_all()
        >>> Update().table("users").values(budget=Expr.col("budget") + 10) \\
        ...     .where(Expr.col("name").like(r"%ali%")) \\
        ...     .order_by(Order(Expr.col("id"), ORDER_ASC)) \\
        ...     .returning("age")
    """

    def __new__(cls) -> Self:
        """
        Create a new UPDATE statement builder.

        Returns:
            A new Update instance
        """
        ...

    def table(self, table: typing.Union[str, Table, TableName]) -> Self:
        """
        Specify the table to update.

        Args:
            table: The table name, Table object, or TableName

        Returns:
            Self for method chaining
        """
        ...

    def from_table(self, table: typing.Union[str, Table, TableName]) -> Self:
        """
        Update using data from another table (`UPDATE .. FROM ..`).

        **Notes** \\
        MySQL doesn't support the UPDATE FROM syntax. And the current implementation attempt to tranform it to the UPDATE JOIN syntax, which only works for one join target.
        
        Args:
            table: The table name, Table object, or TableName

        Returns:
            Self for method chaining
        """
        ...

    def values(self, **kwds: _ExprValue) -> Self:
        """
        Specify columns and their new values.

        Args:
            **kwds: Column names and their new values as keyword arguments

        Returns:
            Self for method chaining
        """
        ...

    def where(self, condition: _ExprValue) -> Self:
        """
        Add a WHERE condition to filter rows to update.

        Args:
            condition: The filter condition expression

        Returns:
            Self for method chaining
        """
        ...

    def order_by(
        self,
        target: _ExprValue,
        order: typing.Literal["asc", "desc"],
        null_order: typing.Optional[typing.Literal["first", "last"]] = ...,
    ) -> Self:
        """
        Specify the order in which to update rows.

        Typically used with LIMIT to update specific rows.

        Returns:
            Self for method chaining
        """
        ...

    def returning(self, *args: typing.Union[Column, str]) -> Self:
        """
        Specify columns to return from the updated rows.

        Args:
            *args: Column names or Column objects to return

        Returns:
            Self for method chaining
        """
        ...

    def returning_all(self) -> Self:
        """
        Return all columns from the updated rows.

        Returns:
            Self for method chaining
        """
        ...

    def limit(self, n: int) -> Self:
        """
        Limit the number of rows to update.

        Args:
            n: Maximum number of rows to update

        Returns:
            Self for method chaining
        """
        ...

    def __repr__(self) -> str: ...

class SelectExpr:
    """
    Represents a column expression with an optional alias in a SELECT clause.

    Used to specify both the expression to select and an optional alias name
    for the result column.

    Example:
        >>> SelectExpr(Expr.col("price") * 1.1, "price_with_tax")
        >>> SelectExpr(Expr.count(), "total_count")
    """

    def __new__(cls, expr: _ExprValue, alias: typing.Optional[str] = ...):
        """
        Create a new SelectExpr.

        Args:
            expr: The expression to select
            alias: Optional alias name for the result column

        Returns:
            A new SelectExpr instance
        """
        ...

    @property
    def expr(self) -> Expr:
        """The expression to be selected."""
        ...

    @property
    def alias(self) -> typing.Optional[str]:
        """The alias name for the result column, if any."""
        ...

    def __repr__(self) -> str: ...

class Select(QueryStatement):
    """
    Builds SELECT SQL statements with a fluent interface.

    Provides a chainable API for constructing SELECT queries with support for:
    - Column selection with expressions and aliases
    - Table and subquery sources
    - Filtering with WHERE and HAVING
    - Joins (inner, left, right, full, cross, lateral)
    - Grouping and aggregation
    - Ordering and pagination
    - Set operations (UNION, EXCEPT, INTERSECT)
    - Row locking for transactions
    - DISTINCT queries

    Example:
        >>> Select(Expr.col("name"), Expr.col("email")).from_table("users") \\
        ...     .where(Expr.col("active") == True) \\
        ...     .order_by(Order(Expr.col("created_at"), ORDER_DESC)) \\
        ...     .limit(10)
        >>> Select().columns("id", "title").from_table("posts") \\
        ...     .join("users", Expr.col("posts.user_id") == Expr.col("users.id")) \\
        ...     .where(Expr.col("published") == True)
    """

    def __new__(cls, *cols: typing.Union[SelectExpr, _ExprValue]) -> Self:
        """
        Create a new SELECT statement builder.

        Args:
            *cols: Optional initial columns to select (expressions or SelectExpr objects)

        Returns:
            A new Select instance
        """
        ...

    def distinct(self, *on: typing.Union[Column, ColumnRef, str]) -> Self:
        """
        Make this a DISTINCT query to eliminate duplicate rows.

        Args:
            *on: Optional columns for DISTINCT ON (PostgreSQL-specific)

        Returns:
            Self for method chaining
        """
        ...

    def columns(self, *cols: typing.Union[SelectExpr, _ExprValue]) -> Self:
        """
        Specify or add columns to select.

        Args:
            *cols: Column names, expressions, or SelectExpr objects to select

        Returns:
            Self for method chaining
        """
        ...

    def from_table(self, table: typing.Union[Table, TableName, str]) -> Self:
        """
        Specify the source table for the query.

        Args:
            table: The table name, Table object, or TableName to select from

        Returns:
            Self for method chaining
        """
        ...

    def from_subquery(self, subquery: Select, alias: str) -> Self:
        """
        Use a subquery as the data source.

        Args:
            subquery: The SELECT query to use as a subquery
            alias: Alias name for the subquery

        Returns:
            Self for method chaining
        """
        ...

    def from_function(self, function: FunctionCall, alias: str) -> Self:
        """
        Use a table-returning function as the data source.

        Args:
            function: The function call that returns table data
            alias: Alias name for the function result

        Returns:
            Self for method chaining
        """
        ...

    def limit(self, n: int) -> Self:
        """
        Limit the number of rows returned.

        Args:
            n: Maximum number of rows to return

        Returns:
            Self for method chaining
        """
        ...

    def offset(self, n: int) -> Self:
        """
        Skip a number of rows before returning results.

        Typically used with LIMIT for pagination.

        Args:
            n: Number of rows to skip

        Returns:
            Self for method chaining
        """
        ...

    def where(self, condition: _ExprValue) -> Self:
        """
        Add a WHERE condition to filter rows.

        Args:
            condition: The filter condition expression

        Returns:
            Self for method chaining
        """
        ...

    def having(self, condition: _ExprValue) -> Self:
        """
        Add a HAVING condition to filter grouped results.

        Used with GROUP BY to filter aggregated data.

        Args:
            condition: The filter condition expression

        Returns:
            Self for method chaining
        """
        ...

    def order_by(
        self,
        target: _ExprValue,
        order: typing.Literal["asc", "desc"],
        null_order: typing.Optional[typing.Literal["first", "last"]] = ...,
    ) -> Self:
        """
        Specify the order of results.

        Returns:
            Self for method chaining
        """
        ...

    def lock(
        self,
        type: typing.Literal["exclusive", "shared"] = ...,
        behavior: typing.Optional[typing.Literal["nowait", "skip"]] = ...,
        tables: typing.Sequence[typing.Union[str, TableName, Table]] = ...,
    ) -> Self:
        """
        Add row locking for transactional queries (FOR UPDATE/FOR SHARE).

        Args:
            type: Lock type - "exclusive" (FOR UPDATE) or "shared" (FOR SHARE)
            behavior: Optional lock behavior - "nowait" or "skip" (SKIP LOCKED)
            tables: Optional specific tables to lock (for multi-table queries)

        Returns:
            Self for method chaining
        """
        ...

    def group_by(
        self,
        *cols: _ExprValue,
    ) -> Self:
        """
        Group results by specified columns for aggregation.

        Args:
            *cols: Column names or expressions to group by

        Returns:
            Self for method chaining
        """
        ...

    def union(
        self,
        statement: Self,
        type: typing.Literal["all", "except", "intersect", "distinct"] = ...,
    ) -> Self:
        """
        Combine this query with another using set operations.

        Args:
            statement: The SELECT query to combine with
            type: Set operation type:
                - "distinct": UNION (default, removes duplicates)
                - "all": UNION ALL (keeps duplicates)
                - "except": EXCEPT (rows in first but not second)
                - "intersect": INTERSECT (rows in both queries)

        Returns:
            Self for method chaining
        """
        ...

    def join(
        self,
        table: typing.Union[str, TableName, Table, AliasedTable],
        on: _ExprValue,
        type: typing.Literal["", "cross", "full", "inner", "right", "left"] = ...,
    ) -> Self:
        """
        Join another table to the query.

        Args:
            table: The table name, Table object, or TableName to join
            on: The join condition expression
            type: Join type:
                - "": Default join (typically INNER)
                - "inner": INNER JOIN
                - "left": LEFT JOIN (LEFT OUTER JOIN)
                - "right": RIGHT JOIN (RIGHT OUTER JOIN)
                - "full": FULL JOIN (FULL OUTER JOIN)
                - "cross": CROSS JOIN

        Returns:
            Self for method chaining
        """
        ...

    def join_lateral(
        self,
        query: Self,
        alias: str,
        on: _ExprValue,
        type: typing.Literal["", "cross", "full", "inner", "right", "left"] = ...,
    ) -> Self:
        """
        Join a lateral subquery (subquery that can reference prior FROM items).

        LATERAL allows the subquery to reference columns from preceding tables
        in the FROM clause. Useful for correlated subqueries in joins.

        Args:
            query: The SELECT query to join laterally
            alias: Alias name for the lateral subquery
            on: The join condition expression
            type: Join type (see join() for options)

        Returns:
            Self for method chaining
        """
        ...

    def __repr__(self) -> str: ...
