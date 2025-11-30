import dataclasses
import typing
import pytest

import rapidquery as rq


@dataclasses.dataclass
class ColumnTestCase:
    name: str
    type: typing.Type
    primary_key: bool = False
    unique: bool = False
    nullable: bool = False
    auto_increment: bool = False
    extra: str | None = None
    comment: str | None = None
    default_expr: str = ""
    stored_generated: bool = False
    column_ref: typing.Optional[rq.ColumnRef] = None


def test_different_types():
    # Simple
    ty = rq.IntegerType()
    assert ty == rq.IntegerType()
    assert repr(ty) == "<IntegerType >"

    # Length
    ty = rq.StringType(None)
    assert ty == rq.StringType()
    assert ty.length is None
    assert repr(ty) == "<StringType length=None>"

    ty = rq.StringType(20)
    assert ty != rq.StringType(30)
    assert ty != rq.StringType(None)
    assert ty == rq.StringType(20)
    assert ty.length == 20
    assert repr(ty) == "<StringType length=20>"

    # Percision Scale
    ty = rq.MoneyType()
    assert ty == rq.MoneyType()
    assert ty.precision_scale is None
    assert repr(ty) == "<MoneyType precision_scale=None>"

    ty = rq.MoneyType((10, 8))
    assert ty != rq.MoneyType((4, 6))
    assert ty != rq.MoneyType(None)
    assert ty == rq.MoneyType((10, 8))
    assert ty.precision_scale == (10, 8)
    assert repr(ty) == "<MoneyType precision_scale=(10, 8)>"

    # Enum
    ty = rq.EnumType("priority", ["low", "medium"])
    assert ty.name == "priority"
    assert ty.variants == ["low", "medium"]

    assert ty == rq.EnumType("priority", ["low", "medium"])
    assert ty != rq.EnumType("priority", ["low", "medium", "high"])

    # Array
    try:
        ty = rq.ArrayType(str)
    except Exception:
        pass
    else:
        pytest.fail()

    ty = rq.ArrayType(rq.TextType())
    assert ty.element == rq.TextType()

    # Interval
    try:
        ty = rq.IntervalType(5983)
    except Exception:
        pass
    else:
        pytest.fail()

    ty = rq.IntervalType(rq.INTERVAL_DAY_TO_MINUTE)
    assert ty.fields == rq.INTERVAL_DAY_TO_MINUTE
    assert ty.precision is None

    ty = rq.IntervalType(rq.INTERVAL_HOUR, 5)
    assert ty.fields == rq.INTERVAL_HOUR
    assert ty.precision == 5


_metadata_column = rq.Column(
    "metadata", rq.ArrayType(rq.IntegerType()), nullable=True, default=[1, 2, 3]
)
rq.Table("users", [_metadata_column])


columndata = [
    (
        rq.Column(
            "id",
            rq.BigIntegerType(),
            primary_key=True,
            nullable=False,
            auto_increment=True,
            default=1,
        ),
        ColumnTestCase(
            "id",
            rq.BigIntegerType,
            primary_key=True,
            nullable=False,
            auto_increment=True,
            default_expr="1",
            column_ref=rq.ColumnRef("id"),
        ),
    ),
    (
        _metadata_column,
        ColumnTestCase(
            "metadata",
            rq.ArrayType,
            nullable=True,
            default_expr="ARRAY [1,2,3]",
            column_ref=rq.ColumnRef("metadata", table="users"),
        ),
    ),
]


@pytest.mark.parametrize("val,case", columndata)
def test_column(val: rq.Column, case: ColumnTestCase):
    assert val.name == case.name
    assert val.primary_key == case.primary_key
    assert val.unique == case.unique
    assert val.nullable == case.nullable
    assert val.auto_increment == case.auto_increment
    assert val.extra == case.extra
    assert val.comment == case.comment
    assert val.stored_generated == case.stored_generated
    assert val.to_column_ref() == case.column_ref
    assert val.default.to_sql("postgres") == case.default_expr

    val.extra = "HELLO"
    val.comment = "COMMENT"

    assert val.extra == "HELLO"
    assert val.comment == "COMMENT"
