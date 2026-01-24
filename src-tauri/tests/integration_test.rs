use database_structure_sync_lib::models::*;
use database_structure_sync_lib::diff::compare_schemas;
use database_structure_sync_lib::db::SqlGenerator;

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
        format!("ALTER TABLE \"{}\" ADD COLUMN \"{}\" {}", table_name, column.name, column.data_type)
    }

    fn generate_drop_column(&self, table_name: &str, column_name: &str) -> String {
        format!("ALTER TABLE \"{}\" DROP COLUMN \"{}\"", table_name, column_name)
    }

    fn generate_modify_column(&self, table_name: &str, column: &Column) -> String {
        format!("ALTER TABLE \"{}\" MODIFY COLUMN \"{}\" {}", table_name, column.name, column.data_type)
    }

    fn generate_add_index(&self, table_name: &str, index: &Index) -> String {
        let idx_type = if index.unique { "UNIQUE INDEX" } else { "INDEX" };
        format!("CREATE {} \"{}\" ON \"{}\" ({})", idx_type, index.name, table_name, index.columns.join(", "))
    }

    fn generate_drop_index(&self, table_name: &str, index_name: &str) -> String {
        format!("DROP INDEX \"{}\" ON \"{}\"", index_name, table_name)
    }

    fn generate_add_foreign_key(&self, table_name: &str, fk: &ForeignKey) -> String {
        format!(
            "ALTER TABLE \"{}\" ADD CONSTRAINT \"{}\" FOREIGN KEY ({}) REFERENCES \"{}\" ({})",
            table_name, fk.name, fk.columns.join(", "), fk.ref_table, fk.ref_columns.join(", ")
        )
    }

    fn generate_drop_foreign_key(&self, table_name: &str, fk_name: &str) -> String {
        format!("ALTER TABLE \"{}\" DROP FOREIGN KEY \"{}\"", table_name, fk_name)
    }

    fn generate_add_unique(&self, table_name: &str, unique: &UniqueConstraint) -> String {
        format!("ALTER TABLE \"{}\" ADD CONSTRAINT \"{}\" UNIQUE ({})", table_name, unique.name, unique.columns.join(", "))
    }

    fn generate_drop_unique(&self, table_name: &str, unique_name: &str) -> String {
        format!("ALTER TABLE \"{}\" DROP CONSTRAINT \"{}\"", table_name, unique_name)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_column(name: &str, data_type: &str, nullable: bool, auto_increment: bool, position: u32) -> Column {
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

fn create_column_with_default(name: &str, data_type: &str, nullable: bool, default: &str, position: u32) -> Column {
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

fn create_foreign_key(name: &str, columns: Vec<&str>, ref_table: &str, ref_columns: Vec<&str>) -> ForeignKey {
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
    let source = vec![create_table("users", vec![
        create_column("id", "INT", false, true, 1),
    ])];
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
    let source = vec![create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("email", "VARCHAR(255)", true, false, 2),
    ])];

    let target = vec![create_table("users", vec![
        create_column("id", "INT", false, true, 1),
    ])];

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
    let source = vec![create_table("users", vec![
        create_column("id", "INT", false, true, 1),
    ])];

    let target = vec![create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("old_field", "TEXT", true, false, 2),
    ])];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_removed = diffs.iter().find(|d| d.diff_type == DiffType::ColumnRemoved);
    assert!(column_removed.is_some());

    let diff = column_removed.unwrap();
    assert_eq!(diff.table_name, "users");
    assert_eq!(diff.object_name, Some("old_field".to_string()));
    assert!(diff.sql.contains("DROP COLUMN"));
}

#[test]
fn test_detect_modified_column_type() {
    let source = vec![create_table("users", vec![
        create_column("name", "VARCHAR(255)", false, false, 1),
    ])];

    let target = vec![create_table("users", vec![
        create_column("name", "VARCHAR(100)", false, false, 1),
    ])];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_modified = diffs.iter().find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(column_modified.is_some());

    let diff = column_modified.unwrap();
    assert_eq!(diff.source_def, Some("VARCHAR(255)".to_string()));
    assert_eq!(diff.target_def, Some("VARCHAR(100)".to_string()));
}

