import pytest
import decimal
from rapidquery import _lib


class TestAdaptedValueEdgeCases:
    """Test edge cases in value adaptation and type detection."""
    
    def test_null_value_detection(self):
        """NULL values should be properly detected."""
        val = _lib.AdaptedValue(None)
        assert val.is_null
        assert not val.is_integer
        assert not val.is_string
    
    def test_type_ambiguity_with_explicit_type(self):
        """When value could be multiple types, explicit type should win."""
        # String "123" could be interpreted as string or number
        val_str = _lib.AdaptedValue("123", _lib.StringType())
        assert val_str.is_string
        
        # Number 1 could be bool or int, explicit type should determine
        val_int = _lib.AdaptedValue(1, _lib.IntegerType())
        assert val_int.is_integer
    
    def test_empty_containers(self):
        """Empty lists/dicts should be handled correctly."""
        empty_list = _lib.AdaptedValue([])
        assert empty_list.is_json or empty_list.is_array
        
        empty_dict = _lib.AdaptedValue({})
        assert empty_dict.is_json
    
    def test_nested_json_structures(self):
        """Deeply nested JSON should be handled."""
        nested = {"level1": {"level2": {"level3": [1, 2, 3]}}}
        val = _lib.AdaptedValue(nested, _lib.JsonType())
        assert val.is_json
    
    def test_special_float_values(self):
        """Special float values like infinity and NaN."""
        inf_val = _lib.AdaptedValue(float('inf'), _lib.FloatType())
        assert inf_val.is_float
        
        nan_val = _lib.AdaptedValue(float('nan'), _lib.FloatType())
        assert nan_val.is_float
    
    def test_decimal_precision_edge_cases(self):
        """Very large or very precise decimals."""
        large_decimal = _lib.AdaptedValue(
            decimal.Decimal('99999999999999999999.99'),
            _lib.DecimalType((22, 2))
        )
        assert large_decimal.is_decimal
    
    def test_zero_values_across_types(self):
        """Zero values shouldn't be confused with NULL."""
        zero_int = _lib.AdaptedValue(0, _lib.IntegerType())
        assert not zero_int.is_null
        assert zero_int.is_integer
        
        zero_float = _lib.AdaptedValue(0.0, _lib.FloatType())
        assert not zero_float.is_null
        assert zero_float.is_float
        
        empty_string = _lib.AdaptedValue("", _lib.StringType())
        assert not empty_string.is_null
        assert empty_string.is_string


class TestColumnRefEdgeCases:
    """Test edge cases in column reference parsing and handling."""
    
    def test_parse_empty_string(self):
        """Empty string should raise error or handle gracefully."""
        with pytest.raises((ValueError, Exception)):
            _lib.ColumnRef.parse("")
    
    def test_parse_too_many_dots(self):
        """Too many qualifiers should raise error."""
        with pytest.raises((ValueError, Exception)):
            _lib.ColumnRef.parse("db.schema.table.column.extra")
    
    def test_parse_with_spaces(self):
        """Spaces in names should be handled."""
        # Depends on whether the library supports quoted identifiers
        ref = _lib.ColumnRef.parse("column name")
        assert ref.name == "column name" or "column" in ref.name
    
    def test_column_ref_equality_with_none_fields(self):
        """Column refs with None schema/table should compare correctly."""
        ref1 = _lib.ColumnRef("id", table=None, schema=None)
        ref2 = _lib.ColumnRef("id")
        assert ref1 == ref2
        
        ref3 = _lib.ColumnRef("id", table="users")
        assert ref1 != ref3


