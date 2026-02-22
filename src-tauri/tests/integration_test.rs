use database_structure_sync_lib::db::SqlGenerator;
use database_structure_sync_lib::diff::compare_schemas;
use database_structure_sync_lib::error::AppError;
use database_structure_sync_lib::models::*;

// ============================================================================
// Mock SQL Generator
// ============================================================================

struct MockSqlGen;

impl SqlGenerator for MockSqlGen {
    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name)
    }

    fn generate_create_table(&self, table: &TableSchema) -> String {
        format!("CREATE TABLE \"{}\"", table.name)
    }

    fn generate_drop_table(&self, table_name: &str) -> String {
        format!("DROP TABLE \"{}\"", table_name)
    }

    fn generate_add_column(&self, table_name: &str, column: &Column) -> String {
        format!(
            "ALTER TABLE \"{}\" ADD COLUMN \"{}\" {}",
            table_name, column.name, column.data_type
        )
    }

    fn generate_drop_column(&self, table_name: &str, column_name: &str) -> String {
        format!(
            "ALTER TABLE \"{}\" DROP COLUMN \"{}\"",
            table_name, column_name
        )
    }

    fn generate_modify_column(&self, table_name: &str, column: &Column) -> String {
        format!(
            "ALTER TABLE \"{}\" MODIFY COLUMN \"{}\" {}",
            table_name, column.name, column.data_type
        )
    }

    fn generate_add_index(&self, table_name: &str, index: &Index) -> String {
        let idx_type = if index.unique {
            "UNIQUE INDEX"
        } else {
            "INDEX"
        };
        format!(
            "CREATE {} \"{}\" ON \"{}\" ({})",
            idx_type,
            index.name,
            table_name,
            index.columns.join(", ")
        )
    }

    fn generate_drop_index(&self, table_name: &str, index_name: &str) -> String {
        format!("DROP INDEX \"{}\" ON \"{}\"", index_name, table_name)
    }

    fn generate_add_foreign_key(&self, table_name: &str, fk: &ForeignKey) -> String {
        format!(
            "ALTER TABLE \"{}\" ADD CONSTRAINT \"{}\" FOREIGN KEY ({}) REFERENCES \"{}\" ({})",
            table_name,
            fk.name,
            fk.columns.join(", "),
            fk.ref_table,
            fk.ref_columns.join(", ")
        )
    }

    fn generate_drop_foreign_key(&self, table_name: &str, fk_name: &str) -> String {
        format!(
            "ALTER TABLE \"{}\" DROP FOREIGN KEY \"{}\"",
            table_name, fk_name
        )
    }

    fn generate_add_unique(&self, table_name: &str, unique: &UniqueConstraint) -> String {
        format!(
            "ALTER TABLE \"{}\" ADD CONSTRAINT \"{}\" UNIQUE ({})",
            table_name,
            unique.name,
            unique.columns.join(", ")
        )
    }

    fn generate_drop_unique(&self, table_name: &str, unique_name: &str) -> String {
        format!(
            "ALTER TABLE \"{}\" DROP CONSTRAINT \"{}\"",
            table_name, unique_name
        )
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_column(
    name: &str,
    data_type: &str,
    nullable: bool,
    auto_increment: bool,
    position: u32,
) -> Column {
    Column {
        name: name.to_string(),
        data_type: data_type.to_string(),
        nullable,
        default_value: None,
        auto_increment,
        comment: None,
        ordinal_position: position,
    }
}

fn create_column_with_default(
    name: &str,
    data_type: &str,
    nullable: bool,
    default: &str,
    position: u32,
) -> Column {
    Column {
        name: name.to_string(),
        data_type: data_type.to_string(),
        nullable,
        default_value: Some(default.to_string()),
        auto_increment: false,
        comment: None,
        ordinal_position: position,
    }
}

fn create_index(name: &str, columns: Vec<&str>, unique: bool) -> Index {
    Index {
        name: name.to_string(),
        columns: columns.iter().map(|s| s.to_string()).collect(),
        unique,
        index_type: "BTREE".to_string(),
    }
}

fn create_foreign_key(
    name: &str,
    columns: Vec<&str>,
    ref_table: &str,
    ref_columns: Vec<&str>,
) -> ForeignKey {
    ForeignKey {
        name: name.to_string(),
        columns: columns.iter().map(|s| s.to_string()).collect(),
        ref_table: ref_table.to_string(),
        ref_columns: ref_columns.iter().map(|s| s.to_string()).collect(),
        on_delete: "CASCADE".to_string(),
        on_update: "CASCADE".to_string(),
    }
}

fn create_unique_constraint(name: &str, columns: Vec<&str>) -> UniqueConstraint {
    UniqueConstraint {
        name: name.to_string(),
        columns: columns.iter().map(|s| s.to_string()).collect(),
    }
}

fn create_table(name: &str, columns: Vec<Column>) -> TableSchema {
    TableSchema {
        name: name.to_string(),
        columns,
        primary_key: None,
        indexes: vec![],
        foreign_keys: vec![],
        unique_constraints: vec![],
    }
}

// ============================================================================
// Table Level Tests
// ============================================================================

#[test]
fn test_compare_empty_schemas() {
    let source: Vec<TableSchema> = vec![];
    let target: Vec<TableSchema> = vec![];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);
    assert!(diffs.is_empty());
}

#[test]
fn test_detect_new_table() {
    let source = vec![create_table(
        "users",
        vec![create_column("id", "INT", false, true, 1)],
    )];
    let target: Vec<TableSchema> = vec![];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].diff_type, DiffType::TableAdded);
    assert_eq!(diffs[0].table_name, "users");
    assert!(diffs[0].sql.contains("CREATE TABLE"));
}

#[test]
fn test_detect_removed_table() {
    let source: Vec<TableSchema> = vec![];
    let target = vec![create_table("old_table", vec![])];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].diff_type, DiffType::TableRemoved);
    assert_eq!(diffs[0].table_name, "old_table");
    assert!(diffs[0].sql.contains("DROP TABLE"));
}

#[test]
fn test_detect_multiple_tables_added() {
    let source = vec![
        create_table("users", vec![create_column("id", "INT", false, true, 1)]),
        create_table("orders", vec![create_column("id", "INT", false, true, 1)]),
        create_table("products", vec![create_column("id", "INT", false, true, 1)]),
    ];
    let target: Vec<TableSchema> = vec![];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);
    assert_eq!(diffs.len(), 3);
    assert!(diffs.iter().all(|d| d.diff_type == DiffType::TableAdded));
}

#[test]
fn test_identical_tables_no_diff() {
    let columns = vec![
        create_column("id", "INT", false, true, 1),
        create_column("name", "VARCHAR(255)", true, false, 2),
    ];
    let source = vec![create_table("users", columns.clone())];
    let target = vec![create_table("users", columns)];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);
    assert!(diffs.is_empty());
}

// ============================================================================
// Column Level Tests
// ============================================================================