#[test]
fn test_detect_modified_column_nullable() {
    let source = vec![create_table("users", vec![
        create_column("email", "VARCHAR(255)", false, false, 1),  // NOT NULL
    ])];

    let target = vec![create_table("users", vec![
        create_column("email", "VARCHAR(255)", true, false, 1),   // NULL
    ])];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_modified = diffs.iter().find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(column_modified.is_some());
}

#[test]
fn test_detect_modified_column_default() {
    let source = vec![create_table("users", vec![
        create_column_with_default("status", "INT", false, "1", 1),
    ])];

    let target = vec![create_table("users", vec![
        create_column_with_default("status", "INT", false, "0", 1),
    ])];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    let column_modified = diffs.iter().find(|d| d.diff_type == DiffType::ColumnModified);
    assert!(column_modified.is_some());
}

#[test]
fn test_multiple_column_changes() {
    let source = vec![create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("name", "VARCHAR(255)", false, false, 2),
        create_column("new_col", "TEXT", true, false, 3),
    ])];

    let target = vec![create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("name", "VARCHAR(100)", false, false, 2),  // Modified
        create_column("old_col", "TEXT", true, false, 3),        // To be removed
    ])];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnAdded && d.object_name == Some("new_col".to_string())));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnRemoved && d.object_name == Some("old_col".to_string())));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnModified && d.object_name == Some("name".to_string())));
}

// ============================================================================
// Index Level Tests
// ============================================================================

#[test]
fn test_detect_added_index() {
    let mut source_table = create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("email", "VARCHAR(255)", true, false, 2),
    ]);
    source_table.indexes = vec![create_index("idx_email", vec!["email"], false)];

    let target_table = create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("email", "VARCHAR(255)", true, false, 2),
    ]);

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let index_added = diffs.iter().find(|d| d.diff_type == DiffType::IndexAdded);
    assert!(index_added.is_some());

    let diff = index_added.unwrap();
    assert_eq!(diff.object_name, Some("idx_email".to_string()));
    assert!(diff.sql.contains("CREATE INDEX"));
}

#[test]
fn test_detect_removed_index() {
    let source_table = create_table("users", vec![
        create_column("id", "INT", false, true, 1),
    ]);

    let mut target_table = create_table("users", vec![
        create_column("id", "INT", false, true, 1),
    ]);
    target_table.indexes = vec![create_index("idx_old", vec!["id"], false)];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let index_removed = diffs.iter().find(|d| d.diff_type == DiffType::IndexRemoved);
    assert!(index_removed.is_some());
    assert!(index_removed.unwrap().sql.contains("DROP INDEX"));
}

#[test]
fn test_detect_modified_index() {
    let mut source_table = create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("email", "VARCHAR(255)", true, false, 2),
        create_column("name", "VARCHAR(255)", true, false, 3),
    ]);
    source_table.indexes = vec![create_index("idx_search", vec!["email", "name"], false)];

    let mut target_table = create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("email", "VARCHAR(255)", true, false, 2),
        create_column("name", "VARCHAR(255)", true, false, 3),
    ]);
    target_table.indexes = vec![create_index("idx_search", vec!["email"], false)];  // Different columns

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let index_modified = diffs.iter().find(|d| d.diff_type == DiffType::IndexModified);
    assert!(index_modified.is_some());

    let diff = index_modified.unwrap();
    assert!(diff.sql.contains("DROP INDEX"));
    assert!(diff.sql.contains("CREATE INDEX"));
}

