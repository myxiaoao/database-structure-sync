use database_structure_sync_lib::db::{MySqlSqlGenerator, PostgresSqlGenerator, SqlGenerator};
use database_structure_sync_lib::models::*;

// ============================================================================
// Helper Functions
// ============================================================================

fn col(name: &str, data_type: &str, nullable: bool, auto_increment: bool, pos: u32) -> Column {
    Column {
        name: name.to_string(),
        data_type: data_type.to_string(),
        nullable,
        default_value: None,
        auto_increment,
        comment: None,
        ordinal_position: pos,
    }
}

fn col_full(
    name: &str,
    data_type: &str,
    nullable: bool,
    default: Option<&str>,
    auto_increment: bool,
    comment: Option<&str>,
    pos: u32,
) -> Column {
    Column {
        name: name.to_string(),
        data_type: data_type.to_string(),
        nullable,
        default_value: default.map(|s| s.to_string()),
        auto_increment,
        comment: comment.map(|s| s.to_string()),
        ordinal_position: pos,
    }
}

fn make_table(name: &str, columns: Vec<Column>) -> TableSchema {
    TableSchema {
        name: name.to_string(),
        columns,
        primary_key: None,
        indexes: vec![],
        foreign_keys: vec![],
        unique_constraints: vec![],
    }
}

fn make_index(name: &str, columns: Vec<&str>, unique: bool) -> Index {
    Index {
        name: name.to_string(),
        columns: columns.iter().map(|s| s.to_string()).collect(),
        unique,
        index_type: "BTREE".to_string(),
    }
}

fn make_fk(name: &str, columns: Vec<&str>, ref_table: &str, ref_columns: Vec<&str>) -> ForeignKey {
    ForeignKey {
        name: name.to_string(),
        columns: columns.iter().map(|s| s.to_string()).collect(),
        ref_table: ref_table.to_string(),
        ref_columns: ref_columns.iter().map(|s| s.to_string()).collect(),
        on_delete: "CASCADE".to_string(),
        on_update: "SET NULL".to_string(),
    }
}

fn make_uc(name: &str, columns: Vec<&str>) -> UniqueConstraint {
    UniqueConstraint {
        name: name.to_string(),
        columns: columns.iter().map(|s| s.to_string()).collect(),
    }
}

// ============================================================================
// MySQL: quote_identifier
// ============================================================================

#[test]
fn mysql_quote_identifier_plain() {
    let sqlgen = MySqlSqlGenerator;
    assert_eq!(sqlgen.quote_identifier("users"), "`users`");
}

#[test]
fn mysql_quote_identifier_with_backtick() {
    let sqlgen = MySqlSqlGenerator;
    assert_eq!(sqlgen.quote_identifier("user`name"), "`user``name`");
}

// ============================================================================
// MySQL: generate_create_table
// ============================================================================

#[test]
fn mysql_create_table_minimal() {
    let sqlgen = MySqlSqlGenerator;
    let table = make_table("t", vec![col("id", "INT", false, false, 1)]);
    let sql = sqlgen.generate_create_table(&table);
    assert_eq!(sql, "CREATE TABLE `t` (\n  `id` INT NOT NULL\n);");
}

#[test]
fn mysql_create_table_with_pk() {
    let sqlgen = MySqlSqlGenerator;
    let mut table = make_table("users", vec![col("id", "INT", false, true, 1)]);
    table.primary_key = Some(PrimaryKey {
        name: Some("PRIMARY".to_string()),
        columns: vec!["id".to_string()],
    });
    let sql = sqlgen.generate_create_table(&table);
    assert!(sql.contains("PRIMARY KEY (`id`)"));
    assert!(sql.contains("AUTO_INCREMENT"));
}

#[test]
fn mysql_create_table_with_index() {
    let sqlgen = MySqlSqlGenerator;
    let mut table = make_table(
        "users",
        vec![
            col("id", "INT", false, true, 1),
            col("email", "VARCHAR(255)", false, false, 2),
        ],
    );
    table.indexes = vec![make_index("idx_email", vec!["email"], false)];
    let sql = sqlgen.generate_create_table(&table);
    assert!(sql.contains("INDEX `idx_email` (`email`)"));
}

