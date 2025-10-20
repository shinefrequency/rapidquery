from rapidquery import _lib
import pytest


exprdata = [
    (_lib.Expr(3) == 3, "3 = 3", _lib.PostgreSQLBackend()),
    (
        _lib.Expr.col("name").cast_as("VARCHAR(1000)").cast_as("hierarchy_path"),
        'CAST(CAST("name" AS VARCHAR(1000)) AS hierarchy_path)',
        _lib.PostgreSQLBackend(),
    ),
    (
        (_lib.Expr.col("oh.level") + 1).between(24, 26),
        '"oh"."level" + 1 BETWEEN 24 AND 26',
        _lib.PostgreSQLBackend(),
    ),
    (
        (_lib.Expr.col("oh.level") + 1).between(24, 26),
        '"oh"."level" + 1 BETWEEN 24 AND 26',
        _lib.PostgreSQLBackend(),
    ),
    (
        _lib.FunctionCall.max(_lib.Expr(_lib.ColumnRef("id"))).to_expr() == 9,
        'MAX("id") = 9',
        _lib.PostgreSQLBackend(),
    ),
    (
        _lib.all(_lib.Expr(_lib.ASTERISK).is_null(), _lib.Expr(None).is_null()),
        "* IS NULL AND NULL IS NULL",
        _lib.PostgreSQLBackend(),
    ),
    (
        _lib.any(_lib.Expr.current_date(), _lib.Expr.current_time()),
        "CURRENT_DATE OR CURRENT_TIME",
        _lib.PostgreSQLBackend(),
    ),
    (
        _lib.not_(_lib.FunctionCall.count(_lib.Expr(_lib.ASTERISK)).to_expr() == 1),
        "NOT COUNT(*) = 1",
        _lib.PostgreSQLBackend(),
    ),
]


@pytest.mark.parametrize("val,expected,backend", exprdata)
def test_expr_build(val: _lib.Expr, expected: str, backend: _lib.BackendMeta):
    expr = val.build(backend)
    assert expr == expected


class Unknown:
    pass


def test_invalid_expr():
    try:
        _lib.Expr(Unknown())
    except ValueError:
        pass


def _benchmark_expr_calling(val, eq):
    _lib.Expr(val) == eq


def test_expr_benchmark(benchmark):
    benchmark(_benchmark_expr_calling, {"json": "hello"}, ["LIST"])
