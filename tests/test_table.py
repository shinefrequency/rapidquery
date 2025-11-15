import pytest
from rapidquery._lib import (
    Table,
    DropTable,
    AlterTable,
    TableName,
    Column,
    AlterTableAddColumnOption,
    AlterTableDropColumnOption,
    AlterTableModifyColumnOption,
    AlterTableRenameColumnOption,
    AlterTableAddForeignKeyOption,
    AlterTableDropForeignKeyOption,
    IntegerType,
    StringType,
    ForeignKey,
    Index,
)


class TestTable:
    """Test cases for Table class"""

    def test_table_creation_basic(self):
        """Test basic table creation with minimal parameters"""
        columns = [Column("id", IntegerType(), primary_key=True), Column("name", StringType(255))]

        table = Table("users", columns)

        assert table.name.name == "users"
        assert table.name.schema is None
        assert table.name.database is None
        assert len(table.columns) == 2
        assert table.c.id.name == "id"
        assert table.c.name.name == "name"
        assert table.if_not_exists is False
        assert table.temporary is False

    def test_table_creation_with_table_name_object(self):
        """Test table creation with TableName object"""
        columns = [Column("id", IntegerType())]
        table_name = TableName("products", schema="inventory")

        table = Table(table_name, columns)

        assert table.name.name == "products"
        assert table.name.schema == "inventory"

    def test_table_creation_full_parameters(self):
        """Test table creation with all parameters"""
        columns = [
            Column("id", IntegerType(), primary_key=True),
            Column("title", StringType(100), nullable=False),
        ]

        table = Table(
            name="articles",
            columns=columns,
            if_not_exists=True,
            temporary=True,
            comment="Test articles table",
            engine="InnoDB",
            collate="utf8mb4_unicode_ci",
            character_set="utf8mb4",
            extra="AUTO_INCREMENT=1000",
        )

        assert table.name.name == "articles"
        assert table.if_not_exists is True
        assert table.temporary is True
        assert table.comment == "Test articles table"
        assert table.engine == "InnoDB"
        assert table.collate == "utf8mb4_unicode_ci"
        assert table.character_set == "utf8mb4"
        assert table.extra == "AUTO_INCREMENT=1000"

    def test_table_get_column_existing(self):
        """Test retrieving existing column by name"""
        columns = [Column("id", IntegerType()), Column("email", StringType(255))]
        table = Table("users", columns)

        column = table.c.email

        assert column.name == "email"
        assert isinstance(column.type, StringType)

    def test_table_get_column_nonexistent(self):
        """Test retrieving non-existent column raises error"""
        columns = [Column("id", IntegerType())]
        table = Table("users", columns)

        with pytest.raises(KeyError):
            table.columns.nonexistent

    def test_table_build_method(self):
        """Test that build method works with backend"""
        columns = [Column("id", IntegerType(), primary_key=True), Column("name", StringType(100))]
        table = Table("test_table", columns)
        backend = "sqlite"

        sql = table.to_sql(backend)

        assert isinstance(sql, str)
        assert "CREATE TABLE" in sql.upper()
        assert "test_table" in sql

    def test_table_repr(self):
        """Test string representation"""
        columns = [Column("id", IntegerType())]
        table = Table("test", columns)

        repr_str = repr(table)

        assert isinstance(repr_str, str)
        assert "Table" in repr_str
        assert "test" in repr_str


