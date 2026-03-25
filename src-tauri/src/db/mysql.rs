use anyhow::Result;
use async_trait::async_trait;
use sqlx::{MySql, Pool, mysql::MySqlPoolOptions};

use super::validate_fk_action;
use crate::db::traits::{SchemaReader, SqlGenerator};
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
                opts = opts.ssl_mode(sqlx::mysql::MySqlSslMode::Required);
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
        Ok(super::assemble_schemas(
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
    async fn fetch_all_columns(&self) -> Result<Vec<super::ColumnRow>> {
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
                    super::ColumnRow {
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

    async fn fetch_all_primary_keys(&self) -> Result<Vec<super::PkRow>> {
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
            .map(|(table_name, constraint_name, column_name)| super::PkRow {
                table_name,
                constraint_name: Some(constraint_name),
                column_name,
            })
            .collect())
    }

    async fn fetch_all_indexes(&self) -> Result<Vec<super::IndexRow>> {
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
                |(table_name, index_name, non_unique, column_name, index_type)| super::IndexRow {
                    table_name,
                    index_name,
                    column_name,
                    is_unique: non_unique == 0,
                    index_type,
                },
            )
            .collect())
    }

    async fn fetch_all_foreign_keys(&self) -> Result<Vec<super::FkRow>> {
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
                    super::FkRow {
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

    async fn fetch_all_unique_constraints(&self) -> Result<Vec<super::UcRow>> {
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
            .map(|(table_name, constraint_name, column_name)| super::UcRow {
                table_name,
                constraint_name,
                column_name,
            })
            .collect())
    }
}

pub struct MySqlSqlGenerator;

impl SqlGenerator for MySqlSqlGenerator {
    fn quote_identifier(&self, name: &str) -> String {
        format!("`{}`", name.replace('`', "``"))
    }

    fn generate_create_table(&self, table: &TableSchema) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.quote_identifier(&table.name));

        let mut parts: Vec<String> = Vec::new();

        for col in &table.columns {
            let mut col_def = format!("  {} {}", self.quote_identifier(&col.name), col.data_type);
            if !col.nullable {
                col_def.push_str(" NOT NULL");
            } else {
                col_def.push_str(" NULL");
            }
            if let Some(default) = &col.default_value {
                col_def.push_str(&format!(" DEFAULT {}", default));
            }
            if col.auto_increment {
                col_def.push_str(" AUTO_INCREMENT");
            }
            if let Some(comment) = &col.comment {
                col_def.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
            }
            parts.push(col_def);
        }

        if let Some(pk) = &table.primary_key {
            let cols: Vec<String> = pk
                .columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            parts.push(format!("  PRIMARY KEY ({})", cols.join(", ")));
        }

        for idx in &table.indexes {
            let cols: Vec<String> = idx
                .columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            let idx_type = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
            parts.push(format!(
                "  {} {} ({})",
                idx_type,
                self.quote_identifier(&idx.name),
                cols.join(", ")
            ));
        }

        for uc in &table.unique_constraints {
            let cols: Vec<String> = uc
                .columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            parts.push(format!(
                "  CONSTRAINT {} UNIQUE ({})",
                self.quote_identifier(&uc.name),
                cols.join(", ")
            ));
        }

        for fk in &table.foreign_keys {
            let cols: Vec<String> = fk
                .columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            let ref_cols: Vec<String> = fk
                .ref_columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            parts.push(format!(
                "  CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {}",
                self.quote_identifier(&fk.name),
                cols.join(", "),
                self.quote_identifier(&fk.ref_table),
                ref_cols.join(", "),
                validate_fk_action(&fk.on_delete),
                validate_fk_action(&fk.on_update)
            ));
        }

        sql.push_str(&parts.join(",\n"));
        sql.push_str("\n);");
        sql
    }

    fn generate_drop_table(&self, table_name: &str) -> String {
        format!("DROP TABLE {};", self.quote_identifier(table_name))
    }

    fn generate_add_column(&self, table: &str, column: &Column) -> String {
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            self.quote_identifier(table),
            self.quote_identifier(&column.name),
            column.data_type
        );
        if !column.nullable {
            sql.push_str(" NOT NULL");
        } else {
            sql.push_str(" NULL");
        }
        if let Some(default) = &column.default_value {
            sql.push_str(&format!(" DEFAULT {}", default));
        }
        if column.auto_increment {
            sql.push_str(" AUTO_INCREMENT");
        }
        if let Some(comment) = &column.comment {
            sql.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
        }
        sql.push(';');
        sql
    }

    fn generate_drop_column(&self, table: &str, column_name: &str) -> String {
        format!(
            "ALTER TABLE {} DROP COLUMN {};",
            self.quote_identifier(table),
            self.quote_identifier(column_name)
        )
    }

    fn generate_modify_column(&self, table: &str, column: &Column) -> String {
        let mut sql = format!(
            "ALTER TABLE {} MODIFY COLUMN {} {}",
            self.quote_identifier(table),
            self.quote_identifier(&column.name),
            column.data_type
        );
        if !column.nullable {
            sql.push_str(" NOT NULL");
        } else {
            sql.push_str(" NULL");
        }
        if let Some(default) = &column.default_value {
            sql.push_str(&format!(" DEFAULT {}", default));
        } else if column.nullable {
            sql.push_str(" DEFAULT NULL");
        }
        if column.auto_increment {
            sql.push_str(" AUTO_INCREMENT");
        }
        if let Some(comment) = &column.comment {
            sql.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
        }
        sql.push(';');
        sql
    }

    fn generate_add_index(&self, table: &str, index: &Index) -> String {
        let cols: Vec<String> = index
            .columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect();
        let idx_type = if index.unique {
            "UNIQUE INDEX"
        } else {
            "INDEX"
        };
        format!(
            "CREATE {} {} ON {} ({});",
            idx_type,
            self.quote_identifier(&index.name),
            self.quote_identifier(table),
            cols.join(", ")
        )
    }

    fn generate_drop_index(&self, table: &str, index_name: &str) -> String {
        format!(
            "DROP INDEX {} ON {};",
            self.quote_identifier(index_name),
            self.quote_identifier(table)
        )
    }

    fn generate_add_foreign_key(&self, table: &str, fk: &ForeignKey) -> String {
        let cols: Vec<String> = fk
            .columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect();
        let ref_cols: Vec<String> = fk
            .ref_columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect();
        format!(
            "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {};",
            self.quote_identifier(table),
            self.quote_identifier(&fk.name),
            cols.join(", "),
            self.quote_identifier(&fk.ref_table),
            ref_cols.join(", "),
            validate_fk_action(&fk.on_delete),
            validate_fk_action(&fk.on_update)
        )
    }

    fn generate_drop_foreign_key(&self, table: &str, fk_name: &str) -> String {
        format!(
            "ALTER TABLE {} DROP FOREIGN KEY {};",
            self.quote_identifier(table),
            self.quote_identifier(fk_name)
        )
    }

    fn generate_add_unique(&self, table: &str, uc: &UniqueConstraint) -> String {
        let cols: Vec<String> = uc
            .columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect();
        format!(
            "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({});",
            self.quote_identifier(table),
            self.quote_identifier(&uc.name),
            cols.join(", ")
        )
    }

    fn generate_drop_unique(&self, table: &str, uc_name: &str) -> String {
        format!(
            "ALTER TABLE {} DROP INDEX {};",
            self.quote_identifier(table),
            self.quote_identifier(uc_name)
        )
    }
}

super::impl_sql_generator_delegation!(MySqlDriver, MySqlSqlGenerator);