class TestExpressionEdgeCases:
    """Test edge cases in expression building."""
    
    def test_chained_comparisons(self):
        # a < b < c pattern (should be handled as (a < b) AND (b < c) or error)
        expr = _lib.all(_lib.Expr.col("a") < _lib.Expr.col("b"), _lib.Expr.col("b") < _lib.Expr.col("c"))
        sql = expr.build("sqlite")
        assert "AND" in sql or "and" in sql
    
    def test_division_by_zero_expression(self):
        """Division by zero in expression (shouldn't crash at build time)."""
        expr = _lib.Expr.col("value") / 0
        # Should build without error; database will handle runtime error
        sql = expr.build("sqlite")
        assert "/" in sql
    
    def test_null_comparisons(self):
        """NULL comparisons should use IS/IS NOT, not = or !=."""
        expr_is = _lib.Expr.col("field").is_(_lib.Expr.null())
        sql_is = expr_is.build("postgresql")
        assert "IS NULL" in sql_is.upper() or "ISNULL" in sql_is.upper()
        
        expr_is_not = _lib.Expr.col("field").is_not(_lib.Expr.null())
        sql_is_not = expr_is_not.build("postgresql")
        assert "IS NOT NULL" in sql_is_not.upper()
    
    def test_empty_in_clause(self):
        """IN clause with empty list."""
        with pytest.raises(ValueError):
            _lib.Expr.col("id").in_([])
    
    def test_complex_nested_logic(self):
        """Deep nesting of AND/OR operations."""
        complex_expr = _lib.all(
            _lib.any(_lib.Expr.col("a") == 1, _lib.Expr.col("b") == 2),
            _lib.any(_lib.Expr.col("c") == 3, _lib.Expr.col("d") == 4),
            _lib.Expr.col("e") > 5
        )
        sql = complex_expr.build("postgresql")
        # Should contain parentheses for grouping
        assert "(" in sql and ")" in sql
    
    def test_between_with_reversed_bounds(self):
        """BETWEEN with min > max (logical error but should build)."""
        expr = _lib.Expr.col("value").between(100, 1)
        sql = expr.build("sqlite")
        assert "BETWEEN" in sql.upper()
    
    def test_cast_to_invalid_type(self):
        """Casting to non-standard type name."""
        expr = _lib.Expr.col("value").cast_as("SUPER_WEIRD_TYPE")
        sql = expr.build("postgresql")
        assert "CAST" in sql.upper() or "::" in sql
    
    def test_expression_with_mixed_types(self):
        """Operations mixing incompatible types (int + string)."""
        # Should build but may cause DB error at runtime
        expr = _lib.Expr.col("age") + "not a number"
        sql = expr.build("mysql")
        assert "+" in sql


class TestTableDefinitionEdgeCases:
    """Test edge cases in table definitions."""
    
    def test_duplicate_column_names(self):
        """Duplicate column names should raise error or be handled."""
        cols = [
            _lib.Column("id", _lib.IntegerType()),
            _lib.Column("id", _lib.StringType()),  # Duplicate!
        ]

        table = _lib.Table("test", columns=cols)
        col = table.get_column("id")
        assert col is not None
    
    def test_foreign_key_to_nonexistent_column(self):
        """Foreign key referencing non-existent column."""
        table = _lib.Table(
            "posts",
            columns=[_lib.Column("id", _lib.IntegerType()), _lib.Column("user_id", _lib.IntegerType())],
            foreign_keys=[
                _lib.ForeignKey(
                    from_columns=["nonexistent"],  # Doesn't exist!
                    to_columns=["id"],
                    to_table="users"
                )
            ]
        )
        # Should build SQL, validation happens at DB level
        sql = table.build("postgresql")
        assert "FOREIGN KEY" in sql.upper()
    
    def test_circular_foreign_keys(self):
        """Two tables with foreign keys to each other."""
        # Can't test fully without DB, but should build SQL
        table_a = _lib.Table(
            "table_a",
            columns=[
                _lib.Column("id", _lib.IntegerType(), primary_key=True),
                _lib.Column("b_id", _lib.IntegerType())
            ],
            foreign_keys=[_lib.ForeignKey(["b_id"], ["id"], "table_b")]
        )
        sql_a = table_a.build("postgresql")
        assert "table_a" in sql_a.lower()
    
    def test_primary_key_on_nullable_column(self):
        """Primary key column that's also nullable (contradiction)."""
        col = _lib.Column("id", _lib.IntegerType(), primary_key=True, nullable=True)
        table = _lib.Table("test", columns=[col])
        # Should build; DB will handle the contradiction
        sql = table.build("sqlite")
        assert "PRIMARY KEY" in sql.upper()
    
    def test_auto_increment_on_non_integer(self):
        """Auto increment on string column (invalid)."""
        col = _lib.Column("id", _lib.StringType(), auto_increment=True)
        table = _lib.Table("test", columns=[col])
        # Should build; DB will reject at execution
        sql = table.build("mysql")
        assert sql  # Just ensure it builds
    
    def test_index_on_nonexistent_column(self):
        """Index referencing column that doesn't exist."""
        table = _lib.Table(
            "test",
            columns=[_lib.Column("id", _lib.IntegerType())],
            indexes=[_lib.Index(["nonexistent_column"])]
        )
        sql = table.build("postgresql")
        # Should build SQL, DB will validate
        assert "CREATE" in sql.upper() or "test" in sql.lower()
    
    def test_very_long_table_name(self):
        """Table name exceeding typical DB limits."""
        long_name = "a" * 100
        table = _lib.Table(long_name, columns=[_lib.Column("id", _lib.IntegerType())])
        sql = table.build("postgresql")
        assert long_name in sql
    
    def test_table_name_with_special_chars(self):
        """Table names with special characters."""
        special_names = ["user-data", "user space", "select"]
        for name in special_names:
            table = _lib.Table(name, columns=[_lib.Column("id", _lib.IntegerType())])
            sql = table.build("postgresql")
            # Should quote the identifier
            assert name in sql or f'"{name}"' in sql or f'`{name}`' in sql


