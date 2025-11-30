from dataclasses import dataclass
import pytest
import typing

import rapidquery as rq


@dataclass
class SQLCase:
    expr: rq.Expr
    expected: str
    backend: str


@dataclass
class DifferentInputCase:
    value: typing.Any
    sqlcontain: str
    error: bool


sqlcases = [
    SQLCase(rq.Expr(3) == 3, "3 = 3", "postgres"),
    SQLCase(
        rq.Expr.col("name").cast_as("VARCHAR(1000)").cast_as("hierarchy_path"),
        'CAST(CAST("name" AS VARCHAR(1000)) AS hierarchy_path)',
        "postgres",
    ),
    SQLCase(
        (rq.Expr.col("oh.level") + 1).between(24, 26),
        '"oh"."level" + 1 BETWEEN 24 AND 26',
        "postgres",
    ),
    SQLCase(
        (rq.Expr.col("oh.level") + 1).between(24, 26),
        '"oh"."level" + 1 BETWEEN 24 AND 26',
        "postgres",
    ),
    SQLCase(
        rq.FunctionCall.max(rq.Expr(rq.ColumnRef("id"))).to_expr() == 9,
        'MAX("id") = 9',
        "postgres",
    ),
    SQLCase(
        rq.all(rq.Expr(rq.ASTERISK).is_null(), rq.Expr(None).is_null()),
        "* IS NULL AND NULL IS NULL",
        "postgres",
    ),
    SQLCase(
        rq.any(rq.Expr.current_date(), rq.Expr.current_time()),
        "CURRENT_DATE OR CURRENT_TIME",
        "postgres",
    ),
    SQLCase(
        rq.not_(rq.FunctionCall.count(rq.Expr(rq.ASTERISK)).to_expr() == 1),
        "NOT COUNT(*) = 1",
        "postgres",
    ),
]


@pytest.mark.parametrize("case", sqlcases)
def test_expr_build(case: SQLCase):
    expr = case.expr.to_sql(case.backend)
    assert expr == case.expected


inputcases = [
    DifferentInputCase(
        rq.Expr.custom("CUSTOM"),
        "CUSTOM",
        False,
    ),
    DifferentInputCase(
        rq.AdaptedValue(1),
        "1",
        False,
    ),
    DifferentInputCase(
        rq.ColumnRef("id"),
        '"id"',
        False,
    ),
    DifferentInputCase(
        rq.Column("id", rq.IntegerType()),
        '"id"',
        False,
    ),
    DifferentInputCase(
        (1, "rapidquery", 3),
        "(1, 'rapidquery', 3)",
        False,
    ),
    DifferentInputCase(
        rq.ASTERISK,
        "*",
        False,
    ),
    DifferentInputCase(
        rq.Select(1),
        "SELECT",
        False,
    ),
    DifferentInputCase(
        rq.Case().when(rq.Expr.col("id") == 1, True),
        "CASE WHEN",
        False,
    ),
    DifferentInputCase(
        rq.FunctionCall.avg(rq.Expr.asterisk()),
        "AVG(*)",
        False,
    ),
    DifferentInputCase(
        rq.IntegerType(),
        "",
        True,
    ),
]


@pytest.mark.parametrize("case", inputcases)
def test_input_value(case: DifferentInputCase):
    try:
        expr = rq.Expr(case.value)
    except (TypeError, ValueError, OverflowError):
        if case.error:
            return

        raise

    assert expr.to_sql("postgresql").find(case.sqlcontain) > -1


def test_invalid_expr():
    class Unknown:
        pass

    try:
        rq.Expr(Unknown())
    except ValueError:
        pass