#[test]
fn mysql_create_table_with_fk() {
    let sqlgen = MySqlSqlGenerator;
    let mut table = make_table(
        "orders",
        vec![
            col("id", "INT", false, true, 1),
            col("user_id", "INT", false, false, 2),
        ],
    );
    table.foreign_keys = vec![make_fk("fk_user", vec!["user_id"], "users", vec!["id"])];
    let sql = sqlgen.generate_create_table(&table);
    assert!(sql.contains("CONSTRAINT `fk_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE CASCADE ON UPDATE SET NULL"));
}

#[test]
fn mysql_create_table_with_uc() {
    let sqlgen = MySqlSqlGenerator;
    let mut table = make_table("users", vec![col("email", "VARCHAR(255)", false, false, 1)]);
    table.unique_constraints = vec![make_uc("uq_email", vec!["email"])];
    let sql = sqlgen.generate_create_table(&table);
    assert!(sql.contains("CONSTRAINT `uq_email` UNIQUE (`email`)"));
}

#[test]
fn mysql_create_table_full() {
    let sqlgen = MySqlSqlGenerator;
    let mut table = make_table(
        "orders",
        vec![
            col_full("id", "INT", false, None, true, None, 1),
            col_full(
                "user_id",
                "INT",
                false,
                None,
                false,
                Some("FK to users"),
                2,
            ),
            col_full("status", "INT", false, Some("0"), false, None, 3),
        ],
    );
    table.primary_key = Some(PrimaryKey {
        name: Some("PRIMARY".to_string()),
        columns: vec!["id".to_string()],
    });
    table.indexes = vec![make_index("idx_status", vec!["status"], false)];
    table.foreign_keys = vec![make_fk("fk_user", vec!["user_id"], "users", vec!["id"])];
    table.unique_constraints = vec![make_uc("uq_user_status", vec!["user_id", "status"])];

    let sql = sqlgen.generate_create_table(&table);
    assert!(sql.contains("CREATE TABLE `orders`"));
    assert!(sql.contains("`id` INT NOT NULL AUTO_INCREMENT"));
    assert!(sql.contains("COMMENT 'FK to users'"));
    assert!(sql.contains("DEFAULT 0"));
    assert!(sql.contains("PRIMARY KEY (`id`)"));
    assert!(sql.contains("INDEX `idx_status` (`status`)"));
    assert!(sql.contains("CONSTRAINT `fk_user` FOREIGN KEY"));
    assert!(sql.contains("CONSTRAINT `uq_user_status` UNIQUE (`user_id`, `status`)"));
}

// ============================================================================
// MySQL: generate_drop_table
// ============================================================================

#[test]
fn mysql_drop_table() {
    let sqlgen = MySqlSqlGenerator;
    assert_eq!(sqlgen.generate_drop_table("users"), "DROP TABLE `users`;");
}

// ============================================================================
// MySQL: generate_add_column
// ============================================================================

#[test]
fn mysql_add_column_basic() {
    let sqlgen = MySqlSqlGenerator;
    let c = col("email", "VARCHAR(255)", true, false, 2);
    let sql = sqlgen.generate_add_column("users", &c);
    assert_eq!(
        sql,
        "ALTER TABLE `users` ADD COLUMN `email` VARCHAR(255);"
    );
}

#[test]
fn mysql_add_column_not_null() {
    let sqlgen = MySqlSqlGenerator;
    let c = col("email", "VARCHAR(255)", false, false, 2);
    let sql = sqlgen.generate_add_column("users", &c);
    assert!(sql.contains("NOT NULL"));
}

#[test]
fn mysql_add_column_default() {
    let sqlgen = MySqlSqlGenerator;
    let c = col_full("status", "INT", false, Some("0"), false, None, 2);
    let sql = sqlgen.generate_add_column("users", &c);
    assert!(sql.contains("DEFAULT 0"));
}

#[test]
fn mysql_add_column_auto_increment() {
    let sqlgen = MySqlSqlGenerator;
    let c = col("id", "INT", false, true, 1);
    let sql = sqlgen.generate_add_column("users", &c);
    assert!(sql.contains("AUTO_INCREMENT"));
}

#[test]
fn mysql_add_column_comment_with_single_quote() {
    let sqlgen = MySqlSqlGenerator;
    let c = col_full(
        "name",
        "VARCHAR(255)",
        true,
        None,
        false,
        Some("user's name"),
        2,
    );
    let sql = sqlgen.generate_add_column("users", &c);
    assert!(sql.contains("COMMENT 'user''s name'"));
}

