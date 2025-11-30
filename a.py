import rapidquery as rq


query = (
    rq.Select(
        rq.Expr.col("account_number"),
        rq.Expr.col("transaction_date"),
        rq.Expr.col("transaction_type"),
        rq.Expr.col("amount"),
        rq.SelectCol(
            rq.Case()
            .when(rq.Expr.col("transaction_type") == "DEBIT", -rq.Expr.col("amount"))
            .else_(rq.Expr.col("amount")),
            alias="signed_amount",
        ),
        rq.SelectCol(
            rq.FunctionCall.sum(
                rq.Case()
                .when(rq.Expr.col("transaction_type") == "DEBIT", -rq.Expr.col("amount"))
                .else_(rq.Expr.col("amount"))
            ),
            alias="running_balance",
            window="account_window",
        ),
        rq.SelectCol(
            rq.FunctionCall.avg(rq.Expr.col("amount")),
            alias="avg_transaction_by_type",
            window=rq.Window(rq.Expr.col("account_number"), rq.Expr.col("transaction_type")),
        ),
        rq.SelectCol(
            rq.FunctionCall.percent_rank(),
            alias="amount_percentile",
            window=rq.Window(rq.Expr.col("account_number")).order_by(rq.Expr.col("amount"), "desc"),
        ),
    )
    .from_table("bank_transactions")
    .where(
        rq.Expr.col("transaction_date").between(
            rq.Expr.custom("'2024-01-01'"), rq.Expr.custom("'2024-12-31'")
        )
    )
    .window(
        "account_window",
        rq.Window(rq.Expr.col("account_number"))
        .order_by(rq.Expr.col("transaction_date"), "desc")
        .order_by(rq.Expr.col("transaction_id"), "desc")
        .frame("rows", rq.WindowFrame.unbounded_preceding(), rq.WindowFrame.current_row()),
    )
)

print(query.to_sql("postgresql"))
"""
SELECT 
    "account_number",
    "transaction_date",
    "transaction_type",
    "amount",
    (CASE WHEN ("transaction_type" = 'DEBIT') THEN "amount" * -1 ELSE "amount" END) AS "signed_amount",
    SUM(
        (CASE WHEN ("transaction_type" = 'DEBIT') THEN "amount" * -1 ELSE "amount" END)
    ) OVER "account_window" AS "running_balance",
    AVG("amount") OVER (PARTITION BY "account_number", "transaction_type") AS "avg_transaction_by_type",
    PERCENT_RANK() OVER (
        PARTITION BY "account_number" ORDER BY "amount" DESC
    ) AS "amount_percentile"
FROM "bank_transactions"
WHERE
    "transaction_date" BETWEEN ('2024-01-01') AND ('2024-12-31')
WINDOW
    "account_window" AS (
        PARTITION BY "account_number"
        ORDER BY "transaction_date" DESC, "transaction_id" DESC
        ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
    )
"""