class TestDropTable:
    """Test cases for DropTable class"""

    def test_drop_table_basic(self):
        """Test basic drop table creation"""
        drop_table = DropTable("users")

        assert drop_table.name.name == "users"
        assert drop_table.if_exists is False
        assert drop_table.restrict is False
        assert drop_table.cascade is False

    def test_drop_table_with_table_name_object(self):
        """Test drop table with TableName object"""
        table_name = TableName("products", schema="public")
        drop_table = DropTable(table_name)

        assert drop_table.name.name == "products"
        assert drop_table.name.schema == "public"

    def test_drop_table_full_parameters(self):
        """Test drop table with all parameters"""
        drop_table = DropTable(name="temp_data", if_exists=True, restrict=True, cascade=False)

        assert drop_table.name.name == "temp_data"
        assert drop_table.if_exists is True
        assert drop_table.restrict is True
        assert drop_table.cascade is False

    def test_drop_table_build_method(self):
        """Test that build method works with backend"""
        drop_table = DropTable("users", if_exists=True)
        backend = "postgres"

        # Should not raise an exception
        sql = drop_table.to_sql(backend)

        assert isinstance(sql, str)
        assert "DROP TABLE" in sql.upper()
        assert "IF EXISTS" in sql.upper()

    def test_drop_table_copy(self):
        """Test copy method"""
        drop_table = DropTable("original", if_exists=True)
        copy_table = drop_table.copy()

        assert copy_table.name.name == "original"
        assert copy_table.if_exists is True
        assert copy_table is not drop_table  # Should be different object

    def test_drop_table_repr(self):
        """Test string representation"""
        drop_table = DropTable("test_table")

        repr_str = repr(drop_table)

        assert isinstance(repr_str, str)
        assert "DropTable" in repr_str


class TestAlterTable:
    """Test cases for AlterTable class"""

    def test_alter_table_basic(self):
        """Test basic alter table creation"""
        options = [AlterTableAddColumnOption(Column("new_column", StringType(50)), False)]

        alter_table = AlterTable("users", options)

        assert alter_table.name.name == "users"
        assert len(alter_table.options) == 1
        assert isinstance(alter_table.options[0], AlterTableAddColumnOption)

    def test_alter_table_with_table_name_object(self):
        """Test alter table with TableName object"""
        table_name = TableName("products", database="shop")
        options = [AlterTableDropColumnOption("old_column")]

        alter_table = AlterTable(table_name, options)

        assert alter_table.name.name == "products"
        assert alter_table.name.database == "shop"

    def test_alter_table_add_option(self):
        """Test adding options after creation"""
        alter_table = AlterTable("users", [])

        # Initially no options
        assert len(alter_table.options) == 0

        # Add an option
        new_option = AlterTableAddColumnOption(Column("age", IntegerType()), False)
        alter_table.add_option(new_option)

        # Should now have one option
        assert len(alter_table.options) == 1
        assert alter_table.options[0].column.name == "age"

    def test_alter_table_multiple_options(self):
        """Test alter table with multiple operation types"""
        options = [
            AlterTableAddColumnOption(Column("email", StringType(255)), True),
            AlterTableDropColumnOption("old_email"),
            AlterTableModifyColumnOption(Column("name", StringType(200))),
            AlterTableRenameColumnOption("first_name", "given_name"),
        ]

        alter_table = AlterTable("customers", options)

        assert len(alter_table.options) == 4
        assert isinstance(alter_table.options[0], AlterTableAddColumnOption)
        assert isinstance(alter_table.options[1], AlterTableDropColumnOption)
        assert isinstance(alter_table.options[2], AlterTableModifyColumnOption)
        assert isinstance(alter_table.options[3], AlterTableRenameColumnOption)

    def test_alter_table_build_method(self):
        """Test that build method works with backend"""
        options = [
            AlterTableAddColumnOption(Column("status", StringType(20)), False),
            AlterTableRenameColumnOption("status", "state"),
        ]
        alter_table = AlterTable("orders", options)
        backend = "mysql"

        # Should not raise an exception
        sql = alter_table.to_sql(backend)

        assert isinstance(sql, str)
        assert "ALTER TABLE" in sql.upper()

    def test_alter_table_copy(self):
        """Test copy method"""
        options = [AlterTableDropColumnOption("temp_column")]
        alter_table = AlterTable("test_table", options)
        copy_alter = alter_table.copy()

        assert copy_alter.name.name == "test_table"
        assert len(copy_alter.options) == 1
        assert copy_alter is not alter_table  # Should be different object

    def test_alter_table_repr(self):
        """Test string representation"""
        options = [AlterTableAddColumnOption(Column("test_col", IntegerType()), False)]
        alter_table = AlterTable("test", options)

        repr_str = repr(alter_table)

        assert isinstance(repr_str, str)
        assert "AlterTable" in repr_str