#[test]
fn test_detect_added_column() {
    let source = vec![create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("email", "VARCHAR(255)", true, false, 2),
        ],
    )];

    let target = vec![create_table(
        "users",
        vec![create_column("id", "INT", false, true, 1)],
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_added = diffs.iter().find(|d| d.diff_type == DiffType::ColumnAdded);
    assert!(column_added.is_some());

    let diff = column_added.unwrap();
    assert_eq!(diff.table_name, "users");
    assert_eq!(diff.object_name, Some("email".to_string()));
    assert!(diff.sql.contains("ADD COLUMN"));
}

#[test]
fn test_detect_removed_column() {
    let source = vec![create_table(
        "users",
        vec![create_column("id", "INT", false, true, 1)],
    )];

    let target = vec![create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("old_field", "TEXT", true, false, 2),
        ],
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_removed = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ColumnRemoved);
    assert!(column_removed.is_some());

    let diff = column_removed.unwrap();
    assert_eq!(diff.table_name, "users");
    assert_eq!(diff.object_name, Some("old_field".to_string()));
    assert!(diff.sql.contains("DROP COLUMN"));
}

#[test]
fn test_detect_modified_column_type() {
    let source = vec![create_table(
        "users",
        vec![create_column("name", "VARCHAR(255)", false, false, 1)],
    )];

    let target = vec![create_table(
        "users",
        vec![create_column("name", "VARCHAR(100)", false, false, 1)],
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_modified = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(column_modified.is_some());

    let diff = column_modified.unwrap();
    assert_eq!(diff.source_def, Some("VARCHAR(255)".to_string()));
    assert_eq!(diff.target_def, Some("VARCHAR(100)".to_string()));
}

#[test]
fn test_detect_modified_column_nullable() {
    let source = vec![create_table(
        "users",
        vec![
            create_column("email", "VARCHAR(255)", false, false, 1), // NOT NULL
        ],
    )];

    let target = vec![create_table(
        "users",
        vec![
            create_column("email", "VARCHAR(255)", true, false, 1), // NULL
        ],
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_modified = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(column_modified.is_some());
}

#[test]
fn test_detect_modified_column_default() {
    let source = vec![create_table(
        "users",
        vec![create_column_with_default("status", "INT", false, "1", 1)],
    )];

    let target = vec![create_table(
        "users",
        vec![create_column_with_default("status", "INT", false, "0", 1)],
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_modified = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(column_modified.is_some());
}

#[test]
fn test_multiple_column_changes() {
    let source = vec![create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("name", "VARCHAR(255)", false, false, 2),
            create_column("new_col", "TEXT", true, false, 3),
        ],
    )];

    let target = vec![create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("name", "VARCHAR(100)", false, false, 2), // Modified
            create_column("old_col", "TEXT", true, false, 3),       // To be removed
        ],
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    assert!(
        diffs.iter().any(|d| d.diff_type == DiffType::ColumnAdded
            && d.object_name == Some("new_col".to_string()))
    );
    assert!(
        diffs.iter().any(|d| d.diff_type == DiffType::ColumnRemoved
            && d.object_name == Some("old_col".to_string()))
    );
    assert!(
        diffs.iter().any(|d| d.diff_type == DiffType::ColumnModified
            && d.object_name == Some("name".to_string()))
    );
}

// ============================================================================
// Index Level Tests
// ============================================================================

#[test]
fn test_detect_added_index() {
    let mut source_table = create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("email", "VARCHAR(255)", true, false, 2),
        ],
    );
    source_table.indexes = vec![create_index("idx_email", vec!["email"], false)];

    let target_table = create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("email", "VARCHAR(255)", true, false, 2),
        ],
    );

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let index_added = diffs.iter().find(|d| d.diff_type == DiffType::IndexAdded);
    assert!(index_added.is_some());

    let diff = index_added.unwrap();
    assert_eq!(diff.object_name, Some("idx_email".to_string()));
    assert!(diff.sql.contains("CREATE INDEX"));
}

#[test]
fn test_detect_removed_index() {
    let source_table = create_table("users", vec![create_column("id", "INT", false, true, 1)]);

    let mut target_table = create_table("users", vec![create_column("id", "INT", false, true, 1)]);
    target_table.indexes = vec![create_index("idx_old", vec!["id"], false)];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let index_removed = diffs.iter().find(|d| d.diff_type == DiffType::IndexRemoved);
    assert!(index_removed.is_some());
    assert!(index_removed.unwrap().sql.contains("DROP INDEX"));
}

#[test]
fn test_detect_modified_index() {
    let mut source_table = create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("email", "VARCHAR(255)", true, false, 2),
            create_column("name", "VARCHAR(255)", true, false, 3),
        ],
    );
    source_table.indexes = vec![create_index("idx_search", vec!["email", "name"], false)];

    let mut target_table = create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("email", "VARCHAR(255)", true, false, 2),
            create_column("name", "VARCHAR(255)", true, false, 3),
        ],
    );
    target_table.indexes = vec![create_index("idx_search", vec!["email"], false)]; // Different columns

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let index_modified = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::IndexModified);
    assert!(index_modified.is_some());

    let diff = index_modified.unwrap();
    assert!(diff.sql.contains("DROP INDEX"));
    assert!(diff.sql.contains("CREATE INDEX"));
}

#[test]
fn test_detect_unique_index_added() {
    let mut source_table = create_table(
        "users",
        vec![create_column("email", "VARCHAR(255)", false, false, 1)],
    );
    source_table.indexes = vec![create_index("idx_email_unique", vec!["email"], true)];

    let target_table = create_table(
        "users",
        vec![create_column("email", "VARCHAR(255)", false, false, 1)],
    );

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let index_added = diffs.iter().find(|d| d.diff_type == DiffType::IndexAdded);
    assert!(index_added.is_some());
    assert!(index_added.unwrap().sql.contains("UNIQUE INDEX"));
}

// ============================================================================
// Foreign Key Level Tests
// ============================================================================

#[test]
fn test_detect_added_foreign_key() {
    let mut source_table = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
        ],
    );
    source_table.foreign_keys = vec![create_foreign_key(
        "fk_user",
        vec!["user_id"],
        "users",
        vec!["id"],
    )];

    let target_table = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
        ],
    );

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let fk_added = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ForeignKeyAdded);
    assert!(fk_added.is_some());

    let diff = fk_added.unwrap();
    assert_eq!(diff.object_name, Some("fk_user".to_string()));
    assert!(diff.sql.contains("FOREIGN KEY"));
    assert!(diff.sql.contains("REFERENCES"));
}

#[test]
fn test_detect_removed_foreign_key() {
    let source_table = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
        ],
    );

    let mut target_table = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
        ],
    );
    target_table.foreign_keys = vec![create_foreign_key(
        "fk_old",
        vec!["user_id"],
        "users",
        vec!["id"],
    )];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let fk_removed = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ForeignKeyRemoved);
    assert!(fk_removed.is_some());
    assert!(fk_removed.unwrap().sql.contains("DROP FOREIGN KEY"));
}

// ============================================================================
// Unique Constraint Level Tests
// ============================================================================

