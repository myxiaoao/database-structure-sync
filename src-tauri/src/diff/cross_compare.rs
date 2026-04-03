use std::collections::HashMap;

use crate::db::SqlGenerator;
use crate::models::*;
use crate::types::{TypeMapper, TypeMapping};

/// Compare schemas across different database types using canonical type mapping.
pub fn compare_schemas_cross(
    source: &[TableSchema],
    target: &[TableSchema],
    sql_gen: &dyn SqlGenerator,
    source_mapper: &dyn TypeMapper,
    target_mapper: &dyn TypeMapper,
) -> Vec<DiffItem> {
    let mut diffs = Vec::new();
    let mut id_counter: u32 = 0;

    let source_map: HashMap<&str, &TableSchema> =
        source.iter().map(|t| (t.name.as_str(), t)).collect();
    let target_map: HashMap<&str, &TableSchema> =
        target.iter().map(|t| (t.name.as_str(), t)).collect();

    // Added tables
    for table in source {
        if !target_map.contains_key(table.name.as_str()) {
            id_counter += 1;
            let (mapped_table, warnings) = map_table_columns(table, source_mapper, target_mapper);
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::TableAdded,
                table_name: table.name.clone(),
                object_name: None,
                source_def: Some(format!("{} columns", table.columns.len())),
                target_def: None,
                sql: sql_gen.generate_create_table(&mapped_table),
                selected: true,
                warnings,
            });
        }
    }

    // Removed tables
    for table in target {
        if !source_map.contains_key(table.name.as_str()) {
            id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::TableRemoved,
                table_name: table.name.clone(),
                object_name: None,
                source_def: None,
                target_def: Some(format!("{} columns", table.columns.len())),
                sql: sql_gen.generate_drop_table(&table.name),
                selected: true,
                warnings: vec![],
            });
        }
    }

    // Compare existing tables
    for source_table in source {
        if let Some(target_table) = target_map.get(source_table.name.as_str()) {
            compare_tables_cross(
                source_table,
                target_table,
                sql_gen,
                source_mapper,
                target_mapper,
                &mut diffs,
                &mut id_counter,
            );
        }
    }

    diffs
}

/// Map a table's columns through source->canonical->target, collecting warnings.
fn map_table_columns(
    table: &TableSchema,
    source_mapper: &dyn TypeMapper,
    target_mapper: &dyn TypeMapper,
) -> (TableSchema, Vec<TypeWarning>) {
    let mut warnings = Vec::new();
    let mapped_columns: Vec<Column> = table
        .columns
        .iter()
        .filter_map(|col| {
            let canonical = source_mapper.to_canonical(&col.data_type);
            let mapping = target_mapper.from_canonical(&canonical);

            if mapping.skipped {
                warnings.push(TypeWarning {
                    column_name: col.name.clone(),
                    source_type: col.data_type.clone(),
                    target_type: String::new(),
                    message: mapping.warning.unwrap_or_default(),
                    severity: WarningSeverity::Skipped,
                });
                return None;
            }

            if let Some(ref msg) = mapping.warning {
                warnings.push(TypeWarning {
                    column_name: col.name.clone(),
                    source_type: col.data_type.clone(),
                    target_type: mapping.sql_type.clone(),
                    message: msg.clone(),
                    severity: WarningSeverity::Degraded,
                });
            }

            let mapped_default = col
                .default_value
                .as_ref()
                .and_then(|d| target_mapper.map_default_value(d, &canonical));

            Some(Column {
                name: col.name.clone(),
                data_type: mapping.sql_type,
                nullable: col.nullable,
                default_value: mapped_default,
                auto_increment: col.auto_increment,
                comment: col.comment.clone(),
                ordinal_position: col.ordinal_position,
            })
        })
        .collect();

    let mapped_table = TableSchema {
        name: table.name.clone(),
        columns: mapped_columns,
        primary_key: table.primary_key.clone(),
        indexes: table.indexes.clone(),
        foreign_keys: table.foreign_keys.clone(),
        unique_constraints: table.unique_constraints.clone(),
    };

    (mapped_table, warnings)
}