#[test]
fn mysql_add_column_all_options() {
    let sqlgen = MySqlSqlGenerator;
    let c = col_full(
        "counter",
        "INT",
        false,
        Some("1"),
        true,
        Some("auto counter"),
        3,
    );
    let sql = sqlgen.generate_add_column("stats", &c);
    assert!(sql.contains("NOT NULL"));
    assert!(sql.contains("DEFAULT 1"));
    assert!(sql.contains("AUTO_INCREMENT"));
    assert!(sql.contains("COMMENT 'auto counter'"));
}

// ============================================================================
// MySQL: generate_drop_column
// ============================================================================

#[test]
fn mysql_drop_column() {
    let sqlgen = MySqlSqlGenerator;
    assert_eq!(
        sqlgen.generate_drop_column("users", "old_col"),
        "ALTER TABLE `users` DROP COLUMN `old_col`;"
    );
}

// ============================================================================
// MySQL: generate_modify_column
// ============================================================================

#[test]
fn mysql_modify_column_basic() {
    let sqlgen = MySqlSqlGenerator;
    let c = col("name", "VARCHAR(500)", true, false, 2);
    let sql = sqlgen.generate_modify_column("users", &c);
    assert_eq!(
        sql,
        "ALTER TABLE `users` MODIFY COLUMN `name` VARCHAR(500);"
    );
}

#[test]
fn mysql_modify_column_all_options() {
    let sqlgen = MySqlSqlGenerator;
    let c = col_full("id", "BIGINT", false, Some("0"), true, Some("PK"), 1);
    let sql = sqlgen.generate_modify_column("users", &c);
    assert!(sql.contains("MODIFY COLUMN"));
    assert!(sql.contains("NOT NULL"));
    assert!(sql.contains("DEFAULT 0"));
    assert!(sql.contains("AUTO_INCREMENT"));
    assert!(sql.contains("COMMENT 'PK'"));
}

// ============================================================================
// MySQL: generate_add_index
// ============================================================================

#[test]
fn mysql_add_index_plain() {
    let sqlgen = MySqlSqlGenerator;
    let idx = make_index("idx_email", vec!["email"], false);
    let sql = sqlgen.generate_add_index("users", &idx);
    assert_eq!(
        sql,
        "CREATE INDEX `idx_email` ON `users` (`email`);"
    );
}

#[test]
fn mysql_add_index_unique() {
    let sqlgen = MySqlSqlGenerator;
    let idx = make_index("idx_email", vec!["email"], true);
    let sql = sqlgen.generate_add_index("users", &idx);
    assert!(sql.contains("CREATE UNIQUE INDEX"));
}

#[test]
fn mysql_add_index_multi_column() {
    let sqlgen = MySqlSqlGenerator;
    let idx = make_index("idx_name_email", vec!["name", "email"], false);
    let sql = sqlgen.generate_add_index("users", &idx);
    assert!(sql.contains("(`name`, `email`)"));
}

// ============================================================================
// MySQL: generate_drop_index (with ON table syntax)
// ============================================================================

#[test]
fn mysql_drop_index() {
    let sqlgen = MySqlSqlGenerator;
    let sql = sqlgen.generate_drop_index("users", "idx_email");
    assert_eq!(sql, "DROP INDEX `idx_email` ON `users`;");
}

// ============================================================================
// MySQL: generate_add_foreign_key
// ============================================================================

#[test]
fn mysql_add_fk_single_column() {
    let sqlgen = MySqlSqlGenerator;
    let fk = make_fk("fk_user", vec!["user_id"], "users", vec!["id"]);
    let sql = sqlgen.generate_add_foreign_key("orders", &fk);
    assert!(sql.contains("ALTER TABLE `orders` ADD CONSTRAINT `fk_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)"));
    assert!(sql.contains("ON DELETE CASCADE ON UPDATE SET NULL"));
}

#[test]
fn mysql_add_fk_multi_column() {
    let sqlgen = MySqlSqlGenerator;
    let fk = make_fk(
        "fk_composite",
        vec!["order_id", "product_id"],
        "order_products",
        vec!["oid", "pid"],
    );
    let sql = sqlgen.generate_add_foreign_key("line_items", &fk);
    assert!(sql.contains("(`order_id`, `product_id`)"));
    assert!(sql.contains("(`oid`, `pid`)"));
}

