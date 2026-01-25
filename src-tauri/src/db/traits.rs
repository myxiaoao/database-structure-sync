use anyhow::Result;
use async_trait::async_trait;

use crate::models::{Column, Index, TableSchema};

#[async_trait]
pub trait SchemaReader: Send + Sync {
    async fn test_connection(&self) -> Result<()>;
    async fn get_tables(&self) -> Result<Vec<TableSchema>>;
    async fn list_databases(&self) -> Result<Vec<String>>;
}

pub trait SqlGenerator: Send + Sync {
    fn quote_identifier(&self, name: &str) -> String;
    fn generate_create_table(&self, table: &TableSchema) -> String;
    fn generate_drop_table(&self, table_name: &str) -> String;
    fn generate_add_column(&self, table: &str, column: &Column) -> String;
    fn generate_drop_column(&self, table: &str, column_name: &str) -> String;
    fn generate_modify_column(&self, table: &str, column: &Column) -> String;
    fn generate_add_index(&self, table: &str, index: &Index) -> String;
    fn generate_drop_index(&self, table: &str, index_name: &str) -> String;
    fn generate_add_foreign_key(&self, table: &str, fk: &crate::models::ForeignKey) -> String;
    fn generate_drop_foreign_key(&self, table: &str, fk_name: &str) -> String;
    fn generate_add_unique(&self, table: &str, uc: &crate::models::UniqueConstraint) -> String;
    fn generate_drop_unique(&self, table: &str, uc_name: &str) -> String;
}
