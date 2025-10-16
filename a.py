from rapidquery import _lib


tb = _lib.Table(
    "users",
    columns=[
        _lib.Column("id", _lib.BigIntegerType(), primary_key=True, auto_increment=True),
        _lib.Column("name", _lib.CharType(64), nullable=False),
        _lib.Column("file_id", _lib.BigIntegerType(), nullable=False),
    ],
    indexes=[
        _lib.Index(["name"]),
    ],
    foreign_keys=[
        _lib.ForeignKeySpec(from_columns=["file_id"], to_columns=["id"], to_table="files")
    ],
    checks=[_lib.Expr.col("name") == _lib.Expr("admin")],
    if_not_exists=True,
)
print(tb.build(_lib.PostgreSQLBackend()))
# CREATE TABLE IF NOT EXISTS "users" (
#   "name" char(64) NOT NULL,
#   "id" bigserial,
#   "file_id" bigint NOT NULL,
#   CONSTRAINT "ix_2d41781a598441" PRIMARY KEY ("id"),
#   CONSTRAINT "FK_b977597b04be57" FOREIGN KEY ("file_id") REFERENCES "files" ("id"),
#   CHECK ("name" = 'admin')
# );
# CREATE INDEX "ix_5e714e65e2a04af7b465ce15bab5b73d" ON "users" ("name");

for i in tb.c.to_list():
    print(i.primary_key)
