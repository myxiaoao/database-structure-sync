use crate::types::CanonicalType;

/// Result of mapping a CanonicalType to a target database type.
#[derive(Debug, Clone)]
pub struct TypeMapping {
    /// Target DDL type string (e.g., "integer", "varchar(255)")
    pub sql_type: String,
    /// Degradation warning message, if any
    pub warning: Option<String>,
    /// True when the type cannot be mapped at all
    pub skipped: bool,
    /// SQL that must run before the column DDL (e.g., CREATE TYPE for PG enums)
    pub prerequisite_sql: Option<String>,
}

impl TypeMapping {
    pub fn direct(sql_type: impl Into<String>) -> Self {
        Self {
            sql_type: sql_type.into(),
            warning: None,
            skipped: false,
            prerequisite_sql: None,
        }
    }

    pub fn degraded(sql_type: impl Into<String>, warning: impl Into<String>) -> Self {
        Self {
            sql_type: sql_type.into(),
            warning: Some(warning.into()),
            skipped: false,
            prerequisite_sql: None,
        }
    }

    pub fn skipped(original: impl Into<String>) -> Self {
        Self {
            sql_type: String::new(),
            warning: Some(original.into()),
            skipped: true,
            prerequisite_sql: None,
        }
    }

    pub fn with_prerequisite(sql_type: impl Into<String>, prerequisite: impl Into<String>) -> Self {
        Self {
            sql_type: sql_type.into(),
            warning: None,
            skipped: false,
            prerequisite_sql: Some(prerequisite.into()),
        }
    }
}

/// Converts raw database type strings to/from CanonicalType.
pub trait TypeMapper: Send + Sync {
    /// Source raw type string → CanonicalType
    fn to_canonical(&self, raw_type: &str) -> CanonicalType;

    /// CanonicalType → target type string + optional warning.
    /// Takes `&self` because mappers may carry configuration state.
    #[allow(clippy::wrong_self_convention)]
    fn from_canonical(&self, canonical: &CanonicalType) -> TypeMapping;

    /// Convert a source default value to target dialect.
    /// Returns None if the default should be dropped.
    fn map_default_value(&self, default: &str, canonical: &CanonicalType) -> Option<String>;
}