#[test]
fn test_detect_added_unique_constraint() {
    let mut source_table = create_table(
        "users",
        vec![create_column("email", "VARCHAR(255)", false, false, 1)],
    );
    source_table.unique_constraints = vec![create_unique_constraint("uq_email", vec!["email"])];

    let target_table = create_table(
        "users",
        vec![create_column("email", "VARCHAR(255)", false, false, 1)],
    );

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let uc_added = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::UniqueConstraintAdded);
    assert!(uc_added.is_some());

    let diff = uc_added.unwrap();
    assert_eq!(diff.object_name, Some("uq_email".to_string()));
    assert!(diff.sql.contains("UNIQUE"));
}

#[test]
fn test_detect_removed_unique_constraint() {
    let source_table = create_table(
        "users",
        vec![create_column("email", "VARCHAR(255)", false, false, 1)],
    );

    let mut target_table = create_table(
        "users",
        vec![create_column("email", "VARCHAR(255)", false, false, 1)],
    );
    target_table.unique_constraints = vec![create_unique_constraint("uq_old", vec!["email"])];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let uc_removed = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::UniqueConstraintRemoved);
    assert!(uc_removed.is_some());
    assert!(uc_removed.unwrap().sql.contains("DROP CONSTRAINT"));
}

#[test]
fn test_composite_unique_constraint() {
    let mut source_table = create_table(
        "order_items",
        vec![
            create_column("order_id", "INT", false, false, 1),
            create_column("product_id", "INT", false, false, 2),
        ],
    );
    source_table.unique_constraints = vec![create_unique_constraint(
        "uq_order_product",
        vec!["order_id", "product_id"],
    )];

    let target_table = create_table(
        "order_items",
        vec![
            create_column("order_id", "INT", false, false, 1),
            create_column("product_id", "INT", false, false, 2),
        ],
    );

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let uc_added = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::UniqueConstraintAdded);
    assert!(uc_added.is_some());
    assert!(
        uc_added
            .unwrap()
            .source_def
            .as_ref()
            .unwrap()
            .contains("order_id")
    );
    assert!(
        uc_added
            .unwrap()
            .source_def
            .as_ref()
            .unwrap()
            .contains("product_id")
    );
}

// ============================================================================
// Complex Scenario Tests
// ============================================================================

#[test]
fn test_complex_schema_comparison() {
    // Source: newer schema with more features
    let mut source_users = create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("email", "VARCHAR(255)", false, false, 2),
            create_column("name", "VARCHAR(100)", true, false, 3),
            create_column("created_at", "TIMESTAMP", false, false, 4),
        ],
    );
    source_users.indexes = vec![create_index("idx_email", vec!["email"], true)];
    source_users.unique_constraints = vec![create_unique_constraint("uq_email", vec!["email"])];

    let mut source_orders = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
            create_column("total", "DECIMAL(10,2)", false, false, 3),
        ],
    );
    source_orders.foreign_keys = vec![create_foreign_key(
        "fk_user",
        vec!["user_id"],
        "users",
        vec!["id"],
    )];

    // Target: older schema
    let target_users = create_table(
        "users",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("email", "VARCHAR(200)", false, false, 2), // Different size
            create_column("old_field", "TEXT", true, false, 3),      // To be removed
        ],
    );

    let diffs = compare_schemas(
        &vec![source_users, source_orders],
        &vec![target_users],
        &MockSqlGen,
    );

    // Should detect:
    // 1. orders table added
    // 2. users.email modified (VARCHAR size)
    // 3. users.name added
    // 4. users.created_at added
    // 5. users.old_field removed
    // 6. idx_email index added
    // 7. uq_email unique constraint added

    assert!(
        diffs
            .iter()
            .any(|d| d.diff_type == DiffType::TableAdded && d.table_name == "orders")
    );
    assert!(
        diffs.iter().any(|d| d.diff_type == DiffType::ColumnModified
            && d.object_name == Some("email".to_string()))
    );
    assert!(
        diffs
            .iter()
            .any(|d| d.diff_type == DiffType::ColumnAdded
                && d.object_name == Some("name".to_string()))
    );
    assert!(
        diffs.iter().any(|d| d.diff_type == DiffType::ColumnAdded
            && d.object_name == Some("created_at".to_string()))
    );
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnRemoved
        && d.object_name == Some("old_field".to_string())));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::IndexAdded));
    assert!(
        diffs
            .iter()
            .any(|d| d.diff_type == DiffType::UniqueConstraintAdded)
    );
}

#[test]
fn test_diff_item_properties() {
    let source = vec![create_table(
        "test",
        vec![create_column("id", "INT", false, true, 1)],
    )];
    let target: Vec<TableSchema> = vec![];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    assert_eq!(diffs.len(), 1);
    let diff = &diffs[0];

    // Check all properties are set correctly
    assert!(!diff.id.is_empty());
    assert_eq!(diff.table_name, "test");
    assert!(diff.selected); // Should default to true
    assert!(!diff.sql.is_empty());
}

#[test]
fn test_multiple_tables_mixed_changes() {
    let source = vec![
        create_table("table_a", vec![create_column("id", "INT", false, true, 1)]),
        create_table("table_b", vec![create_column("id", "INT", false, true, 1)]),
        create_table(
            "table_new",
            vec![create_column("id", "INT", false, true, 1)],
        ),
    ];

    let target = vec![
        create_table("table_a", vec![create_column("id", "INT", false, true, 1)]),
        create_table(
            "table_b",
            vec![
                create_column("id", "INT", false, true, 1),
                create_column("extra", "TEXT", true, false, 2),
            ],
        ),
        create_table(
            "table_old",
            vec![create_column("id", "INT", false, true, 1)],
        ),
    ];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    // table_a: no changes
    // table_b: extra column removed
    // table_new: added
    // table_old: removed

    assert!(
        diffs
            .iter()
            .any(|d| d.diff_type == DiffType::TableAdded && d.table_name == "table_new")
    );
    assert!(
        diffs
            .iter()
            .any(|d| d.diff_type == DiffType::TableRemoved && d.table_name == "table_old")
    );
    assert!(
        diffs
            .iter()
            .any(|d| d.diff_type == DiffType::ColumnRemoved && d.table_name == "table_b")
    );

    // table_a should have no diffs
    assert!(!diffs.iter().any(|d| d.table_name == "table_a"));
}

// ============================================================================
// Model Tests
// ============================================================================

#[test]
fn test_db_type_default_ports() {
    assert_eq!(DbType::MySQL.default_port(), 3306);
    assert_eq!(DbType::PostgreSQL.default_port(), 5432);
    assert_eq!(DbType::MariaDB.default_port(), 3306);
}

#[test]
fn test_column_equality() {
    let col1 = create_column("id", "INT", false, true, 1);
    let col2 = create_column("id", "INT", false, true, 1);
    let col3 = create_column("id", "BIGINT", false, true, 1);

    assert_eq!(col1, col2);
    assert_ne!(col1, col3);
}

