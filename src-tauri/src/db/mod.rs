pub mod mysql;
pub mod postgres;
pub mod traits;

pub use mysql::MySqlDriver;
pub use mysql::MySqlSqlGenerator;
pub use postgres::PostgresDriver;
pub use postgres::PostgresSqlGenerator;
pub use traits::{SchemaReader, SqlGenerator};

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