#[test]
fn test_detect_unique_index_added() {
    let mut source_table = create_table("users", vec![
        create_column("email", "VARCHAR(255)", false, false, 1),
    ]);
    source_table.indexes = vec![create_index("idx_email_unique", vec!["email"], true)];

    let target_table = create_table("users", vec![
        create_column("email", "VARCHAR(255)", false, false, 1),
    ]);

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
    let mut source_table = create_table("orders", vec![
        create_column("id", "INT", false, true, 1),
        create_column("user_id", "INT", false, false, 2),
    ]);
    source_table.foreign_keys = vec![create_foreign_key("fk_user", vec!["user_id"], "users", vec!["id"])];

    let target_table = create_table("orders", vec![
        create_column("id", "INT", false, true, 1),
        create_column("user_id", "INT", false, false, 2),
    ]);

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let fk_added = diffs.iter().find(|d| d.diff_type == DiffType::ForeignKeyAdded);
    assert!(fk_added.is_some());

    let diff = fk_added.unwrap();
    assert_eq!(diff.object_name, Some("fk_user".to_string()));
    assert!(diff.sql.contains("FOREIGN KEY"));
    assert!(diff.sql.contains("REFERENCES"));
}

#[test]
fn test_detect_removed_foreign_key() {
    let source_table = create_table("orders", vec![
        create_column("id", "INT", false, true, 1),
        create_column("user_id", "INT", false, false, 2),
    ]);

    let mut target_table = create_table("orders", vec![
        create_column("id", "INT", false, true, 1),
        create_column("user_id", "INT", false, false, 2),
    ]);
    target_table.foreign_keys = vec![create_foreign_key("fk_old", vec!["user_id"], "users", vec!["id"])];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let fk_removed = diffs.iter().find(|d| d.diff_type == DiffType::ForeignKeyRemoved);
    assert!(fk_removed.is_some());
    assert!(fk_removed.unwrap().sql.contains("DROP FOREIGN KEY"));
}

// ============================================================================
// Unique Constraint Level Tests
// ============================================================================

#[test]
fn test_detect_added_unique_constraint() {
    let mut source_table = create_table("users", vec![
        create_column("email", "VARCHAR(255)", false, false, 1),
    ]);
    source_table.unique_constraints = vec![create_unique_constraint("uq_email", vec!["email"])];

    let target_table = create_table("users", vec![
        create_column("email", "VARCHAR(255)", false, false, 1),
    ]);

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let uc_added = diffs.iter().find(|d| d.diff_type == DiffType::UniqueConstraintAdded);
    assert!(uc_added.is_some());

    let diff = uc_added.unwrap();
    assert_eq!(diff.object_name, Some("uq_email".to_string()));
    assert!(diff.sql.contains("UNIQUE"));
}

#[test]
fn test_detect_removed_unique_constraint() {
    let source_table = create_table("users", vec![
        create_column("email", "VARCHAR(255)", false, false, 1),
    ]);

    let mut target_table = create_table("users", vec![
        create_column("email", "VARCHAR(255)", false, false, 1),
    ]);
    target_table.unique_constraints = vec![create_unique_constraint("uq_old", vec!["email"])];

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let uc_removed = diffs.iter().find(|d| d.diff_type == DiffType::UniqueConstraintRemoved);
    assert!(uc_removed.is_some());
    assert!(uc_removed.unwrap().sql.contains("DROP CONSTRAINT"));
}

#[test]
fn test_composite_unique_constraint() {
    let mut source_table = create_table("order_items", vec![
        create_column("order_id", "INT", false, false, 1),
        create_column("product_id", "INT", false, false, 2),
    ]);
    source_table.unique_constraints = vec![create_unique_constraint("uq_order_product", vec!["order_id", "product_id"])];

    let target_table = create_table("order_items", vec![
        create_column("order_id", "INT", false, false, 1),
        create_column("product_id", "INT", false, false, 2),
    ]);

    let diffs = compare_schemas(&vec![source_table], &vec![target_table], &MockSqlGen);

    let uc_added = diffs.iter().find(|d| d.diff_type == DiffType::UniqueConstraintAdded);
    assert!(uc_added.is_some());
    assert!(uc_added.unwrap().source_def.as_ref().unwrap().contains("order_id"));
    assert!(uc_added.unwrap().source_def.as_ref().unwrap().contains("product_id"));
}