#[test]
fn test_diff_type_variants() {
    // Ensure all diff types are distinct
    let types = vec![
        DiffType::TableAdded,
        DiffType::TableRemoved,
        DiffType::ColumnAdded,
        DiffType::ColumnRemoved,
        DiffType::ColumnModified,
        DiffType::IndexAdded,
        DiffType::IndexRemoved,
        DiffType::IndexModified,
        DiffType::ForeignKeyAdded,
        DiffType::ForeignKeyRemoved,
        DiffType::ForeignKeyModified,
        DiffType::UniqueConstraintAdded,
        DiffType::UniqueConstraintRemoved,
        DiffType::UniqueConstraintModified,
    ];

    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            assert_ne!(types[i], types[j]);
        }
    }
}

// ============================================================================
// Error Tests
// ============================================================================

#[test]
fn test_app_error_connection_display() {
    let err = AppError::Connection("refused".to_string());
    assert_eq!(format!("{}", err), "Connection failed: refused");
}

#[test]
fn test_app_error_database_display() {
    let err = AppError::Database("timeout".to_string());
    assert_eq!(format!("{}", err), "Database error: timeout");
}

#[test]
fn test_app_error_storage_display() {
    let err = AppError::Storage("disk full".to_string());
    assert_eq!(format!("{}", err), "Storage error: disk full");
}

#[test]
fn test_app_error_ssh_tunnel_display() {
    let err = AppError::SshTunnel("auth failed".to_string());
    assert_eq!(format!("{}", err), "SSH tunnel error: auth failed");
}

#[test]
fn test_app_error_ssl_config_display() {
    let err = AppError::SslConfig("invalid cert".to_string());
    assert_eq!(format!("{}", err), "SSL configuration error: invalid cert");
}

#[test]
fn test_app_error_not_found_display() {
    let err = AppError::NotFound("record 42".to_string());
    assert_eq!(format!("{}", err), "Not found: record 42");
}

#[test]
fn test_app_error_validation_display() {
    let err = AppError::Validation("name required".to_string());
    assert_eq!(format!("{}", err), "Validation error: name required");
}

#[test]
fn test_app_error_internal_display() {
    let err = AppError::Internal("unexpected".to_string());
    assert_eq!(format!("{}", err), "Internal error: unexpected");
}

#[test]
fn test_app_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let app_err: AppError = io_err.into();
    match &app_err {
        AppError::Internal(msg) => assert!(msg.contains("file missing")),
        other => panic!("expected Internal, got {:?}", other),
    }
}

#[test]
fn test_app_error_from_anyhow_error() {
    let anyhow_err = anyhow::anyhow!("something went wrong");
    let app_err: AppError = anyhow_err.into();
    match &app_err {
        AppError::Internal(msg) => assert!(msg.contains("something went wrong")),
        other => panic!("expected Internal, got {:?}", other),
    }
}

#[test]
fn test_app_error_serialize_connection() {
    let err = AppError::Connection("refused".to_string());
    let json = serde_json::to_string(&err).unwrap();
    assert!(json.contains("\"type\":\"Connection\""));
    assert!(json.contains("\"message\":\"refused\""));
}

#[test]
fn test_app_error_serialize_all_variants() {
    let variants: Vec<(&str, AppError)> = vec![
        ("Connection", AppError::Connection("a".into())),
        ("Database", AppError::Database("b".into())),
        ("Storage", AppError::Storage("c".into())),
        ("SshTunnel", AppError::SshTunnel("d".into())),
        ("SslConfig", AppError::SslConfig("e".into())),
        ("NotFound", AppError::NotFound("f".into())),
        ("Validation", AppError::Validation("g".into())),
        ("Internal", AppError::Internal("h".into())),
    ];
    for (expected_type, err) in variants {
        let json = serde_json::to_string(&err).unwrap();
        assert!(
            json.contains(&format!("\"type\":\"{}\"", expected_type)),
            "Expected type '{}' in JSON: {}",
            expected_type,
            json
        );
    }
}

// ============================================================================
// DbType Display Tests
// ============================================================================

#[test]
fn test_db_type_display_mysql() {
    assert_eq!(DbType::MySQL.to_string(), "MySQL");
}

#[test]
fn test_db_type_display_postgresql() {
    assert_eq!(DbType::PostgreSQL.to_string(), "PostgreSQL");
}

#[test]
fn test_db_type_display_mariadb() {
    assert_eq!(DbType::MariaDB.to_string(), "MariaDB");
}

// ============================================================================
// DbType Serialization Tests
// ============================================================================

#[test]
fn test_db_type_serialize_lowercase() {
    // serde(rename_all = "lowercase") means MySQL -> "mysql"
    let json = serde_json::to_string(&DbType::MySQL).unwrap();
    assert_eq!(json, "\"mysql\"");

    let json = serde_json::to_string(&DbType::PostgreSQL).unwrap();
    assert_eq!(json, "\"postgresql\"");

    let json = serde_json::to_string(&DbType::MariaDB).unwrap();
    assert_eq!(json, "\"mariadb\"");
}

#[test]
fn test_db_type_deserialize_lowercase() {
    let mysql: DbType = serde_json::from_str("\"mysql\"").unwrap();
    assert_eq!(mysql, DbType::MySQL);

    let pg: DbType = serde_json::from_str("\"postgresql\"").unwrap();
    assert_eq!(pg, DbType::PostgreSQL);

    let maria: DbType = serde_json::from_str("\"mariadb\"").unwrap();
    assert_eq!(maria, DbType::MariaDB);
}

// ============================================================================
// Connection Serialization Tests
// ============================================================================

#[test]
fn test_connection_serialize_skips_password() {
    let conn = Connection {
        id: "conn-1".to_string(),
        name: "Test DB".to_string(),
        db_type: DbType::MySQL,
        host: "localhost".to_string(),
        port: 3306,
        username: "root".to_string(),
        password: "secret123".to_string(),
        database: "testdb".to_string(),
        ssh_config: None,
        ssl_config: None,
        created_at: "2025-01-01T00:00:00Z".to_string(),
        updated_at: "2025-01-01T00:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&conn).unwrap();
    // password should be skipped during serialization (skip_serializing)
    assert!(!json.contains("secret123"));
    assert!(json.contains("\"name\":\"Test DB\""));
    assert!(json.contains("\"db_type\":\"mysql\""));
}

#[test]
fn test_connection_deserialize_default_password() {
    let json = r#"{
        "id": "conn-1",
        "name": "Test",
        "db_type": "postgresql",
        "host": "localhost",
        "port": 5432,
        "username": "user",
        "database": "mydb",
        "created_at": "2025-01-01",
        "updated_at": "2025-01-01"
    }"#;

    let conn: Connection = serde_json::from_str(json).unwrap();
    assert_eq!(conn.password, ""); // default
    assert_eq!(conn.db_type, DbType::PostgreSQL);
    assert_eq!(conn.port, 5432);
}