class TestAlterTableOptions:
    """Test cases for individual AlterTable option classes"""

    def test_alter_table_add_column_option(self):
        """Test AddColumn option"""
        column = Column("phone", StringType(20))
        option = AlterTableAddColumnOption(column, True)

        assert option.column.name == "phone"
        assert option.if_not_exists is True

        repr_str = repr(option)
        assert "AlterTableAddColumnOption" in repr_str

    def test_alter_table_drop_column_option(self):
        """Test DropColumn option"""
        option = AlterTableDropColumnOption("obsolete_column")

        assert option.name == "obsolete_column"

        repr_str = repr(option)
        assert "AlterTableDropColumnOption" in repr_str

    def test_alter_table_modify_column_option(self):
        """Test ModifyColumn option"""
        column = Column("description", StringType(500))
        option = AlterTableModifyColumnOption(column)

        assert option.column.name == "description"
        assert isinstance(option.column.type, StringType)

        repr_str = repr(option)
        assert "AlterTableModifyColumnOption" in repr_str

    def test_alter_table_rename_column_option(self):
        """Test RenameColumn option"""
        option = AlterTableRenameColumnOption("old_name", "new_name")

        assert option.from_name == "old_name"
        assert option.to_name == "new_name"

        repr_str = repr(option)
        assert "AlterTableRenameColumnOption" in repr_str

    def test_alter_table_add_foreign_key_option(self):
        """Test AddForeignKey option"""
        foreign_key = ForeignKey(from_columns=["user_id"], to_columns=["id"], to_table="users")
        option = AlterTableAddForeignKeyOption(foreign_key)

        assert option.foreign_key.from_columns == ["user_id"]
        assert option.foreign_key.to_columns == ["id"]

        repr_str = repr(option)
        assert "AlterTableAddForeignKeyOption" in repr_str

    def test_alter_table_drop_foreign_key_option(self):
        """Test DropForeignKey option"""
        option = AlterTableDropForeignKeyOption("fk_user_profile")

        assert option.name == "fk_user_profile"

        repr_str = repr(option)
        assert "AlterTableDropForeignKeyOption" in repr_str


class TestIntegration:
    """Integration tests for table operations"""

    def test_complete_table_lifecycle(self):
        """Test complete table lifecycle: create, alter, drop"""
        # Create table
        columns = [Column("id", IntegerType(), primary_key=True), Column("name", StringType(100))]
        table = Table("products", columns)

        # Alter table
        alter_options = [
            AlterTableAddColumnOption(Column("price", IntegerType()), False),
            AlterTableRenameColumnOption("name", "product_name"),
        ]
        alter_table = AlterTable("products", alter_options)

        # Drop table
        drop_table = DropTable("products", if_exists=True)

        # Verify all objects created successfully
        assert table.name.name == "products"
        assert len(alter_table.options) == 2
        assert drop_table.name.name == "products"
        assert drop_table.if_exists is True

    def test_table_with_indexes_and_foreign_keys(self):
        """Test table creation with indexes and foreign keys"""
        columns = [
            Column("id", IntegerType(), primary_key=True),
            Column("user_id", IntegerType()),
            Column("title", StringType(200)),
        ]

        indexes = [Index(columns=["title"], name="idx_title")]

        foreign_keys = [ForeignKey(from_columns=["user_id"], to_columns=["id"], to_table="users")]

        table = Table(name="posts", columns=columns, indexes=indexes, foreign_keys=foreign_keys)

        assert len(table.columns) == 3
        assert len(table.indexes) == 1
        assert len(table.foreign_keys) == 1
        assert table.indexes[0].name == "ix_posts_title"
        assert table.foreign_keys[0].to_table.name == "users"