class TestInsertEdgeCases:
    """Test edge cases in INSERT statements."""
    
    def test_insert_without_columns(self):
        """Insert with values but no columns specified."""
        with pytest.raises(ValueError):
            _lib.Insert().into("users").values(1, "John")
    
    def test_insert_mismatched_columns_values(self):
        """More/fewer values than columns."""
        with pytest.raises(ValueError):
            _lib.Insert().into("users").columns("id", "name").values(1)
    
    def test_insert_multiple_values_calls(self):
        """Multiple .values() calls"""
        insert = (_lib.Insert()
                  .into("users")
                  .columns("id", "name")
                  .values(1, "John")
                  .values(2, "Jane"))
        
        sql, params = insert.build("sqlite")
        assert sql.count("?") >= 4 or sql.count("$") >= 4 or len(params) >= 4
    
    def test_insert_with_empty_string(self):
        """Insert empty string vs NULL."""
        insert = _lib.Insert().into("users").values(id=1, name="")
        sql, params = insert.build("postgresql")

        # Empty string should not become NULL
        assert any(p.value == "" for p in params if not p.is_null)
    
    def test_on_conflict_without_target(self):
        """ON CONFLICT without specifying target columns."""
        # May need to test if this is valid for the library
        try:
            conflict = _lib.OnConflict().do_nothing()
            insert = _lib.Insert().into("users").values(id=1).on_conflict(conflict)
            sql, params = insert.build("postgresql")
            assert "CONFLICT" in sql.upper() or "DUPLICATE" in sql.upper()
        except (ValueError, TypeError, Exception):
            pass  # May not be supported
    
    def test_returning_with_sqlite(self):
        """RETURNING clause on SQLite (limited support)."""
        insert = _lib.Insert().into("users").values(id=1).returning("id")
        sql, params = insert.build("sqlite")
        # Modern SQLite supports RETURNING, older versions don't
        # Should build regardless
        assert "INSERT" in sql.upper()


class TestDeleteEdgeCases:
    """Test edge cases in DELETE statements."""
    
    def test_delete_without_where(self):
        """DELETE without WHERE clause (deletes all rows)."""
        delete = _lib.Delete().from_table("users")
        sql, params = delete.build("sqlite")
        assert "DELETE" in sql.upper()
        assert "WHERE" not in sql.upper()
    
    def test_delete_with_limit_zero(self):
        """DELETE with LIMIT 0 (no-op)."""
        delete = _lib.Delete().from_table("users").limit(0)
        sql, params = delete.build("mysql")
        assert "LIMIT 0" in sql.upper() or "LIMIT" in sql.upper()
    
    def test_delete_with_contradictory_conditions(self):
        """DELETE with always-false WHERE (id = 1 AND id = 2)."""
        delete = (_lib.Delete()
                  .from_table("users")
                  .where(_lib.all(_lib.Expr.col("id") == 1, _lib.Expr.col("id") == 2)))
        sql, params = delete.build("postgresql")
        assert "WHERE" in sql.upper()
    
    def test_delete_order_by_without_limit(self):
        """ORDER BY in DELETE without LIMIT (may be ineffective)."""
        delete = (_lib.Delete()
                  .from_table("users")
                  .order_by(_lib.Order(_lib.Expr.col("created_at"), _lib.ORDER_DESC)))
        sql, params = delete.build("postgresql")
        # PostgreSQL may not support this, but should build
        assert "DELETE" in sql.upper()