#[test]
fn test_connection_input_serialize_deserialize() {
    let input = ConnectionInput {
        name: "Dev DB".to_string(),
        db_type: DbType::MariaDB,
        host: "db.example.com".to_string(),
        port: 3306,
        username: "admin".to_string(),
        password: "pass".to_string(),
        database: "app".to_string(),
        ssh_config: None,
        ssl_config: None,
    };

    let json = serde_json::to_string(&input).unwrap();
    let deserialized: ConnectionInput = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "Dev DB");
    assert_eq!(deserialized.db_type, DbType::MariaDB);
    assert_eq!(deserialized.host, "db.example.com");
}

#[test]
fn test_connection_input_deserialize_default_password() {
    let json = r#"{
        "name": "Test",
        "db_type": "mysql",
        "host": "localhost",
        "port": 3306,
        "username": "root",
        "database": "mydb"
    }"#;

    let input: ConnectionInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.password, ""); // serde(default)
}

// ============================================================================
// SshAuthMethod Serialization Tests
// ============================================================================

#[test]
fn test_ssh_auth_method_password_serialize() {
    let auth = SshAuthMethod::Password {
        password: "secret".to_string(),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("\"password\""));
    assert!(json.contains("secret"));

    let deserialized: SshAuthMethod = serde_json::from_str(&json).unwrap();
    match deserialized {
        SshAuthMethod::Password { password } => assert_eq!(password, "secret"),
        _ => panic!("expected Password variant"),
    }
}

#[test]
fn test_ssh_auth_method_private_key_serialize() {
    let auth = SshAuthMethod::PrivateKey {
        private_key_path: "/home/user/.ssh/id_rsa".to_string(),
        passphrase: Some("mypass".to_string()),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("id_rsa"));

    let deserialized: SshAuthMethod = serde_json::from_str(&json).unwrap();
    match deserialized {
        SshAuthMethod::PrivateKey {
            private_key_path,
            passphrase,
        } => {
            assert_eq!(private_key_path, "/home/user/.ssh/id_rsa");
            assert_eq!(passphrase, Some("mypass".to_string()));
        }
        _ => panic!("expected PrivateKey variant"),
    }
}

#[test]
fn test_ssh_auth_method_private_key_no_passphrase() {
    let auth = SshAuthMethod::PrivateKey {
        private_key_path: "/home/user/.ssh/id_ed25519".to_string(),
        passphrase: None,
    };
    let json = serde_json::to_string(&auth).unwrap();
    let deserialized: SshAuthMethod = serde_json::from_str(&json).unwrap();
    match deserialized {
        SshAuthMethod::PrivateKey { passphrase, .. } => assert_eq!(passphrase, None),
        _ => panic!("expected PrivateKey variant"),
    }
}

// ============================================================================
// SslConfig Serialization Tests
// ============================================================================

#[test]
fn test_ssl_config_serialize_deserialize() {
    let ssl = SslConfig {
        enabled: true,
        ca_cert_path: Some("/certs/ca.pem".to_string()),
        client_cert_path: Some("/certs/client.pem".to_string()),
        client_key_path: Some("/certs/client-key.pem".to_string()),
        verify_server: true,
    };

    let json = serde_json::to_string(&ssl).unwrap();
    let deserialized: SslConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.enabled);
    assert_eq!(deserialized.ca_cert_path, Some("/certs/ca.pem".to_string()));
    assert!(deserialized.verify_server);
}

#[test]
fn test_ssl_config_minimal() {
    let ssl = SslConfig {
        enabled: false,
        ca_cert_path: None,
        client_cert_path: None,
        client_key_path: None,
        verify_server: false,
    };

    let json = serde_json::to_string(&ssl).unwrap();
    let deserialized: SslConfig = serde_json::from_str(&json).unwrap();
    assert!(!deserialized.enabled);
    assert_eq!(deserialized.ca_cert_path, None);
}

// ============================================================================
// SshConfig Serialization Tests
// ============================================================================

#[test]
fn test_ssh_config_serialize_deserialize() {
    let ssh = SshConfig {
        enabled: true,
        host: "bastion.example.com".to_string(),
        port: 22,
        username: "jump".to_string(),
        auth_method: SshAuthMethod::Password {
            password: "sshpass".to_string(),
        },
    };

    let json = serde_json::to_string(&ssh).unwrap();
    let deserialized: SshConfig = serde_json::from_str(&json).unwrap();
    assert!(deserialized.enabled);
    assert_eq!(deserialized.host, "bastion.example.com");
    assert_eq!(deserialized.port, 22);
}

// ============================================================================
// Connection with SSH/SSL Serialization Tests
// ============================================================================

#[test]
fn test_connection_with_ssh_config_serialize() {
    let conn = Connection {
        id: "conn-2".to_string(),
        name: "SSH DB".to_string(),
        db_type: DbType::PostgreSQL,
        host: "db.internal".to_string(),
        port: 5432,
        username: "admin".to_string(),
        password: "pw".to_string(),
        database: "prod".to_string(),
        ssh_config: Some(SshConfig {
            enabled: true,
            host: "jump.example.com".to_string(),
            port: 22,
            username: "jumpuser".to_string(),
            auth_method: SshAuthMethod::Password {
                password: "jump_pass".to_string(),
            },
        }),
        ssl_config: None,
        created_at: "2025-01-01".to_string(),
        updated_at: "2025-01-01".to_string(),
    };

    let json = serde_json::to_string(&conn).unwrap();
    assert!(json.contains("ssh_config"));
    assert!(json.contains("jump.example.com"));
    // password of Connection itself is skipped
    assert!(!json.contains("\"pw\""));
}

// ============================================================================
// DiffType Serialization Tests (snake_case)
// ============================================================================

#[test]
fn test_diff_type_serialize_snake_case() {
    assert_eq!(
        serde_json::to_string(&DiffType::TableAdded).unwrap(),
        "\"table_added\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::TableRemoved).unwrap(),
        "\"table_removed\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::ColumnAdded).unwrap(),
        "\"column_added\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::ColumnRemoved).unwrap(),
        "\"column_removed\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::ColumnModified).unwrap(),
        "\"column_modified\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::IndexAdded).unwrap(),
        "\"index_added\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::IndexRemoved).unwrap(),
        "\"index_removed\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::IndexModified).unwrap(),
        "\"index_modified\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::ForeignKeyAdded).unwrap(),
        "\"foreign_key_added\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::ForeignKeyRemoved).unwrap(),
        "\"foreign_key_removed\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::UniqueConstraintAdded).unwrap(),
        "\"unique_constraint_added\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::UniqueConstraintRemoved).unwrap(),
        "\"unique_constraint_removed\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::ForeignKeyModified).unwrap(),
        "\"foreign_key_modified\""
    );
    assert_eq!(
        serde_json::to_string(&DiffType::UniqueConstraintModified).unwrap(),
        "\"unique_constraint_modified\""
    );
}

#[test]
fn test_diff_type_deserialize_snake_case() {
    let added: DiffType = serde_json::from_str("\"table_added\"").unwrap();
    assert_eq!(added, DiffType::TableAdded);

    let fk_removed: DiffType = serde_json::from_str("\"foreign_key_removed\"").unwrap();
    assert_eq!(fk_removed, DiffType::ForeignKeyRemoved);

    let uc_added: DiffType = serde_json::from_str("\"unique_constraint_added\"").unwrap();
    assert_eq!(uc_added, DiffType::UniqueConstraintAdded);
}

