pub mod mysql;
pub mod postgres;
pub mod traits;

pub use mysql::MySqlDriver;
pub use mysql::MySqlSqlGenerator;
pub use postgres::PostgresDriver;
pub use postgres::PostgresSqlGenerator;
pub use traits::{SchemaReader, SqlGenerator};

/// Validate a foreign key action string. Returns the action if valid, or "NO ACTION" as fallback.
pub fn validate_fk_action(action: &str) -> &str {
    match action.to_uppercase().as_str() {
        "CASCADE" | "SET NULL" | "SET DEFAULT" | "RESTRICT" | "NO ACTION" => action,
        _ => "NO ACTION",
    }
}
