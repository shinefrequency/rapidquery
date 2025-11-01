from rapidquery import _lib
import pytest
import decimal
import datetime
import uuid


inferdata = [
    (12, "is_integer"),
    (12.4, "is_float"),
    ("string", "is_string"),
    (b"bytes", "is_bytes"),
    (True, "is_boolean"),
    (list(), "is_json"),
    (dict(), "is_json"),
    ({2: 3}, "is_json"),
    (decimal.Decimal(3.4), "is_decimal"),
    (datetime.datetime.now(), "is_datetime"),
    (datetime.datetime.now(tz=datetime.timezone.utc), "is_datetime"),
    (datetime.datetime.now().date(), "is_date"),
    (datetime.datetime.now().time(), "is_time"),
    (uuid.uuid4(), "is_uuid"),
    (None, "is_null"),
]
specificdata = [
    (12, "is_integer", _lib.TinyUnsignedType()),
    (12.4, "is_float", _lib.FloatType()),
    ("string", "is_json", _lib.JsonBinaryType()),
    ("string", "is_string", _lib.EnumType("name", ["var1"])),
    (b"bytes", "is_bytes", _lib.BlobType()),
    (True, "is_json", _lib.JsonType()),
    (list(), "is_array", _lib.ArrayType(_lib.IntegerType())),
    (uuid.uuid4(), "is_uuid", _lib.UuidType()),
]


@pytest.mark.parametrize("value,attribute", inferdata)
def test_infer(value, attribute):
    adapted = _lib.AdaptedValue(value)
    assert getattr(adapted, attribute) is True

    _lib.Expr(adapted)  # Force AdaptedValue to adapt


@pytest.mark.parametrize("value,attribute,typ", specificdata)
def test_specific_type(value, attribute, typ):
    adapted = _lib.AdaptedValue(value, type=typ)
    assert getattr(adapted, attribute) is True

    _lib.Expr(adapted)  # Force AdaptedValue to adapt