// ============================================================================
// DiffItem Serialization Tests
// ============================================================================

#[test]
fn test_diff_item_serialize_deserialize() {
    let item = DiffItem {
        id: "1".to_string(),
        diff_type: DiffType::ColumnAdded,
        table_name: "users".to_string(),
        object_name: Some("email".to_string()),
        source_def: Some("VARCHAR(255)".to_string()),
        target_def: None,
        sql: "ALTER TABLE users ADD COLUMN email VARCHAR(255)".to_string(),
        selected: true,
    };

    let json = serde_json::to_string(&item).unwrap();
    assert!(json.contains("\"diff_type\":\"column_added\""));
    assert!(json.contains("\"table_name\":\"users\""));
    assert!(json.contains("\"selected\":true"));

    let deserialized: DiffItem = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "1");
    assert_eq!(deserialized.diff_type, DiffType::ColumnAdded);
    assert_eq!(deserialized.table_name, "users");
    assert_eq!(deserialized.object_name, Some("email".to_string()));
    assert_eq!(deserialized.target_def, None);
    assert!(deserialized.selected);
}

#[test]
fn test_diff_item_with_none_fields() {
    let item = DiffItem {
        id: "5".to_string(),
        diff_type: DiffType::TableAdded,
        table_name: "orders".to_string(),
        object_name: None,
        source_def: None,
        target_def: None,
        sql: "CREATE TABLE orders".to_string(),
        selected: false,
    };

    let json = serde_json::to_string(&item).unwrap();
    let deserialized: DiffItem = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.object_name, None);
    assert_eq!(deserialized.source_def, None);
    assert_eq!(deserialized.target_def, None);
    assert!(!deserialized.selected);
}

// ============================================================================
// DiffResult Serialization Tests
// ============================================================================

#[test]
fn test_diff_result_serialize_deserialize() {
    let result = DiffResult {
        items: vec![
            DiffItem {
                id: "1".to_string(),
                diff_type: DiffType::TableAdded,
                table_name: "users".to_string(),
                object_name: None,
                source_def: Some("3 columns".to_string()),
                target_def: None,
                sql: "CREATE TABLE users".to_string(),
                selected: true,
            },
            DiffItem {
                id: "2".to_string(),
                diff_type: DiffType::ColumnRemoved,
                table_name: "orders".to_string(),
                object_name: Some("old_col".to_string()),
                source_def: None,
                target_def: Some("TEXT".to_string()),
                sql: "ALTER TABLE orders DROP COLUMN old_col".to_string(),
                selected: true,
            },
        ],
        source_tables: 5,
        target_tables: 3,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: DiffResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.items.len(), 2);
    assert_eq!(deserialized.source_tables, 5);
    assert_eq!(deserialized.target_tables, 3);
    assert_eq!(deserialized.items[0].diff_type, DiffType::TableAdded);
    assert_eq!(deserialized.items[1].diff_type, DiffType::ColumnRemoved);
}

#[test]
fn test_diff_result_empty() {
    let result = DiffResult {
        items: vec![],
        source_tables: 0,
        target_tables: 0,
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: DiffResult = serde_json::from_str(&json).unwrap();
    assert!(deserialized.items.is_empty());
    assert_eq!(deserialized.source_tables, 0);
    assert_eq!(deserialized.target_tables, 0);
}

// ============================================================================
// Schema Model Serialization Tests
// ============================================================================

#[test]
fn test_column_serialize_deserialize() {
    let col = Column {
        name: "email".to_string(),
        data_type: "VARCHAR(255)".to_string(),
        nullable: true,
        default_value: Some("''".to_string()),
        auto_increment: false,
        comment: Some("User email address".to_string()),
        ordinal_position: 3,
    };

    let json = serde_json::to_string(&col).unwrap();
    let deserialized: Column = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "email");
    assert_eq!(deserialized.data_type, "VARCHAR(255)");
    assert!(deserialized.nullable);
    assert_eq!(deserialized.default_value, Some("''".to_string()));
    assert!(!deserialized.auto_increment);
    assert_eq!(deserialized.comment, Some("User email address".to_string()));
    assert_eq!(deserialized.ordinal_position, 3);
}

#[test]
fn test_column_serialize_none_optionals() {
    let col = Column {
        name: "id".to_string(),
        data_type: "INT".to_string(),
        nullable: false,
        default_value: None,
        auto_increment: true,
        comment: None,
        ordinal_position: 1,
    };

    let json = serde_json::to_string(&col).unwrap();
    let deserialized: Column = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.default_value, None);
    assert_eq!(deserialized.comment, None);
    assert!(deserialized.auto_increment);
}

#[test]
fn test_primary_key_serialize_deserialize() {
    let pk = PrimaryKey {
        name: Some("pk_users".to_string()),
        columns: vec!["id".to_string()],
    };

    let json = serde_json::to_string(&pk).unwrap();
    let deserialized: PrimaryKey = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, Some("pk_users".to_string()));
    assert_eq!(deserialized.columns, vec!["id".to_string()]);
}

#[test]
fn test_primary_key_composite() {
    let pk = PrimaryKey {
        name: None,
        columns: vec!["order_id".to_string(), "product_id".to_string()],
    };

    let json = serde_json::to_string(&pk).unwrap();
    let deserialized: PrimaryKey = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, None);
    assert_eq!(deserialized.columns.len(), 2);
}

#[test]
fn test_index_serialize_deserialize() {
    let idx = Index {
        name: "idx_email".to_string(),
        columns: vec!["email".to_string()],
        unique: true,
        index_type: "BTREE".to_string(),
    };

    let json = serde_json::to_string(&idx).unwrap();
    let deserialized: Index = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "idx_email");
    assert!(deserialized.unique);
    assert_eq!(deserialized.index_type, "BTREE");
}

#[test]
fn test_index_multi_column() {
    let idx = Index {
        name: "idx_composite".to_string(),
        columns: vec!["last_name".to_string(), "first_name".to_string()],
        unique: false,
        index_type: "HASH".to_string(),
    };

    let json = serde_json::to_string(&idx).unwrap();
    let deserialized: Index = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.columns.len(), 2);
    assert!(!deserialized.unique);
    assert_eq!(deserialized.index_type, "HASH");
}

#[test]
fn test_foreign_key_serialize_deserialize() {
    let fk = ForeignKey {
        name: "fk_order_user".to_string(),
        columns: vec!["user_id".to_string()],
        ref_table: "users".to_string(),
        ref_columns: vec!["id".to_string()],
        on_delete: "CASCADE".to_string(),
        on_update: "SET NULL".to_string(),
    };

    let json = serde_json::to_string(&fk).unwrap();
    let deserialized: ForeignKey = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "fk_order_user");
    assert_eq!(deserialized.columns, vec!["user_id"]);
    assert_eq!(deserialized.ref_table, "users");
    assert_eq!(deserialized.ref_columns, vec!["id"]);
    assert_eq!(deserialized.on_delete, "CASCADE");
    assert_eq!(deserialized.on_update, "SET NULL");
}