// ============================================================================
// MySQL: generate_drop_foreign_key (DROP FOREIGN KEY syntax)
// ============================================================================

#[test]
fn mysql_drop_fk() {
    let sqlgen = MySqlSqlGenerator;
    let sql = sqlgen.generate_drop_foreign_key("orders", "fk_user");
    assert_eq!(
        sql,
        "ALTER TABLE `orders` DROP FOREIGN KEY `fk_user`;"
    );
}

// ============================================================================
// MySQL: generate_add_unique
// ============================================================================

#[test]
fn mysql_add_unique_single() {
    let sqlgen = MySqlSqlGenerator;
    let uc = make_uc("uq_email", vec!["email"]);
    let sql = sqlgen.generate_add_unique("users", &uc);
    assert_eq!(
        sql,
        "ALTER TABLE `users` ADD CONSTRAINT `uq_email` UNIQUE (`email`);"
    );
}

#[test]
fn mysql_add_unique_multi() {
    let sqlgen = MySqlSqlGenerator;
    let uc = make_uc("uq_name_email", vec!["name", "email"]);
    let sql = sqlgen.generate_add_unique("users", &uc);
    assert!(sql.contains("(`name`, `email`)"));
}

// ============================================================================
// MySQL: generate_drop_unique (DROP INDEX syntax)
// ============================================================================

#[test]
fn mysql_drop_unique() {
    let sqlgen = MySqlSqlGenerator;
    let sql = sqlgen.generate_drop_unique("users", "uq_email");
    assert_eq!(
        sql,
        "ALTER TABLE `users` DROP INDEX `uq_email`;"
    );
}

// ============================================================================
// PostgreSQL: quote_identifier
// ============================================================================

#[test]
fn pg_quote_identifier_plain() {
    let sqlgen = PostgresSqlGenerator;
    assert_eq!(sqlgen.quote_identifier("users"), "\"users\"");
}

#[test]
fn pg_quote_identifier_with_double_quote() {
    let sqlgen = PostgresSqlGenerator;
    assert_eq!(sqlgen.quote_identifier("user\"name"), "\"user\"\"name\"");
}

// ============================================================================
// PostgreSQL: generate_add_column (auto_increment → SERIAL, SERIAL skips NOT NULL)
// ============================================================================

#[test]
fn pg_add_column_auto_increment_becomes_serial() {
    let sqlgen = PostgresSqlGenerator;
    let c = col("id", "INT", false, true, 1);
    let sql = sqlgen.generate_add_column("users", &c);
    assert!(sql.contains("SERIAL"));
    assert!(!sql.contains("INT"));
}

#[test]
fn pg_add_column_serial_skips_not_null() {
    let sqlgen = PostgresSqlGenerator;
    let c = col("id", "INT", false, true, 1);
    let sql = sqlgen.generate_add_column("users", &c);
    assert!(!sql.contains("NOT NULL"));
}

#[test]
fn pg_add_column_basic() {
    let sqlgen = PostgresSqlGenerator;
    let c = col("email", "VARCHAR(255)", false, false, 2);
    let sql = sqlgen.generate_add_column("users", &c);
    assert_eq!(
        sql,
        "ALTER TABLE \"users\" ADD COLUMN \"email\" VARCHAR(255) NOT NULL;"
    );
}

// ============================================================================
// PostgreSQL: generate_modify_column (ALTER COLUMN TYPE syntax)
// ============================================================================

#[test]
fn pg_modify_column_type_syntax() {
    let sqlgen = PostgresSqlGenerator;
    let c = col("name", "VARCHAR(500)", true, false, 2);
    let sql = sqlgen.generate_modify_column("users", &c);
    assert_eq!(
        sql,
        "ALTER TABLE \"users\" ALTER COLUMN \"name\" TYPE VARCHAR(500);"
    );
    assert!(!sql.contains("MODIFY COLUMN"));
}

#[test]
fn pg_modify_column_auto_increment_serial() {
    let sqlgen = PostgresSqlGenerator;
    let c = col("id", "INT", false, true, 1);
    let sql = sqlgen.generate_modify_column("users", &c);
    assert!(sql.contains("TYPE SERIAL"));
}