/// Map a single column through source->canonical->target.
fn map_column(
    col: &Column,
    source_mapper: &dyn TypeMapper,
    target_mapper: &dyn TypeMapper,
) -> (Column, TypeMapping) {
    let canonical = source_mapper.to_canonical(&col.data_type);
    let mapping = target_mapper.from_canonical(&canonical);

    let mapped_default = col
        .default_value
        .as_ref()
        .and_then(|d| target_mapper.map_default_value(d, &canonical));

    let mapped_col = Column {
        name: col.name.clone(),
        data_type: mapping.sql_type.clone(),
        nullable: col.nullable,
        default_value: mapped_default,
        auto_increment: col.auto_increment,
        comment: col.comment.clone(),
        ordinal_position: col.ordinal_position,
    };

    (mapped_col, mapping)
}

/// Cross-db column comparison: compare on canonical type, ignore comments.
fn columns_equal_cross(
    source: &Column,
    target: &Column,
    source_mapper: &dyn TypeMapper,
    target_mapper: &dyn TypeMapper,
) -> bool {
    let source_canonical = source_mapper.to_canonical(&source.data_type);
    let target_canonical = target_mapper.to_canonical(&target.data_type);

    source.name == target.name
        && source_canonical == target_canonical
        && source.nullable == target.nullable
        && source.auto_increment == target.auto_increment
    // Intentionally skip: comment (PG doesn't support column comments in the same way)
    // Intentionally skip: default_value (compared via canonical mapping)
}

fn column_detail_mapped(col: &Column) -> String {
    let mut parts = vec![col.data_type.clone()];
    if col.nullable {
        parts.push("NULL".to_string());
    } else {
        parts.push("NOT NULL".to_string());
    }
    if let Some(default) = &col.default_value {
        parts.push(format!("DEFAULT {}", default));
    }
    if col.auto_increment {
        parts.push("AUTO_INCREMENT".to_string());
    }
    parts.join(" ")
}