#[test]
fn test_foreign_key_composite() {
    let fk = ForeignKey {
        name: "fk_composite".to_string(),
        columns: vec!["order_id".to_string(), "product_id".to_string()],
        ref_table: "order_products".to_string(),
        ref_columns: vec!["oid".to_string(), "pid".to_string()],
        on_delete: "RESTRICT".to_string(),
        on_update: "NO ACTION".to_string(),
    };

    let json = serde_json::to_string(&fk).unwrap();
    let deserialized: ForeignKey = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.columns.len(), 2);
    assert_eq!(deserialized.ref_columns.len(), 2);
}

#[test]
fn test_unique_constraint_serialize_deserialize() {
    let uc = UniqueConstraint {
        name: "uq_email".to_string(),
        columns: vec!["email".to_string()],
    };

    let json = serde_json::to_string(&uc).unwrap();
    let deserialized: UniqueConstraint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "uq_email");
    assert_eq!(deserialized.columns, vec!["email"]);
}

#[test]
fn test_unique_constraint_composite_serialize() {
    let uc = UniqueConstraint {
        name: "uq_name_email".to_string(),
        columns: vec!["first_name".to_string(), "email".to_string()],
    };

    let json = serde_json::to_string(&uc).unwrap();
    let deserialized: UniqueConstraint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.columns.len(), 2);
}

#[test]
fn test_table_schema_serialize_deserialize() {
    let table = TableSchema {
        name: "users".to_string(),
        columns: vec![
            Column {
                name: "id".to_string(),
                data_type: "INT".to_string(),
                nullable: false,
                default_value: None,
                auto_increment: true,
                comment: None,
                ordinal_position: 1,
            },
            Column {
                name: "email".to_string(),
                data_type: "VARCHAR(255)".to_string(),
                nullable: false,
                default_value: None,
                auto_increment: false,
                comment: Some("email".to_string()),
                ordinal_position: 2,
            },
        ],
        primary_key: Some(PrimaryKey {
            name: Some("PRIMARY".to_string()),
            columns: vec!["id".to_string()],
        }),
        indexes: vec![Index {
            name: "idx_email".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
            index_type: "BTREE".to_string(),
        }],
        foreign_keys: vec![],
        unique_constraints: vec![UniqueConstraint {
            name: "uq_email".to_string(),
            columns: vec!["email".to_string()],
        }],
    };

    let json = serde_json::to_string(&table).unwrap();
    let deserialized: TableSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "users");
    assert_eq!(deserialized.columns.len(), 2);
    assert!(deserialized.primary_key.is_some());
    assert_eq!(deserialized.indexes.len(), 1);
    assert!(deserialized.foreign_keys.is_empty());
    assert_eq!(deserialized.unique_constraints.len(), 1);
}

#[test]
fn test_table_schema_minimal() {
    let table = TableSchema {
        name: "empty_table".to_string(),
        columns: vec![],
        primary_key: None,
        indexes: vec![],
        foreign_keys: vec![],
        unique_constraints: vec![],
    };

    let json = serde_json::to_string(&table).unwrap();
    let deserialized: TableSchema = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "empty_table");
    assert!(deserialized.columns.is_empty());
    assert!(deserialized.primary_key.is_none());
}

// ============================================================================
// Schema Model Equality Tests
// ============================================================================

#[test]
fn test_index_equality() {
    let idx1 = create_index("idx_a", vec!["col1", "col2"], false);
    let idx2 = create_index("idx_a", vec!["col1", "col2"], false);
    let idx3 = create_index("idx_a", vec!["col1"], false); // different columns
    let idx4 = create_index("idx_a", vec!["col1", "col2"], true); // different uniqueness

    assert_eq!(idx1, idx2);
    assert_ne!(idx1, idx3);
    assert_ne!(idx1, idx4);
}

#[test]
fn test_foreign_key_equality() {
    let fk1 = create_foreign_key("fk_a", vec!["user_id"], "users", vec!["id"]);
    let fk2 = create_foreign_key("fk_a", vec!["user_id"], "users", vec!["id"]);
    let fk3 = create_foreign_key("fk_a", vec!["user_id"], "accounts", vec!["id"]); // different ref table

    assert_eq!(fk1, fk2);
    assert_ne!(fk1, fk3);
}

#[test]
fn test_unique_constraint_equality() {
    let uc1 = create_unique_constraint("uq_a", vec!["col1"]);
    let uc2 = create_unique_constraint("uq_a", vec!["col1"]);
    let uc3 = create_unique_constraint("uq_b", vec!["col1"]); // different name

    assert_eq!(uc1, uc2);
    assert_ne!(uc1, uc3);
}

#[test]
fn test_primary_key_equality() {
    let pk1 = PrimaryKey {
        name: Some("pk_users".to_string()),
        columns: vec!["id".to_string()],
    };
    let pk2 = PrimaryKey {
        name: Some("pk_users".to_string()),
        columns: vec!["id".to_string()],
    };
    let pk3 = PrimaryKey {
        name: None,
        columns: vec!["id".to_string()],
    };

    assert_eq!(pk1, pk2);
    assert_ne!(pk1, pk3);
}

// ============================================================================
// Comparator Edge Case Tests
// ============================================================================

#[test]
fn test_foreign_key_same_name_different_content() {
    // The comparator now detects FK content changes as ForeignKeyModified.
    let mut source_table = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
        ],
    );
    source_table.foreign_keys = vec![create_foreign_key(
        "fk_user",
        vec!["user_id"],
        "users",
        vec!["id"],
    )];

    let mut target_table = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
        ],
    );
    // Same FK name "fk_user" but pointing to a different table
    target_table.foreign_keys = vec![create_foreign_key(
        "fk_user",
        vec!["user_id"],
        "accounts",
        vec!["account_id"],
    )];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].diff_type, DiffType::ForeignKeyModified);
    assert_eq!(diffs[0].object_name, Some("fk_user".to_string()));
    assert!(diffs[0].sql.contains("DROP FOREIGN KEY"));
    assert!(diffs[0].sql.contains("FOREIGN KEY"));
    assert!(diffs[0].sql.contains("REFERENCES"));
}

#[test]
fn test_detect_modified_foreign_key_on_delete() {
    let mut source_table = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
        ],
    );
    source_table.foreign_keys = vec![ForeignKey {
        name: "fk_user".to_string(),
        columns: vec!["user_id".to_string()],
        ref_table: "users".to_string(),
        ref_columns: vec!["id".to_string()],
        on_delete: "SET NULL".to_string(),
        on_update: "CASCADE".to_string(),
    }];

    let mut target_table = create_table(
        "orders",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("user_id", "INT", false, false, 2),
        ],
    );
    target_table.foreign_keys = vec![ForeignKey {
        name: "fk_user".to_string(),
        columns: vec!["user_id".to_string()],
        ref_table: "users".to_string(),
        ref_columns: vec!["id".to_string()],
        on_delete: "CASCADE".to_string(),
        on_update: "CASCADE".to_string(),
    }];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].diff_type, DiffType::ForeignKeyModified);
}

