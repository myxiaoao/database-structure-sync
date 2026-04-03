use log::debug;

use crate::db::SqlGenerator;
use crate::models::*;
use std::collections::HashMap;

trait NamedItem {
    fn name(&self) -> &str;
}

impl NamedItem for Column {
    fn name(&self) -> &str {
        &self.name
    }
}
impl NamedItem for Index {
    fn name(&self) -> &str {
        &self.name
    }
}
impl NamedItem for ForeignKey {
    fn name(&self) -> &str {
        &self.name
    }
}
impl NamedItem for UniqueConstraint {
    fn name(&self) -> &str {
        &self.name
    }
}

struct DiffConfig<'a, T> {
    table_name: &'a str,
    source_items: &'a [T],
    target_items: &'a [T],
    added_type: DiffType,
    removed_type: DiffType,
    modified_type: DiffType,
    source_def: fn(&T) -> String,
    target_def: fn(&T) -> String,
    generate_add: fn(&dyn SqlGenerator, &str, &T) -> String,
    generate_drop: fn(&dyn SqlGenerator, &str, &str) -> String,
}

fn compare_named_items<T: NamedItem + PartialEq>(
    config: &DiffConfig<T>,
    sql_gen: &dyn SqlGenerator,
    id_counter: &mut u32,
    diffs: &mut Vec<DiffItem>,
) {
    let source_map: HashMap<&str, &T> = config.source_items.iter().map(|i| (i.name(), i)).collect();
    let target_map: HashMap<&str, &T> = config.target_items.iter().map(|i| (i.name(), i)).collect();

    // Added + Modified
    for item in config.source_items {
        if !target_map.contains_key(item.name()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: config.added_type.clone(),
                table_name: config.table_name.to_string(),
                object_name: Some(item.name().to_string()),
                source_def: Some((config.source_def)(item)),
                target_def: None,
                sql: (config.generate_add)(sql_gen, config.table_name, item),
                selected: true,
                warnings: vec![],
            });
        } else if let Some(target_item) = target_map.get(item.name()) {
            if item != *target_item {
                *id_counter += 1;
                diffs.push(DiffItem {
                    id: id_counter.to_string(),
                    diff_type: config.modified_type.clone(),
                    table_name: config.table_name.to_string(),
                    object_name: Some(item.name().to_string()),
                    source_def: Some((config.source_def)(item)),
                    target_def: Some((config.target_def)(target_item)),
                    sql: format!(
                        "{}\n{}",
                        (config.generate_drop)(sql_gen, config.table_name, item.name()),
                        (config.generate_add)(sql_gen, config.table_name, item)
                    ),
                    selected: true,
                    warnings: vec![],
                });
            }
        }
    }

    // Removed
    for item in config.target_items {
        if !source_map.contains_key(item.name()) {
            *id_counter += 1;
            diffs.push(DiffItem {
                id: id_counter.to_string(),
                diff_type: config.removed_type.clone(),
                table_name: config.table_name.to_string(),
                object_name: Some(item.name().to_string()),
                source_def: None,
                target_def: Some((config.target_def)(item)),
                sql: (config.generate_drop)(sql_gen, config.table_name, item.name()),
                selected: true,
                warnings: vec![],
            });
        }
    }
}

fn column_detail(col: &Column) -> String {
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
    if let Some(comment) = &col.comment {
        parts.push(format!("COMMENT '{}'", comment));
    }
    parts.join(" ")
}

