from rapidquery import _lib
import pytest


exprdata = [
    (_lib.Expr(3) == 3, "3 = 3", "postgres"),
    (
        _lib.Expr.col("name").cast_as("VARCHAR(1000)").cast_as("hierarchy_path"),
        'CAST(CAST("name" AS VARCHAR(1000)) AS hierarchy_path)',
        "postgres",
    ),
    (
        (_lib.Expr.col("oh.level") + 1).between(24, 26),
        '"oh"."level" + 1 BETWEEN 24 AND 26',
        "postgres",
    ),
    (
        (_lib.Expr.col("oh.level") + 1).between(24, 26),
        '"oh"."level" + 1 BETWEEN 24 AND 26',
        "postgres",
    ),
    (
        _lib.FunctionCall.max(_lib.Expr(_lib.ColumnRef("id"))).to_expr() == 9,
        'MAX("id") = 9',
        "postgres",
    ),
    (
        _lib.all(_lib.Expr(_lib.ASTERISK).is_null(), _lib.Expr(None).is_null()),
        "* IS NULL AND NULL IS NULL",
        "postgres",
    ),
    (
        _lib.any(_lib.Expr.current_date(), _lib.Expr.current_time()),
        "CURRENT_DATE OR CURRENT_TIME",
        "postgres",
    ),
    (
        _lib.not_(_lib.FunctionCall.count(_lib.Expr(_lib.ASTERISK)).to_expr() == 1),
        "NOT COUNT(*) = 1",
        "postgres",
    ),
]


@pytest.mark.parametrize("val,expected,backend", exprdata)
def test_expr_build(val: _lib.Expr, expected: str, backend: str):
    expr = val.build(backend)
    assert expr == expected


class Unknown:
    pass


def test_invalid_expr():
    try:
        _lib.Expr(Unknown())
    except ValueError:
        pass