// ============================================================================
// Complex Scenario Tests
// ============================================================================

#[test]
fn test_complex_schema_comparison() {
    // Source: newer schema with more features
    let mut source_users = create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("email", "VARCHAR(255)", false, false, 2),
        create_column("name", "VARCHAR(100)", true, false, 3),
        create_column("created_at", "TIMESTAMP", false, false, 4),
    ]);
    source_users.indexes = vec![create_index("idx_email", vec!["email"], true)];
    source_users.unique_constraints = vec![create_unique_constraint("uq_email", vec!["email"])];

    let mut source_orders = create_table("orders", vec![
        create_column("id", "INT", false, true, 1),
        create_column("user_id", "INT", false, false, 2),
        create_column("total", "DECIMAL(10,2)", false, false, 3),
    ]);
    source_orders.foreign_keys = vec![create_foreign_key("fk_user", vec!["user_id"], "users", vec!["id"])];

    // Target: older schema
    let target_users = create_table("users", vec![
        create_column("id", "INT", false, true, 1),
        create_column("email", "VARCHAR(200)", false, false, 2),  // Different size
        create_column("old_field", "TEXT", true, false, 3),       // To be removed
    ]);

    let diffs = compare_schemas(
        &vec![source_users, source_orders],
        &vec![target_users],
        &MockSqlGen
    );

    // Should detect:
    // 1. orders table added
    // 2. users.email modified (VARCHAR size)
    // 3. users.name added
    // 4. users.created_at added
    // 5. users.old_field removed
    // 6. idx_email index added
    // 7. uq_email unique constraint added

    assert!(diffs.iter().any(|d| d.diff_type == DiffType::TableAdded && d.table_name == "orders"));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnModified && d.object_name == Some("email".to_string())));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnAdded && d.object_name == Some("name".to_string())));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnAdded && d.object_name == Some("created_at".to_string())));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnRemoved && d.object_name == Some("old_field".to_string())));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::IndexAdded));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::UniqueConstraintAdded));
}

#[test]
fn test_diff_item_properties() {
    let source = vec![create_table("test", vec![
        create_column("id", "INT", false, true, 1),
    ])];
    let target: Vec<TableSchema> = vec![];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    assert_eq!(diffs.len(), 1);
    let diff = &diffs[0];

    // Check all properties are set correctly
    assert!(!diff.id.is_empty());
    assert_eq!(diff.table_name, "test");
    assert!(diff.selected);  // Should default to true
    assert!(!diff.sql.is_empty());
}

#[test]
fn test_multiple_tables_mixed_changes() {
    let source = vec![
        create_table("table_a", vec![create_column("id", "INT", false, true, 1)]),
        create_table("table_b", vec![create_column("id", "INT", false, true, 1)]),
        create_table("table_new", vec![create_column("id", "INT", false, true, 1)]),
    ];

    let target = vec![
        create_table("table_a", vec![create_column("id", "INT", false, true, 1)]),
        create_table("table_b", vec![
            create_column("id", "INT", false, true, 1),
            create_column("extra", "TEXT", true, false, 2),
        ]),
        create_table("table_old", vec![create_column("id", "INT", false, true, 1)]),
    ];

    let diffs = compare_schemas(&source, &target, &MockSqlGen);

    // table_a: no changes
    // table_b: extra column removed
    // table_new: added
    // table_old: removed

    assert!(diffs.iter().any(|d| d.diff_type == DiffType::TableAdded && d.table_name == "table_new"));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::TableRemoved && d.table_name == "table_old"));
    assert!(diffs.iter().any(|d| d.diff_type == DiffType::ColumnRemoved && d.table_name == "table_b"));

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
        DiffType::UniqueConstraintAdded,
        DiffType::UniqueConstraintRemoved,
    ];

    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            assert_ne!(types[i], types[j]);
        }
    }
}