#[test]
fn test_detect_modified_unique_constraint() {
    let mut source_table = create_table(
        "users",
        vec![
            create_column("email", "VARCHAR(255)", false, false, 1),
            create_column("phone", "VARCHAR(20)", false, false, 2),
        ],
    );
    source_table.unique_constraints = vec![create_unique_constraint(
        "uq_contact",
        vec!["email", "phone"],
    )];

    let mut target_table = create_table(
        "users",
        vec![
            create_column("email", "VARCHAR(255)", false, false, 1),
            create_column("phone", "VARCHAR(20)", false, false, 2),
        ],
    );
    target_table.unique_constraints = vec![create_unique_constraint("uq_contact", vec!["email"])];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].diff_type, DiffType::UniqueConstraintModified);
    assert_eq!(diffs[0].object_name, Some("uq_contact".to_string()));
    assert!(diffs[0].sql.contains("DROP CONSTRAINT"));
    assert!(diffs[0].sql.contains("UNIQUE"));
}

#[test]
fn test_id_counter_increments_across_all_diff_types() {
    // Build a scenario that produces multiple diff types and verify IDs increment sequentially.
    let mut source_table = create_table(
        "items",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("new_col", "TEXT", true, false, 2),
        ],
    );
    source_table.indexes = vec![create_index("idx_new", vec!["new_col"], false)];
    source_table.foreign_keys = vec![create_foreign_key(
        "fk_new",
        vec!["id"],
        "other",
        vec!["id"],
    )];
    source_table.unique_constraints = vec![create_unique_constraint("uq_new", vec!["new_col"])];

    // A new table that doesn't exist in target
    let new_table = create_table(
        "brand_new",
        vec![create_column("id", "INT", false, true, 1)],
    );

    let mut target_table = create_table(
        "items",
        vec![
            create_column("id", "INT", false, true, 1),
            create_column("old_col", "TEXT", true, false, 2),
        ],
    );
    target_table.indexes = vec![create_index("idx_old", vec!["old_col"], false)];
    target_table.foreign_keys = vec![create_foreign_key(
        "fk_old",
        vec!["id"],
        "legacy",
        vec!["id"],
    )];
    target_table.unique_constraints = vec![create_unique_constraint("uq_old", vec!["old_col"])];

    // Also a table in target only, to be removed
    let old_table = create_table(
        "deprecated",
        vec![create_column("id", "INT", false, true, 1)],
    );

    let diffs = compare_schemas(
        &vec![new_table, source_table],
        &vec![old_table, target_table],
        &MockSqlGen,
    );

    // Should have at least: TableAdded (brand_new), TableRemoved (deprecated),
    // ColumnAdded (new_col), ColumnRemoved (old_col),
    // IndexAdded (idx_new), IndexRemoved (idx_old),
    // ForeignKeyAdded (fk_new), ForeignKeyRemoved (fk_old),
    // UniqueConstraintAdded (uq_new), UniqueConstraintRemoved (uq_old)
    assert!(
        diffs.len() >= 10,
        "Expected at least 10 diffs, got {}",
        diffs.len()
    );

    // Check all IDs are unique
    let ids: Vec<&str> = diffs.iter().map(|d| d.id.as_str()).collect();
    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(
        ids.len(),
        unique_ids.len(),
        "IDs should all be unique: {:?}",
        ids
    );

    // Check IDs are sequential integers starting from 1
    for (i, diff) in diffs.iter().enumerate() {
        let expected_id = (i + 1).to_string();
        assert_eq!(
            diff.id, expected_id,
            "Expected diff #{} to have id '{}', got '{}'",
            i, expected_id, diff.id
        );
    }
}

#[test]
fn test_modified_column_auto_increment_change() {
    let source = vec![create_table(
        "users",
        vec![create_column("id", "INT", false, true, 1)], // auto_increment = true
    )];

    let target = vec![create_table(
        "users",
        vec![create_column("id", "INT", false, false, 1)], // auto_increment = false
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);
    let col_modified = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(
        col_modified.is_some(),
        "Should detect auto_increment change"
    );
}

#[test]
fn test_modified_column_comment_change() {
    let source = vec![create_table(
        "users",
        vec![Column {
            name: "name".to_string(),
            data_type: "VARCHAR(255)".to_string(),
            nullable: false,
            default_value: None,
            auto_increment: false,
            comment: Some("full name".to_string()),
            ordinal_position: 1,
        }],
    )];

    let target = vec![create_table(
        "users",
        vec![Column {
            name: "name".to_string(),
            data_type: "VARCHAR(255)".to_string(),
            nullable: false,
            default_value: None,
            auto_increment: false,
            comment: None,
            ordinal_position: 1,
        }],
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);
    let col_modified = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(col_modified.is_some(), "Should detect comment change");
}

#[test]
fn test_modified_column_ordinal_position_change() {
    let source = vec![create_table(
        "users",
        vec![create_column("name", "VARCHAR(255)", false, false, 1)],
    )];

    let target = vec![create_table(
        "users",
        vec![create_column("name", "VARCHAR(255)", false, false, 5)], // different position
    )];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);
    let col_modified = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(
        col_modified.is_some(),
        "Should detect ordinal_position change"
    );
}

#[test]
fn test_index_uniqueness_change_detected_as_modified() {
    let mut source_table = create_table(
        "users",
        vec![create_column("email", "VARCHAR(255)", false, false, 1)],
    );
    source_table.indexes = vec![create_index("idx_email", vec!["email"], true)]; // unique

    let mut target_table = create_table(
        "users",
        vec![create_column("email", "VARCHAR(255)", false, false, 1)],
    );
    target_table.indexes = vec![create_index("idx_email", vec!["email"], false)]; // not unique

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let idx_modified = diffs
        .iter()
        .find(|d| d.diff_type == DiffType::IndexModified);
    assert!(
        idx_modified.is_some(),
        "Should detect uniqueness change as index modification"
    );
    // Modified index SQL should contain both DROP and CREATE
    let sql = &idx_modified.unwrap().sql;
    assert!(sql.contains("DROP INDEX"));
    assert!(sql.contains("CREATE UNIQUE INDEX"));
}

#[test]
fn test_table_schema_equality() {
    let t1 = create_table("users", vec![create_column("id", "INT", false, true, 1)]);
    let t2 = create_table("users", vec![create_column("id", "INT", false, true, 1)]);
    let t3 = create_table("accounts", vec![create_column("id", "INT", false, true, 1)]);

    assert_eq!(t1, t2);
    assert_ne!(t1, t3);
}

#[test]
fn test_column_with_default_value_serialize() {
    let col = create_column_with_default("status", "INT", false, "0", 1);
    let json = serde_json::to_string(&col).unwrap();
    let deserialized: Column = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.default_value, Some("0".to_string()));
    assert!(!deserialized.auto_increment);
}