pub fn compare_schemas(
    source: &[TableSchema],
    target: &[TableSchema],
    sql_gen: &dyn SqlGenerator,
) -> Vec<DiffItem> {
    let mut diffs = Vec::new();
    let mut id_counter = 0;

    let source_map: HashMap<&str, &TableSchema> =
        source.iter().map(|t| (t.name.as_str(), t)).collect();
    let target_map: HashMap<&str, &TableSchema> =
        target.iter().map(|t| (t.name.as_str(), t)).collect();

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
                warnings: vec![],
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
                warnings: vec![],
            });
        }
    }

    // Compare existing tables
    for source_table in source {
        if let Some(target_table) = target_map.get(source_table.name.as_str()) {
            compare_tables(
                source_table,
                target_table,
                sql_gen,
                &mut diffs,
                &mut id_counter,
            );
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
                warnings: vec![],
            });
        } else if let Some(target_col) = target_cols.get(col.name.as_str()) {
            if col != *target_col {
                debug!(
                    "Column diff detected: {}.{} | source: {:?} | target: {:?}",
                    source.name, col.name, col, target_col
                );
                *id_counter += 1;
                diffs.push(DiffItem {
                    id: id_counter.to_string(),
                    diff_type: DiffType::ColumnModified,
                    table_name: source.name.clone(),
                    object_name: Some(col.name.clone()),
                    source_def: Some(column_detail(col)),
                    target_def: Some(column_detail(target_col)),
                    sql: sql_gen.generate_modify_column(&source.name, col),
                    selected: true,
                    warnings: vec![],
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
                warnings: vec![],
            });
        }
    }

    // Compare indexes, foreign keys, unique constraints
    compare_indexes(source, target, sql_gen, diffs, id_counter);
    compare_foreign_keys(source, target, sql_gen, diffs, id_counter);
    compare_unique_constraints(source, target, sql_gen, diffs, id_counter);
}

pub(crate) fn compare_indexes(
    source: &TableSchema,
    target: &TableSchema,
    sql_gen: &dyn SqlGenerator,
    diffs: &mut Vec<DiffItem>,
    id_counter: &mut u32,
) {
    compare_named_items(
        &DiffConfig {
            table_name: &source.name,
            source_items: &source.indexes,
            target_items: &target.indexes,
            added_type: DiffType::IndexAdded,
            removed_type: DiffType::IndexRemoved,
            modified_type: DiffType::IndexModified,
            source_def: |idx| idx.columns.join(", "),
            target_def: |idx| idx.columns.join(", "),
            generate_add: |sg, t, idx| sg.generate_add_index(t, idx),
            generate_drop: |sg, t, name| sg.generate_drop_index(t, name),
        },
        sql_gen,
        id_counter,
        diffs,
    );
}

pub(crate) fn compare_foreign_keys(
    source: &TableSchema,
    target: &TableSchema,
    sql_gen: &dyn SqlGenerator,
    diffs: &mut Vec<DiffItem>,
    id_counter: &mut u32,
) {
    compare_named_items(
        &DiffConfig {
            table_name: &source.name,
            source_items: &source.foreign_keys,
            target_items: &target.foreign_keys,
            added_type: DiffType::ForeignKeyAdded,
            removed_type: DiffType::ForeignKeyRemoved,
            modified_type: DiffType::ForeignKeyModified,
            source_def: |fk| format!("-> {}", fk.ref_table),
            target_def: |fk| format!("-> {}", fk.ref_table),
            generate_add: |sg, t, fk| sg.generate_add_foreign_key(t, fk),
            generate_drop: |sg, t, name| sg.generate_drop_foreign_key(t, name),
        },
        sql_gen,
        id_counter,
        diffs,
    );
}

pub(crate) fn compare_unique_constraints(
    source: &TableSchema,
    target: &TableSchema,
    sql_gen: &dyn SqlGenerator,
    diffs: &mut Vec<DiffItem>,
    id_counter: &mut u32,
) {
    compare_named_items(
        &DiffConfig {
            table_name: &source.name,
            source_items: &source.unique_constraints,
            target_items: &target.unique_constraints,
            added_type: DiffType::UniqueConstraintAdded,
            removed_type: DiffType::UniqueConstraintRemoved,
            modified_type: DiffType::UniqueConstraintModified,
            source_def: |uc| uc.columns.join(", "),
            target_def: |uc| uc.columns.join(", "),
            generate_add: |sg, t, uc| sg.generate_add_unique(t, uc),
            generate_drop: |sg, t, name| sg.generate_drop_unique(t, name),
        },
        sql_gen,
        id_counter,
        diffs,
    );
}