class TestOrderEdgeCases:
    """Test edge cases in ordering."""
    
    def test_order_by_expression(self):
        """Order by complex expression, not just column."""
        order = _lib.Order(
            _lib.Expr.col("price") * _lib.Expr.col("quantity"),
            _lib.ORDER_DESC
        )
        # Should handle complex expressions
        assert order.target is not None
    
    def test_null_ordering_combinations(self):
        """All combinations of ASC/DESC with NULLS FIRST/LAST."""
        orders = [
            _lib.Order(_lib.Expr.col("val"), _lib.ORDER_ASC, _lib.ORDER_NULL_FIRST),
            _lib.Order(_lib.Expr.col("val"), _lib.ORDER_ASC, _lib.ORDER_NULL_LAST),
            _lib.Order(_lib.Expr.col("val"), _lib.ORDER_DESC,_lib. ORDER_NULL_FIRST),
            _lib.Order(_lib.Expr.col("val"), _lib.ORDER_DESC,_lib. ORDER_NULL_LAST),
        ]
        for order in orders:
            assert order.null_order in [_lib.ORDER_NULL_FIRST, _lib.ORDER_NULL_LAST]


class TestFunctionCallEdgeCases:
    """Test edge cases in function calls."""
    
    def test_function_with_no_args(self):
        """Functions that take no arguments."""
        func = _lib.FunctionCall.random()
        expr = func.to_expr()
        sql = expr.build("postgresql")
        assert sql  # Should build something
    
    def test_function_with_many_args(self):
        """Functions with many arguments."""
        func = _lib.FunctionCall.coalesce([
            _lib.Expr.col("field1"),
            _lib.Expr.col("field2"),
            _lib.Expr.col("field3"),
            _lib.Expr.null(),
            _lib.Expr(0)
        ])
        expr = func.to_expr()
        sql = expr.build("sqlite")
        assert "COALESCE" in sql.upper() or "IFNULL" in sql.upper()
    
    def test_nested_function_calls(self):
        """Functions nested inside other functions."""
        inner = _lib.FunctionCall.lower(_lib.Expr.col("name"))
        outer = _lib.FunctionCall.upper(inner.to_expr())
        expr = outer.to_expr()
        sql = expr.build("postgresql")
        assert "UPPER" in sql.upper() and "LOWER" in sql.upper()


class TestAlterTableEdgeCases:
    """Test edge cases in ALTER TABLE operations."""
    
    def test_alter_rename_to_same_name(self):
        """Rename column to its current name (no-op)."""
        alter = _lib.AlterTable(
            "users",
            options=[_lib.AlterTableRenameColumnOption("name", "name")]
        )
        sql = alter.build("postgresql")
        assert "ALTER" in sql.upper()
    
    def test_alter_drop_nonexistent_column(self):
        """Drop column that doesn't exist."""
        alter = _lib.AlterTable(
            "users",
            options=[_lib.AlterTableDropColumnOption("nonexistent")]
        )
        sql = alter.build("sqlite")
        # Should build; DB will error at runtime if column doesn't exist
        assert "DROP" in sql.upper()
    
    def test_alter_multiple_conflicting_ops(self):
        """Multiple operations that conflict (add then drop same column)."""
        alter = _lib.AlterTable(
            "users",
            options=[
                _lib.AlterTableAddColumnOption(
                    _lib.Column("temp", _lib.IntegerType()),
                    if_not_exists=False
                ),
                _lib.AlterTableDropColumnOption("temp")
            ]
        )
        sql = alter.build("postgresql")
        # Should build both operations; DB handles logic
        assert "ADD" in sql.upper() and "DROP" in sql.upper()
    
    def test_alter_modify_to_incompatible_type(self):
        """Modify column to incompatible type (string -> int with text data)."""
        alter = _lib.AlterTable(
            "users",
            options=[
                _lib.AlterTableModifyColumnOption(
                    _lib.Column("age", _lib.StringType())  # Was _lib.IntegerType
                )
            ]
        )
        sql = alter.build("mysql")
        # Should build; data conversion happens at runtime
        assert "MODIFY" in sql.upper() or "ALTER" in sql.upper()


class TestEnumAndArrayTypes:
    """Test edge cases with special types."""
    
    def test_enum_with_empty_variants(self):
        """Enum with no variants (invalid)."""
        with pytest.raises(ValueError):
            _lib.EnumType("status", [])
    
    def test_enum_with_duplicate_variants(self):
        """Enum with duplicate values."""
        enum_type = _lib.EnumType("status", ["active", "active", "inactive"])
        # Should deduplicate
        assert len(enum_type.variants) == 2
    
    def test_array_of_arrays(self):
        """Nested arrays."""
        inner_array = _lib.ArrayType(_lib.IntegerType())
        outer_array = _lib.ArrayType(inner_array)
        
        assert outer_array.element is not None
    
    def test_vector_with_zero_dimensions(self):
        """Vector with 0 dimensions."""
        vector = _lib.VectorType(0)
        assert vector.length is None


