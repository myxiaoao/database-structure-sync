pub mod mysql;
pub mod postgres;
pub mod traits;

use crate::models::*;
use std::collections::HashMap;

pub use mysql::MySqlDriver;
pub use mysql::MySqlSqlGenerator;
pub use postgres::PostgresDriver;
pub use postgres::PostgresSqlGenerator;
pub use traits::{SchemaReader, SqlGenerator};

/// Raw row types for batch metadata queries.
/// Each driver queries all tables at once and returns these intermediate types.

pub struct ColumnRow {
    pub table_name: String,
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub auto_increment: bool,
    pub comment: Option<String>,
    pub ordinal_position: u32,
}

pub struct PkRow {
    pub table_name: String,
    pub constraint_name: Option<String>,
    pub column_name: String,
}

pub struct IndexRow {
    pub table_name: String,
    pub index_name: String,
    pub column_name: String,
    pub is_unique: bool,
    pub index_type: String,
}

pub struct FkRow {
    pub table_name: String,
    pub constraint_name: String,
    pub column_name: String,
    pub ref_table: String,
    pub ref_column: String,
    pub on_delete: String,
    pub on_update: String,
}

pub struct UcRow {
    pub table_name: String,
    pub constraint_name: String,
    pub column_name: String,
}

/// Assemble raw metadata rows into Vec<TableSchema>, grouped by table name.
pub fn assemble_schemas(
    table_names: Vec<String>,
    column_rows: Vec<ColumnRow>,
    pk_rows: Vec<PkRow>,
    index_rows: Vec<IndexRow>,
    fk_rows: Vec<FkRow>,
    uc_rows: Vec<UcRow>,
) -> Vec<TableSchema> {
    // Group columns by table
    let mut columns_map: HashMap<String, Vec<Column>> = HashMap::new();
    for r in column_rows {
        columns_map.entry(r.table_name).or_default().push(Column {
            name: r.name,
            data_type: r.data_type,
            nullable: r.nullable,
            default_value: r.default_value,
            auto_increment: r.auto_increment,
            comment: r.comment,
            ordinal_position: r.ordinal_position,
        });
    }

    // Group PKs by table
    let mut pk_map: HashMap<String, (Option<String>, Vec<String>)> = HashMap::new();
    for r in pk_rows {
        let entry = pk_map
            .entry(r.table_name)
            .or_insert((r.constraint_name.clone(), Vec::new()));
        entry.1.push(r.column_name);
    }

    // Group indexes by table -> index_name
    let mut index_map: HashMap<String, HashMap<String, (bool, String, Vec<String>)>> =
        HashMap::new();
    for r in index_rows {
        let table_entry = index_map.entry(r.table_name).or_default();
        let idx_entry =
            table_entry
                .entry(r.index_name)
                .or_insert((r.is_unique, r.index_type, Vec::new()));
        idx_entry.2.push(r.column_name);
    }

    // Group FKs by table -> constraint_name
    let mut fk_map: HashMap<
        String,
        HashMap<String, (String, Vec<String>, Vec<String>, String, String)>,
    > = HashMap::new();
    for r in fk_rows {
        let table_entry = fk_map.entry(r.table_name).or_default();
        let fk_entry = table_entry.entry(r.constraint_name).or_insert((
            r.ref_table,
            Vec::new(),
            Vec::new(),
            r.on_delete,
            r.on_update,
        ));
        fk_entry.1.push(r.column_name);
        fk_entry.2.push(r.ref_column);
    }

    // Group UCs by table -> constraint_name
    let mut uc_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    for r in uc_rows {
        let table_entry = uc_map.entry(r.table_name).or_default();
        table_entry
            .entry(r.constraint_name)
            .or_default()
            .push(r.column_name);
    }

    // Assemble
    table_names
        .into_iter()
        .map(|name| {
            let columns = columns_map.remove(&name).unwrap_or_default();

            let primary_key = pk_map.remove(&name).map(|(pk_name, columns)| PrimaryKey {
                name: pk_name,
                columns,
            });

            let indexes = index_map
                .remove(&name)
                .unwrap_or_default()
                .into_iter()
                .map(|(idx_name, (unique, idx_type, cols))| Index {
                    name: idx_name,
                    columns: cols,
                    unique,
                    index_type: idx_type,
                })
                .collect();

            let foreign_keys = fk_map
                .remove(&name)
                .unwrap_or_default()
                .into_iter()
                .map(
                    |(fk_name, (ref_table, cols, ref_cols, on_delete, on_update))| ForeignKey {
                        name: fk_name,
                        columns: cols,
                        ref_table,
                        ref_columns: ref_cols,
                        on_delete,
                        on_update,
                    },
                )
                .collect();

            let unique_constraints = uc_map
                .remove(&name)
                .unwrap_or_default()
                .into_iter()
                .map(|(uc_name, cols)| UniqueConstraint {
                    name: uc_name,
                    columns: cols,
                })
                .collect();

            TableSchema {
                name,
                columns,
                primary_key,
                indexes,
                foreign_keys,
                unique_constraints,
            }
        })
        .collect()
}

/// Macro to delegate SqlGenerator trait from a Driver to its inner SqlGenerator.
macro_rules! impl_sql_generator_delegation {
    ($driver:ty, $generator:expr) => {
        impl SqlGenerator for $driver {
            fn quote_identifier(&self, name: &str) -> String {
                $generator.quote_identifier(name)
            }
            fn generate_create_table(&self, table: &TableSchema) -> String {
                $generator.generate_create_table(table)
            }
            fn generate_drop_table(&self, table_name: &str) -> String {
                $generator.generate_drop_table(table_name)
            }
            fn generate_add_column(&self, table: &str, column: &Column) -> String {
                $generator.generate_add_column(table, column)
            }
            fn generate_drop_column(&self, table: &str, column_name: &str) -> String {
                $generator.generate_drop_column(table, column_name)
            }
            fn generate_modify_column(&self, table: &str, column: &Column) -> String {
                $generator.generate_modify_column(table, column)
            }
            fn generate_add_index(&self, table: &str, index: &Index) -> String {
                $generator.generate_add_index(table, index)
            }
            fn generate_drop_index(&self, table: &str, index_name: &str) -> String {
                $generator.generate_drop_index(table, index_name)
            }
            fn generate_add_foreign_key(&self, table: &str, fk: &ForeignKey) -> String {
                $generator.generate_add_foreign_key(table, fk)
            }
            fn generate_drop_foreign_key(&self, table: &str, fk_name: &str) -> String {
                $generator.generate_drop_foreign_key(table, fk_name)
            }
            fn generate_add_unique(&self, table: &str, uc: &UniqueConstraint) -> String {
                $generator.generate_add_unique(table, uc)
            }
            fn generate_drop_unique(&self, table: &str, uc_name: &str) -> String {
                $generator.generate_drop_unique(table, uc_name)
            }
        }
    };
}

pub(crate) use impl_sql_generator_delegation;

/// Validate a foreign key action string. Returns the action if valid, or "NO ACTION" as fallback.
pub fn validate_fk_action(action: &str) -> &str {
    match action.to_uppercase().as_str() {
        "CASCADE" | "SET NULL" | "SET DEFAULT" | "RESTRICT" | "NO ACTION" => action,
        _ => "NO ACTION",
    }
}
