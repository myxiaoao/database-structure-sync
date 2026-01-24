use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiffType {
    TableAdded,
    TableRemoved,
    ColumnAdded,
    ColumnRemoved,
    ColumnModified,
    IndexAdded,
    IndexRemoved,
    IndexModified,
    ForeignKeyAdded,
    ForeignKeyRemoved,
    UniqueConstraintAdded,
    UniqueConstraintRemoved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffItem {
    pub id: String,
    pub diff_type: DiffType,
    pub table_name: String,
    pub object_name: Option<String>,
    pub source_def: Option<String>,
    pub target_def: Option<String>,
    pub sql: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub items: Vec<DiffItem>,
    pub source_tables: usize,
    pub target_tables: usize,
}
