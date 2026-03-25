use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::db::traits::SchemaReader;
use crate::models::*;

pub struct PostgresDriver {
    pool: PgPool,
}

impl PostgresDriver {
    pub async fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
    ) -> Result<Self> {
        Self::new_with_ssl(host, port, user, password, database, None).await
    }

    pub async fn new_with_ssl(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        database: &str,
        ssl_config: Option<&SslConfig>,
    ) -> Result<Self> {
        let mut opts = sqlx::postgres::PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(user)
            .password(password)
            .database(database);

        if let Some(ssl) = ssl_config {
            if ssl.enabled {
                opts = opts.ssl_mode(sqlx::postgres::PgSslMode::Require);
                if let Some(ca_path) = &ssl.ca_cert_path {
                    opts = opts.ssl_root_cert(ca_path);
                }
            }
        }

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl SchemaReader for PostgresDriver {
    async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    async fn list_databases(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT datname FROM pg_database WHERE datistemplate = false AND datname NOT IN ('postgres') ORDER BY datname"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|(name,)| name).collect())
    }

    async fn get_tables(&self) -> Result<Vec<TableSchema>> {
        let table_names: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
        )
        .fetch_all(&self.pool)
        .await?;

        let table_names: Vec<String> = table_names.into_iter().map(|(n,)| n).collect();
        let columns = self.fetch_all_columns().await?;
        let pks = self.fetch_all_primary_keys().await?;
        let indexes = self.fetch_all_indexes().await?;
        let fks = self.fetch_all_foreign_keys().await?;
        let ucs = self.fetch_all_unique_constraints().await?;
        Ok(crate::db::assemble_schemas(
            table_names,
            columns,
            pks,
            indexes,
            fks,
            ucs,
        ))
    }
}

impl PostgresDriver {
    async fn fetch_all_columns(&self) -> Result<Vec<crate::db::ColumnRow>> {
        let rows: Vec<(String, String, String, String, Option<String>, i32)> = sqlx::query_as(
            r#"
            SELECT
                table_name,
                column_name,
                CASE
                    WHEN data_type = 'character varying' THEN 'varchar(' || character_maximum_length || ')'
                    WHEN data_type = 'character' THEN 'char(' || character_maximum_length || ')'
                    WHEN data_type = 'numeric' THEN 'numeric(' || numeric_precision || ',' || numeric_scale || ')'
                    ELSE data_type
                END as data_type,
                is_nullable,
                column_default,
                ordinal_position
            FROM information_schema.columns
            WHERE table_schema = 'public'
            ORDER BY table_name, ordinal_position
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(table_name, name, data_type, nullable, default, pos)| {
                let auto_increment = default
                    .as_ref()
                    .map(|d| d.starts_with("nextval("))
                    .unwrap_or(false);
                crate::db::ColumnRow {
                    table_name,
                    name,
                    data_type,
                    nullable: nullable == "YES",
                    default_value: if auto_increment { None } else { default },
                    auto_increment,
                    comment: None,
                    ordinal_position: pos as u32,
                }
            })
            .collect())
    }

    async fn fetch_all_primary_keys(&self) -> Result<Vec<crate::db::PkRow>> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            r#"
            SELECT tc.table_name, tc.constraint_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
            WHERE tc.table_schema = 'public' AND tc.constraint_type = 'PRIMARY KEY'
            ORDER BY tc.table_name, kcu.ordinal_position
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(table_name, constraint_name, column_name)| crate::db::PkRow {
                    table_name,
                    constraint_name: Some(constraint_name),
                    column_name,
                },
            )
            .collect())
    }

    async fn fetch_all_indexes(&self) -> Result<Vec<crate::db::IndexRow>> {
        let rows: Vec<(String, String, bool, String, String)> = sqlx::query_as(
            r#"
            SELECT
                t.relname as table_name,
                i.relname as index_name,
                ix.indisunique as is_unique,
                a.attname as column_name,
                am.amname as index_type
            FROM pg_index ix
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_am am ON i.relam = am.oid
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            WHERE t.relnamespace = 'public'::regnamespace
                AND NOT ix.indisprimary
                AND NOT EXISTS (
                    SELECT 1 FROM pg_constraint c
                    WHERE c.conindid = ix.indexrelid AND c.contype = 'u'
                )
            ORDER BY t.relname, i.relname, array_position(ix.indkey, a.attnum)
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(table_name, index_name, is_unique, column_name, index_type)| {
                    crate::db::IndexRow {
                        table_name,
                        index_name,
                        column_name,
                        is_unique,
                        index_type,
                    }
                },
            )
            .collect())
    }

    async fn fetch_all_foreign_keys(&self) -> Result<Vec<crate::db::FkRow>> {
        let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT
                tc.table_name,
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name AS ref_table,
                ccu.column_name AS ref_column,
                rc.delete_rule,
                rc.update_rule
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage ccu ON tc.constraint_name = ccu.constraint_name
            JOIN information_schema.referential_constraints rc ON tc.constraint_name = rc.constraint_name
            WHERE tc.table_schema = 'public' AND tc.constraint_type = 'FOREIGN KEY'
            ORDER BY tc.table_name, tc.constraint_name, kcu.ordinal_position
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    table_name,
                    constraint_name,
                    column_name,
                    ref_table,
                    ref_column,
                    on_delete,
                    on_update,
                )| {
                    crate::db::FkRow {
                        table_name,
                        constraint_name,
                        column_name,
                        ref_table,
                        ref_column,
                        on_delete,
                        on_update,
                    }
                },
            )
            .collect())
    }

    async fn fetch_all_unique_constraints(&self) -> Result<Vec<crate::db::UcRow>> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            r#"
            SELECT tc.table_name, tc.constraint_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            WHERE tc.table_schema = 'public' AND tc.constraint_type = 'UNIQUE'
            ORDER BY tc.table_name, tc.constraint_name, kcu.ordinal_position
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(table_name, constraint_name, column_name)| crate::db::UcRow {
                    table_name,
                    constraint_name,
                    column_name,
                },
            )
            .collect())
    }
}
