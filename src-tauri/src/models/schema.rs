use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub auto_increment: bool,
    pub comment: Option<String>,
    pub ordinal_position: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimaryKey {
    pub name: Option<String>,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub index_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForeignKey {
    pub name: String,
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
    pub on_delete: String,
    pub on_update: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniqueConstraint {
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Option<PrimaryKey>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub unique_constraints: Vec<UniqueConstraint>,
}
