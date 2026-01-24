use crate::db::SqlGenerator;
use crate::models::*;
use std::collections::HashMap;

pub fn compare_schemas(
    source: &[TableSchema],
    target: &[TableSchema],
    sql_gen: &dyn SqlGenerator,
) -> Vec<DiffItem> {
    let mut diffs = Vec::new();
    let mut id_counter = 0;

    let source_map: HashMap<&str, &TableSchema> = source.iter().map(|t| (t.name.as_str(), t)).collect();
    let target_map: HashMap<&str, &TableSchema> = target.iter().map(|t| (t.name.as_str(), t)).collect();

    // Find added tables (in source but not in target)
    for table in source {
        if !target_map.contains_key(table.name.as_str()) {
            id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::TableAdded,
                table_name: table.name.clone(),
                object_name: None,
                source_def: Some(format!("{} columns", table.columns.len())),
                target_def: None,
                sql: sql_gen.generate_create_table(table),
                selected: true,
            });
        }
    }

    // Find removed tables (in target but not in source)
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
            });
        }
    }

    // Compare existing tables
    for source_table in source {
        if let Some(target_table) = target_map.get(source_table.name.as_str()) {
            compare_tables(source_table, target_table, sql_gen, &mut diffs, &mut id_counter);
        }
    }

    diffs
}

fn compare_tables(
    source: &TableSchema,
    target: &TableSchema,
    sql_gen: &dyn SqlGenerator,
    diffs: &mut Vec<DiffItem>,
    id_counter: &mut u32,
) {
    let source_cols: HashMap<&str, &Column> = source.columns.iter().map(|c| (c.name.as_str(), c)).collect();
    let target_cols: HashMap<&str, &Column> = target.columns.iter().map(|c| (c.name.as_str(), c)).collect();

    // Compare columns
    for col in &source.columns {
        if !target_cols.contains_key(col.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ColumnAdded,
                table_name: source.name.clone(),
                object_name: Some(col.name.clone()),
                source_def: Some(col.data_type.clone()),
                target_def: None,
                sql: sql_gen.generate_add_column(&source.name, col),
                selected: true,
            });
        } else if let Some(target_col) = target_cols.get(col.name.as_str()) {
            if col != *target_col {
                *id_counter += 1;
                diffs.push(DiffItem {
                    id: id_counter.to_string(),
                    diff_type: DiffType::ColumnModified,
                    table_name: source.name.clone(),
                    object_name: Some(col.name.clone()),
                    source_def: Some(col.data_type.clone()),
                    target_def: Some(target_col.data_type.clone()),
                    sql: sql_gen.generate_modify_column(&source.name, col),
                    selected: true,
                });
            }
        }
    }

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
            });
        }
    }

    // Compare indexes
    let source_idx: HashMap<&str, &Index> = source.indexes.iter().map(|i| (i.name.as_str(), i)).collect();
    let target_idx: HashMap<&str, &Index> = target.indexes.iter().map(|i| (i.name.as_str(), i)).collect();

    for idx in &source.indexes {
        if !target_idx.contains_key(idx.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::IndexAdded,
                table_name: source.name.clone(),
                object_name: Some(idx.name.clone()),
                source_def: Some(idx.columns.join(", ")),
                target_def: None,
                sql: sql_gen.generate_add_index(&source.name, idx),
                selected: true,
            });
        } else if let Some(target_index) = target_idx.get(idx.name.as_str()) {
            if idx != *target_index {
                *id_counter += 1;
                diffs.push(DiffItem {
                    id: id_counter.to_string(),
                    diff_type: DiffType::IndexModified,
                    table_name: source.name.clone(),
                    object_name: Some(idx.name.clone()),
                    source_def: Some(idx.columns.join(", ")),
                    target_def: Some(target_index.columns.join(", ")),
                    sql: format!("{}\n{}", sql_gen.generate_drop_index(&source.name, &idx.name), sql_gen.generate_add_index(&source.name, idx)),
                    selected: true,
                });
            }
        }
    }

    for idx in &target.indexes {
        if !source_idx.contains_key(idx.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::IndexRemoved,
                table_name: source.name.clone(),
                object_name: Some(idx.name.clone()),
                source_def: None,
                target_def: Some(idx.columns.join(", ")),
                sql: sql_gen.generate_drop_index(&source.name, &idx.name),
                selected: true,
            });
        }
    }

    // Compare foreign keys
    let source_fks: HashMap<&str, &ForeignKey> = source.foreign_keys.iter().map(|f| (f.name.as_str(), f)).collect();
    let target_fks: HashMap<&str, &ForeignKey> = target.foreign_keys.iter().map(|f| (f.name.as_str(), f)).collect();

    for fk in &source.foreign_keys {
        if !target_fks.contains_key(fk.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ForeignKeyAdded,
                table_name: source.name.clone(),
                object_name: Some(fk.name.clone()),
                source_def: Some(format!("-> {}", fk.ref_table)),
                target_def: None,
                sql: sql_gen.generate_add_foreign_key(&source.name, fk),
                selected: true,
            });
        }
    }

    for fk in &target.foreign_keys {
        if !source_fks.contains_key(fk.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::ForeignKeyRemoved,
                table_name: source.name.clone(),
                object_name: Some(fk.name.clone()),
                source_def: None,
                target_def: Some(format!("-> {}", fk.ref_table)),
                sql: sql_gen.generate_drop_foreign_key(&source.name, &fk.name),
                selected: true,
            });
        }
    }

    // Compare unique constraints
    let source_ucs: HashMap<&str, &UniqueConstraint> = source.unique_constraints.iter().map(|u| (u.name.as_str(), u)).collect();
    let target_ucs: HashMap<&str, &UniqueConstraint> = target.unique_constraints.iter().map(|u| (u.name.as_str(), u)).collect();

    for uc in &source.unique_constraints {
        if !target_ucs.contains_key(uc.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::UniqueConstraintAdded,
                table_name: source.name.clone(),
                object_name: Some(uc.name.clone()),
                source_def: Some(uc.columns.join(", ")),
                target_def: None,
                sql: sql_gen.generate_add_unique(&source.name, uc),
                selected: true,
            });
        }
    }

    for uc in &target.unique_constraints {
        if !source_ucs.contains_key(uc.name.as_str()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: DiffType::UniqueConstraintRemoved,
                table_name: source.name.clone(),
                object_name: Some(uc.name.clone()),
                source_def: None,
                target_def: Some(uc.columns.join(", ")),
                sql: sql_gen.generate_drop_unique(&source.name, &uc.name),
                selected: true,
            });
        }
    }
}