// ============================================================================
// PostgreSQL: generate_create_table (indexes outside CREATE TABLE, SERIAL cols)
// ============================================================================

#[test]
fn pg_create_table_serial_column() {
    let sqlgen = PostgresSqlGenerator;
    let table = make_table("users", vec![col("id", "INT", false, true, 1)]);
    let sql = sqlgen.generate_create_table(&table);
    assert!(sql.contains("SERIAL"));
    assert!(!sql.contains("NOT NULL")); // SERIAL skips NOT NULL
}

#[test]
fn pg_create_table_indexes_outside() {
    let sqlgen = PostgresSqlGenerator;
    let mut table = make_table("users", vec![col("email", "VARCHAR(255)", false, false, 1)]);
    table.indexes = vec![make_index("idx_email", vec!["email"], false)];
    let sql = sqlgen.generate_create_table(&table);

    // The CREATE TABLE part should NOT contain the index
    let create_end = sql.find(");").unwrap();
    let create_part = &sql[..create_end + 2];
    assert!(!create_part.contains("idx_email"));

    // The index should be after the CREATE TABLE
    let after_create = &sql[create_end + 2..];
    assert!(after_create.contains("CREATE INDEX \"idx_email\" ON \"users\" (\"email\");"));
}

// ============================================================================
// PostgreSQL: generate_drop_index (no ON table)
// ============================================================================

#[test]
fn pg_drop_index_no_on_table() {
    let sqlgen = PostgresSqlGenerator;
    let sql = sqlgen.generate_drop_index("users", "idx_email");
    assert_eq!(sql, "DROP INDEX \"idx_email\";");
    assert!(!sql.contains("ON"));
}

// ============================================================================
// PostgreSQL: generate_drop_foreign_key (DROP CONSTRAINT syntax)
// ============================================================================

#[test]
fn pg_drop_fk_constraint_syntax() {
    let sqlgen = PostgresSqlGenerator;
    let sql = sqlgen.generate_drop_foreign_key("orders", "fk_user");
    assert_eq!(
        sql,
        "ALTER TABLE \"orders\" DROP CONSTRAINT \"fk_user\";"
    );
    assert!(!sql.contains("FOREIGN KEY"));
}

// ============================================================================
// PostgreSQL: generate_drop_unique (DROP CONSTRAINT syntax)
// ============================================================================

#[test]
fn pg_drop_unique_constraint_syntax() {
    let sqlgen = PostgresSqlGenerator;
    let sql = sqlgen.generate_drop_unique("users", "uq_email");
    assert_eq!(
        sql,
        "ALTER TABLE \"users\" DROP CONSTRAINT \"uq_email\";"
    );
    assert!(!sql.contains("DROP INDEX"));
}

// ============================================================================
// Cross-generator comparison tests
// ============================================================================

#[test]
fn cross_gen_different_quote_styles() {
    let mysql = MySqlSqlGenerator;
    let pg = PostgresSqlGenerator;
    assert_eq!(mysql.quote_identifier("users"), "`users`");
    assert_eq!(pg.quote_identifier("users"), "\"users\"");
}

#[test]
fn cross_gen_drop_index_syntax_difference() {
    let mysql = MySqlSqlGenerator;
    let pg = PostgresSqlGenerator;

    let mysql_sql = mysql.generate_drop_index("users", "idx_email");
    let pg_sql = pg.generate_drop_index("users", "idx_email");

    // MySQL uses ON table
    assert!(mysql_sql.contains("ON"));
    // PostgreSQL does not
    assert!(!pg_sql.contains("ON"));
}

#[test]
fn cross_gen_drop_fk_syntax_difference() {
    let mysql = MySqlSqlGenerator;
    let pg = PostgresSqlGenerator;

    let mysql_sql = mysql.generate_drop_foreign_key("orders", "fk_user");
    let pg_sql = pg.generate_drop_foreign_key("orders", "fk_user");

    // MySQL uses DROP FOREIGN KEY
    assert!(mysql_sql.contains("DROP FOREIGN KEY"));
    // PostgreSQL uses DROP CONSTRAINT
    assert!(pg_sql.contains("DROP CONSTRAINT"));
}
