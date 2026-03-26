use anyhow::Result;
use async_trait::async_trait;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};

use crate::db::traits::SchemaReader;
use crate::models::*;

pub struct MySqlDriver {
    pool: Pool<MySql>,
}

impl MySqlDriver {
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
        let mut opts = sqlx::mysql::MySqlConnectOptions::new()
            .host(host)
            .port(port)
            .username(user)
            .password(password)
            .database(database);

        if let Some(ssl) = ssl_config {
            if ssl.enabled {
                if ssl.verify_server {
                    opts = opts.ssl_mode(sqlx::mysql::MySqlSslMode::VerifyCa);
                } else {
                    opts = opts.ssl_mode(sqlx::mysql::MySqlSslMode::Required);
                }
                if let Some(ca_path) = &ssl.ca_cert_path {
                    opts = opts.ssl_ca(ca_path);
                }
                if let Some(cert_path) = &ssl.client_cert_path {
                    opts = opts.ssl_client_cert(cert_path);
                }
                if let Some(key_path) = &ssl.client_key_path {
                    opts = opts.ssl_client_key(key_path);
                }
            }
        }

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &Pool<MySql> {
        &self.pool
    }
}

#[async_trait]
impl SchemaReader for MySqlDriver {
    async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    async fn list_databases(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT CAST(schema_name AS CHAR) FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys') ORDER BY schema_name"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|(name,)| name).collect())
    }

    async fn get_tables(&self) -> Result<Vec<TableSchema>> {
        let table_names: Vec<(String,)> = sqlx::query_as(
            "SELECT CAST(table_name AS CHAR) FROM information_schema.tables WHERE table_schema = DATABASE() AND table_type = 'BASE TABLE'"
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

impl MySqlDriver {
    async fn fetch_all_columns(&self) -> Result<Vec<crate::db::ColumnRow>> {
        let rows: Vec<(
            String,
            String,
            String,
            String,
            Option<String>,
            String,
            Option<String>,
            u32,
        )> = sqlx::query_as(
            r#"
            SELECT
                CAST(table_name AS CHAR),
                CAST(column_name AS CHAR),
                CAST(column_type AS CHAR),
                CAST(is_nullable AS CHAR),
                CAST(column_default AS CHAR),
                CAST(extra AS CHAR),
                CAST(column_comment AS CHAR),
                ordinal_position
            FROM information_schema.columns
            WHERE table_schema = DATABASE()
            ORDER BY table_name, ordinal_position
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(table_name, name, data_type, nullable, default, extra, comment, pos)| {
                    crate::db::ColumnRow {
                        table_name,
                        name,
                        data_type,
                        nullable: nullable == "YES",
                        default_value: default,
                        auto_increment: extra.contains("auto_increment"),
                        comment: if comment.as_ref().map(|c| c.is_empty()).unwrap_or(true) {
                            None
                        } else {
                            comment
                        },
                        ordinal_position: pos,
                    }
                },
            )
            .collect())
    }

    async fn fetch_all_primary_keys(&self) -> Result<Vec<crate::db::PkRow>> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            r#"
            SELECT CAST(table_name AS CHAR), CAST(constraint_name AS CHAR), CAST(column_name AS CHAR)
            FROM information_schema.key_column_usage
            WHERE table_schema = DATABASE() AND constraint_name = 'PRIMARY'
            ORDER BY table_name, ordinal_position
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
        let rows: Vec<(String, String, i32, String, String)> = sqlx::query_as(
            r#"
            SELECT CAST(s.table_name AS CHAR), CAST(s.index_name AS CHAR), s.non_unique, CAST(s.column_name AS CHAR), CAST(s.index_type AS CHAR)
            FROM information_schema.statistics s
            WHERE s.table_schema = DATABASE() AND s.index_name != 'PRIMARY'
                AND NOT EXISTS (
                    SELECT 1 FROM information_schema.table_constraints tc
                    WHERE tc.table_schema = s.table_schema
                        AND tc.table_name = s.table_name
                        AND tc.constraint_name = s.index_name
                        AND tc.constraint_type = 'UNIQUE'
                )
            ORDER BY s.table_name, s.index_name, s.seq_in_index
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(table_name, index_name, non_unique, column_name, index_type)| {
                    crate::db::IndexRow {
                        table_name,
                        index_name,
                        column_name,
                        is_unique: non_unique == 0,
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
                CAST(kcu.table_name AS CHAR),
                CAST(kcu.constraint_name AS CHAR),
                CAST(kcu.column_name AS CHAR),
                CAST(kcu.referenced_table_name AS CHAR),
                CAST(kcu.referenced_column_name AS CHAR),
                CAST(rc.delete_rule AS CHAR),
                CAST(rc.update_rule AS CHAR)
            FROM information_schema.key_column_usage kcu
            JOIN information_schema.referential_constraints rc
                ON kcu.constraint_name = rc.constraint_name AND kcu.table_schema = rc.constraint_schema
            WHERE kcu.table_schema = DATABASE() AND kcu.referenced_table_name IS NOT NULL
            ORDER BY kcu.table_name, kcu.constraint_name, kcu.ordinal_position
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
            SELECT CAST(tc.table_name AS CHAR), CAST(tc.constraint_name AS CHAR), CAST(kcu.column_name AS CHAR)
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema AND tc.table_name = kcu.table_name
            WHERE tc.table_schema = DATABASE() AND tc.constraint_type = 'UNIQUE'
            ORDER BY tc.table_name, tc.constraint_name, kcu.ordinal_position
            "#
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