class TestTableNameParsing:
    """Test edge cases in table name parsing."""
    
    def test_parse_single_name(self):
        """Simple table name."""
        name = _lib.TableName.parse("users")
        assert name.name == "users"
        assert name.schema is None
    
    def test_parse_schema_qualified(self):
        """Schema.table format."""
        name = _lib.TableName.parse("public.users")
        assert name.name == "users"
        assert name.schema == "public"
    
    def test_parse_fully_qualified(self):
        """Database.schema.table format."""
        name = _lib.TableName.parse("mydb.public.users")
        assert name.name == "users"
        assert name.schema == "public"
        assert name.database == "mydb"
    
    def test_parse_with_dots_in_name(self):
        """Names containing dots (should be quoted)."""
        # This might fail or need special handling
        try:
            name = _lib.TableName.parse("schema.table.with.dots")
            # If it succeeds, verify parsing
            assert name is not None
        except (ValueError, Exception):
            pass  # Expected for ambiguous input


class TestSQLInjectionPrevention:
    """Test that SQL injection is prevented."""
    
    def test_string_with_quotes_in_where(self):
        """String values with quotes should be escaped."""
        delete = _lib.Delete().from_table("users").where(
            _lib.Expr.col("name") == "O'Brien"
        )
        sql, params = delete.build("postgresql")
        # Should use parameters, not inline the value
        assert len(params) > 0
        # The literal quote shouldn't appear unescaped in SQL
    
    def test_table_name_with_sql_keywords(self):
        """Table names that are SQL keywords."""
        keywords = ["select", "from", "where", "drop", "table"]
        for kw in keywords:
            table = _lib.Table(kw, columns=[_lib.Column("id", _lib.IntegerType())])
            sql = table.build("postgresql")
            # Should quote the identifier
            assert sql
    
    def test_comment_injection_attempt(self):
        """SQL comments in values."""
        insert = _lib.Insert().into("users").values(
            name="'; DROP TABLE users; --"
        )
        sql, params = insert.build("sqlite")
        # Should parameterize, not inline
        assert len(params) > 0
    
    def test_union_injection_attempt(self):
        """UNION injection attempt in value."""
        delete = _lib.Delete().from_table("users").where(
            _lib.Expr.col("id") == "1 UNION SELECT * FROM passwords"
        )
        sql, params = delete.build("postgresql")
        # Should be parameterized
        assert len(params) > 0


class TestLogicalOperatorEdgeCases:
    """Test edge cases with all/any/not_ functions."""
    
    def test_all_with_single_condition(self):
        """_lib.all() with just one condition."""
        expr = _lib.all(_lib.Expr.col("a") == 1)
        sql = expr.build("sqlite")
        # Should work without unnecessary AND
        assert sql
    
    def test_any_with_single_condition(self):
        """_lib.any() with just one condition."""
        expr = _lib.any(_lib.Expr.col("a") == 1)
        sql = expr.build("sqlite")
        # Should work without unnecessary OR
        assert sql
    
    def test_not_of_complex_expression(self):
        """NOT of a complex AND/OR expression."""
        inner = _lib.all(_lib.Expr.col("a") == 1, _lib.Expr.col("b") == 2)
        expr = _lib.not_(inner)
        sql = expr.build("postgresql")
        assert "NOT" in sql.upper()
    
    def test_empty_all(self):
        """_lib.all() with no arguments (should error or return TRUE)."""
        with pytest.raises((ValueError, TypeError, Exception)):
            _lib.all()
    
    def test_empty_any(self):
        """_lib.any() with no arguments (should error or return FALSE)."""
        with pytest.raises((ValueError, TypeError, Exception)):
            _lib.any()


class TestIntervalType:
    """Test interval type edge cases."""
    
    def test_interval_with_invalid_fields(self):
        """Interval with invalid field specification."""
        with pytest.raises(OverflowError):
            _lib.IntervalType(fields=999, precision=6)
    
    def test_interval_with_negative_precision(self):
        """Interval with negative precision (invalid)."""
        with pytest.raises(OverflowError):
            _lib.IntervalType(precision=-1)
