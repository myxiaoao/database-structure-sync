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
                if ssl.verify_server {
                    opts = opts.ssl_mode(sqlx::postgres::PgSslMode::VerifyCa);
                } else {
                    opts = opts.ssl_mode(sqlx::postgres::PgSslMode::Require);
                }
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
        // Use udt_name for USER-DEFINED (enum) and ARRAY types to get the real type name.
        // For arrays, udt_name starts with '_' (e.g., '_int4' for integer[]).
        // For enums, data_type = 'USER-DEFINED' and udt_name = the enum type name.
        let rows: Vec<(String, String, String, String, String, Option<String>, i32)> =
            sqlx::query_as(
                r#"
            SELECT
                table_name,
                column_name,
                CASE
                    WHEN data_type = 'character varying' THEN 'varchar(' || character_maximum_length || ')'
                    WHEN data_type = 'character' THEN 'char(' || character_maximum_length || ')'
                    WHEN data_type = 'numeric' THEN 'numeric(' || numeric_precision || ',' || numeric_scale || ')'
                    WHEN data_type = 'ARRAY' THEN udt_name
                    WHEN data_type = 'USER-DEFINED' THEN udt_name
                    ELSE data_type
                END as data_type,
                udt_name,
                is_nullable,
                column_default,
                ordinal_position
            FROM information_schema.columns
            WHERE table_schema = 'public'
            ORDER BY table_name, ordinal_position
            "#,
            )
            .fetch_all(&self.pool)
            .await?;

        // Fetch enum values for all user-defined enum types in public schema
        let enum_values = self.fetch_enum_values().await?;

        Ok(rows
            .into_iter()
            .map(
                |(table_name, name, data_type, udt_name, nullable, default, pos)| {
                    let auto_increment = default
                        .as_ref()
                        .map(|d| d.starts_with("nextval("))
                        .unwrap_or(false);

                    // Resolve the final data_type:
                    // - Arrays: udt_name starts with '_', convert to element_type[]
                    // - Enums: if we have enum values, format as enum('a','b','c')
                    // - Otherwise: use the CASE result as-is
                    let resolved_type = if udt_name.starts_with('_') {
                        // Array type: strip leading '_' and map to base type + '[]'
                        let element_udt = &udt_name[1..];
                        let element_type = Self::udt_to_sql_type(element_udt);
                        format!("{}[]", element_type)
                    } else if let Some(values) = enum_values.get(&udt_name) {
                        // User-defined enum: format as enum('val1','val2',...)
                        let vals = values
                            .iter()
                            .map(|v| format!("'{}'", v))
                            .collect::<Vec<_>>()
                            .join(",");
                        format!("enum({})", vals)
                    } else {
                        data_type
                    };

                    crate::db::ColumnRow {
                        table_name,
                        name,
                        data_type: resolved_type,
                        nullable: nullable == "YES",
                        default_value: if auto_increment { None } else { default },
                        auto_increment,
                        comment: None,
                        ordinal_position: pos as u32,
                    }
                },
            )
            .collect())
    }

    /// Fetch all enum type values from pg_enum for types in the public schema.
    async fn fetch_enum_values(&self) -> Result<std::collections::HashMap<String, Vec<String>>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT t.typname, e.enumlabel
            FROM pg_type t
            JOIN pg_enum e ON t.oid = e.enumtypid
            JOIN pg_namespace n ON t.typnamespace = n.oid
            WHERE n.nspname = 'public'
            ORDER BY t.typname, e.enumsortorder
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (type_name, label) in rows {
            map.entry(type_name).or_default().push(label);
        }
        Ok(map)
    }

    /// Map PostgreSQL udt_name (internal name) to SQL type string.
    fn udt_to_sql_type(udt_name: &str) -> &str {
        match udt_name {
            "int2" => "smallint",
            "int4" => "integer",
            "int8" => "bigint",
            "float4" => "real",
            "float8" => "double precision",
            "bool" => "boolean",
            "varchar" => "character varying",
            "bpchar" => "character",
            "text" => "text",
            "bytea" => "bytea",
            "numeric" => "numeric",
            "date" => "date",
            "time" | "timetz" => "time",
            "timestamp" | "timestamptz" => "timestamp",
            "json" => "json",
            "jsonb" => "jsonb",
            "uuid" => "uuid",
            "inet" => "inet",
            other => other,
        }
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