fn compare_tables_cross(
    source: &TableSchema,
    target: &TableSchema,
    sql_gen: &dyn SqlGenerator,
    source_mapper: &dyn TypeMapper,
    target_mapper: &dyn TypeMapper,
    diffs: &mut Vec<DiffItem>,
    id_counter: &mut u32,
) {
    let source_cols: HashMap<&str, &Column> = source
        .columns
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    let target_cols: HashMap<&str, &Column> = target
        .columns
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    // Added + Modified columns
    for col in &source.columns {
        if !target_cols.contains_key(col.name.as_str()) {
            let (mapped_col, mapping) = map_column(col, source_mapper, target_mapper);
            if mapping.skipped {
                continue;
            }
            let mut warnings = vec![];
            if let Some(ref msg) = mapping.warning {
                warnings.push(TypeWarning {
                    column_name: col.name.clone(),
                    source_type: col.data_type.clone(),
                    target_type: mapped_col.data_type.clone(),
                    message: msg.clone(),
                    severity: WarningSeverity::Degraded,
                });
            }
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ColumnAdded,
                table_name: source.name.clone(),
                object_name: Some(col.name.clone()),
                source_def: Some(col.data_type.clone()),
                target_def: None,
                sql: sql_gen.generate_add_column(&source.name, &mapped_col),
                selected: true,
                warnings,
            });
        } else if let Some(target_col) = target_cols.get(col.name.as_str()) {
            if !columns_equal_cross(col, target_col, source_mapper, target_mapper) {
                let (mapped_col, mapping) = map_column(col, source_mapper, target_mapper);
                if mapping.skipped {
                    continue;
                }
                let mut warnings = vec![];
                if let Some(ref msg) = mapping.warning {
                    warnings.push(TypeWarning {
                        column_name: col.name.clone(),
                        source_type: col.data_type.clone(),
                        target_type: mapped_col.data_type.clone(),
                        message: msg.clone(),
                        severity: WarningSeverity::Degraded,
                    });
                }
                *id_counter += 1;
                diffs.push(DiffItem {
                    id: id_counter.to_string(),
                    diff_type: DiffType::ColumnModified,
                    table_name: source.name.clone(),
                    object_name: Some(col.name.clone()),
                    source_def: Some(column_detail_mapped(&mapped_col)),
                    target_def: Some(column_detail_mapped(target_col)),
                    sql: sql_gen.generate_modify_column(&source.name, &mapped_col),
                    selected: true,
                    warnings,
                });
            }
        }
    }

    // Removed columns
    for col in &target.columns {
        if !source_cols.contains_key(col.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ColumnRemoved,
                table_name: source.name.clone(),
                object_name: Some(col.name.clone()),
                source_def: None,
                target_def: Some(col.data_type.clone()),
                sql: sql_gen.generate_drop_column(&source.name, &col.name),
                selected: true,
                warnings: vec![],
            });
        }
    }

    // Indexes, FKs, UCs -- delegate to existing helpers (same-db logic is fine,
    // these don't have type mapping concerns)
    super::comparator::compare_indexes(source, target, sql_gen, diffs, id_counter);
    super::comparator::compare_foreign_keys(source, target, sql_gen, diffs, id_counter);
    super::comparator::compare_unique_constraints(source, target, sql_gen, diffs, id_counter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MySqlSqlGenerator;
    use crate::db::PostgresSqlGenerator;
    use crate::types::{MySqlTypeMapper, PostgresTypeMapper};

    fn make_column(name: &str, data_type: &str) -> Column {
        Column {
            name: name.to_string(),
            data_type: data_type.to_string(),
            nullable: false,
            default_value: None,
            auto_increment: false,
            comment: None,
            ordinal_position: 1,
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

    #[test]
    fn test_int_vs_integer_no_false_positive() {
        // MySQL "int(11)" and PG "integer" should map to same CanonicalType::Int
        let source = vec![make_table("users", vec![make_column("id", "int(11)")])];
        let target = vec![make_table("users", vec![make_column("id", "integer")])];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        let col_mods: Vec<_> = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::ColumnModified)
            .collect();
        assert!(
            col_mods.is_empty(),
            "int(11) vs integer should not produce a diff"
        );
    }

    #[test]
    fn test_real_type_difference_detected() {
        let source = vec![make_table(
            "users",
            vec![make_column("name", "varchar(255)")],
        )];
        let target = vec![make_table("users", vec![make_column("name", "integer")])];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        let col_mods: Vec<_> = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::ColumnModified)
            .collect();
        assert_eq!(col_mods.len(), 1);
    }

    #[test]
    fn test_comment_difference_ignored_cross_db() {
        let mut source_col = make_column("id", "int(11)");
        source_col.comment = Some("Primary key".to_string());
        let mut target_col = make_column("id", "integer");
        target_col.comment = None; // PG doesn't read comments

        let source = vec![make_table("users", vec![source_col])];
        let target = vec![make_table("users", vec![target_col])];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        let col_mods: Vec<_> = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::ColumnModified)
            .collect();
        assert!(
            col_mods.is_empty(),
            "comment difference should be ignored cross-db"
        );
    }

    #[test]
    fn test_warnings_attached_to_diff_items() {
        // jsonb -> json produces a degraded warning when targeting MySQL
        let source = vec![make_table("data", vec![make_column("payload", "jsonb")])];
        let target: Vec<TableSchema> = vec![]; // table doesn't exist in target

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &PostgresTypeMapper,
            &MySqlTypeMapper,
        );

        // TableAdded -- check if any warnings are present on it
        let table_added: Vec<_> = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::TableAdded)
            .collect();
        assert_eq!(table_added.len(), 1);
        assert!(
            !table_added[0].warnings.is_empty(),
            "should have degradation warning for jsonb"
        );
    }

    #[test]
    fn test_new_table_added_cross_db() {
        let source = vec![make_table("users", vec![make_column("id", "int(11)")])];
        let target: Vec<TableSchema> = vec![];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::TableAdded);
    }

    #[test]
    fn test_table_removed_cross_db() {
        let source: Vec<TableSchema> = vec![];
        let target = vec![make_table("old_table", vec![make_column("id", "integer")])];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::TableRemoved);
    }

    #[test]
    fn test_skipped_columns_filtered_from_create_table() {
        // Unknown type should be skipped with a Skipped warning
        let source = vec![make_table(
            "data",
            vec![
                make_column("id", "int(11)"),
                make_column("meta", "some_custom_type"), // Unknown → skipped
            ],
        )];
        let target: Vec<TableSchema> = vec![];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::TableAdded);
        // The SQL should NOT contain the skipped column
        assert!(
            !diffs[0].sql.contains("meta"),
            "skipped column should not appear in SQL"
        );
        // Should have a Skipped warning
        let skipped: Vec<_> = diffs[0]
            .warnings
            .iter()
            .filter(|w| w.severity == WarningSeverity::Skipped)
            .collect();
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0].column_name, "meta");
    }

    #[test]
    fn test_nullable_difference_detected() {
        let mut source_col = make_column("name", "varchar(255)");
        source_col.nullable = true;
        let mut target_col = make_column("name", "character varying(255)");
        target_col.nullable = false;

        let source = vec![make_table("users", vec![source_col])];
        let target = vec![make_table("users", vec![target_col])];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &PostgresSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        let col_mods: Vec<_> = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::ColumnModified)
            .collect();
        assert_eq!(col_mods.len(), 1, "nullable difference should be detected");
    }

    #[test]
    fn test_auto_increment_difference_detected() {
        let mut source_col = make_column("id", "int(11)");
        source_col.auto_increment = true;
        let target_col = make_column("id", "integer");

        let source = vec![make_table("users", vec![source_col])];
        let target = vec![make_table("users", vec![target_col])];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &PostgresSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        let col_mods: Vec<_> = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::ColumnModified)
            .collect();
        assert_eq!(
            col_mods.len(),
            1,
            "auto_increment difference should be detected"
        );
    }

    #[test]
    fn test_modified_column_with_degradation_warning() {
        // Source has jsonb, target has text — type differs AND jsonb degrades to json on MySQL target
        let source = vec![make_table("data", vec![make_column("payload", "jsonb")])];
        let target = vec![make_table("data", vec![make_column("payload", "text")])];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &PostgresTypeMapper,
            &MySqlTypeMapper,
        );

        let col_mods: Vec<_> = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::ColumnModified)
            .collect();
        assert_eq!(col_mods.len(), 1);
        // Should have a degradation warning for jsonb → json
        assert!(
            !col_mods[0].warnings.is_empty(),
            "should have degradation warning for jsonb"
        );
        assert_eq!(col_mods[0].warnings[0].severity, WarningSeverity::Degraded);
    }

    #[test]
    fn test_multiple_columns_mixed_warnings() {
        // Table with: normal column (int), degraded column (jsonb), skipped column (unknown)
        let source = vec![make_table(
            "mixed",
            vec![
                make_column("id", "integer"),
                make_column("data", "jsonb"),
                make_column("custom", "hstore"), // Unknown to MySQL
            ],
        )];
        let target: Vec<TableSchema> = vec![];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &PostgresTypeMapper,
            &MySqlTypeMapper,
        );

        assert_eq!(diffs.len(), 1);
        let warnings = &diffs[0].warnings;
        let degraded: Vec<_> = warnings
            .iter()
            .filter(|w| w.severity == WarningSeverity::Degraded)
            .collect();
        let skipped: Vec<_> = warnings
            .iter()
            .filter(|w| w.severity == WarningSeverity::Skipped)
            .collect();
        assert_eq!(
            degraded.len(),
            1,
            "jsonb should produce one degraded warning"
        );
        assert_eq!(
            skipped.len(),
            1,
            "hstore should produce one skipped warning"
        );
    }

    #[test]
    fn test_round_trip_mysql_to_pg_canonical_consistency() {
        // MySQL int(11) → canonical → PG integer → canonical should both be CanonicalType::Int
        let mysql = MySqlTypeMapper;
        let pg = PostgresTypeMapper;

        let mysql_canonical = mysql.to_canonical("int(11)");
        let pg_mapping = pg.from_canonical(&mysql_canonical);
        let pg_canonical = pg.to_canonical(&pg_mapping.sql_type);

        assert_eq!(
            mysql_canonical, pg_canonical,
            "round-trip MySQL→PG should preserve canonical type"
        );
    }

    #[test]
    fn test_round_trip_pg_to_mysql_canonical_consistency() {
        let pg = PostgresTypeMapper;
        let mysql = MySqlTypeMapper;

        // PG text → canonical → MySQL text → canonical
        let pg_canonical = pg.to_canonical("text");
        let mysql_mapping = mysql.from_canonical(&pg_canonical);
        let mysql_canonical = mysql.to_canonical(&mysql_mapping.sql_type);
        assert_eq!(
            pg_canonical, mysql_canonical,
            "round-trip PG→MySQL should preserve text"
        );

        // PG boolean → canonical → MySQL tinyint(1) → canonical
        let pg_canonical = pg.to_canonical("boolean");
        let mysql_mapping = mysql.from_canonical(&pg_canonical);
        let mysql_canonical = mysql.to_canonical(&mysql_mapping.sql_type);
        assert_eq!(
            pg_canonical, mysql_canonical,
            "round-trip PG→MySQL should preserve boolean"
        );

        // PG bigint → canonical → MySQL bigint → canonical
        let pg_canonical = pg.to_canonical("bigint");
        let mysql_mapping = mysql.from_canonical(&pg_canonical);
        let mysql_canonical = mysql.to_canonical(&mysql_mapping.sql_type);
        assert_eq!(
            pg_canonical, mysql_canonical,
            "round-trip PG→MySQL should preserve bigint"
        );
    }

    #[test]
    fn test_added_column_with_default_mapped() {
        let mut col = make_column("active", "boolean");
        col.default_value = Some("true".to_string());
        let source = vec![make_table("users", vec![make_column("id", "integer"), col])];
        let target = vec![make_table("users", vec![make_column("id", "int(11)")])];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &MySqlSqlGenerator as &dyn SqlGenerator,
            &PostgresTypeMapper,
            &MySqlTypeMapper,
        );

        let col_added: Vec<_> = diffs
            .iter()
            .filter(|d| d.diff_type == DiffType::ColumnAdded)
            .collect();
        assert_eq!(col_added.len(), 1);
        // Default "true" should be mapped to "1" for MySQL
        assert!(
            col_added[0].sql.contains("DEFAULT 1"),
            "PG 'true' default should map to MySQL '1'"
        );
    }

    #[test]
    fn test_identical_tables_cross_db_no_diff() {
        // Same logical schema but different raw types — should produce no diffs
        let source = vec![make_table(
            "users",
            vec![
                make_column("id", "int(11)"),
                make_column("name", "varchar(255)"),
                make_column("active", "tinyint(1)"),
            ],
        )];
        let target = vec![make_table(
            "users",
            vec![
                make_column("id", "integer"),
                make_column("name", "character varying(255)"),
                make_column("active", "boolean"),
            ],
        )];

        let diffs = compare_schemas_cross(
            &source,
            &target,
            &PostgresSqlGenerator as &dyn SqlGenerator,
            &MySqlTypeMapper,
            &PostgresTypeMapper,
        );

        assert!(
            diffs.is_empty(),
            "logically identical schemas should produce no diffs, got: {:?}",
            diffs
                .iter()
                .map(|d| (&d.diff_type, &d.object_name))
                .collect::<Vec<_>>()
        );
    }
}
