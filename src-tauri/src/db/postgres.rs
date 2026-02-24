use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::db::traits::{SchemaReader, SqlGenerator};
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

        let mut tables = Vec::new();
        for (table_name,) in table_names {
            let columns = self.get_columns(&table_name).await?;
            let primary_key = self.get_primary_key(&table_name).await?;
            let indexes = self.get_indexes(&table_name).await?;
            let foreign_keys = self.get_foreign_keys(&table_name).await?;
            let unique_constraints = self.get_unique_constraints(&table_name).await?;

            tables.push(TableSchema {
                name: table_name,
                columns,
                primary_key,
                indexes,
                foreign_keys,
                unique_constraints,
            });
        }

        Ok(tables)
    }
}

impl PostgresDriver {
    async fn get_columns(&self, table_name: &str) -> Result<Vec<Column>> {
        let rows: Vec<(String, String, String, Option<String>, i32)> = sqlx::query_as(
            r#"
            SELECT
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
            WHERE table_schema = 'public' AND table_name = $1
            ORDER BY ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(name, data_type, nullable, default, pos)| {
                let auto_increment = default
                    .as_ref()
                    .map(|d| d.starts_with("nextval("))
                    .unwrap_or(false);
                Column {
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

    async fn get_primary_key(&self, table_name: &str) -> Result<Option<PrimaryKey>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT tc.constraint_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
            WHERE tc.table_schema = 'public' AND tc.table_name = $1 AND tc.constraint_type = 'PRIMARY KEY'
            ORDER BY kcu.ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let name = rows.first().map(|(n, _)| n.clone());
        let columns: Vec<String> = rows.into_iter().map(|(_, col)| col).collect();
        Ok(Some(PrimaryKey { name, columns }))
    }

    async fn get_indexes(&self, table_name: &str) -> Result<Vec<Index>> {
        let rows: Vec<(String, bool, String, String)> = sqlx::query_as(
            r#"
            SELECT
                i.relname as index_name,
                ix.indisunique as is_unique,
                a.attname as column_name,
                am.amname as index_type
            FROM pg_index ix
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_am am ON i.relam = am.oid
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            WHERE t.relname = $1 AND t.relnamespace = 'public'::regnamespace
                AND NOT ix.indisprimary
                AND NOT EXISTS (
                    SELECT 1 FROM pg_constraint c
                    WHERE c.conindid = ix.indexrelid AND c.contype = 'u'
                )
            ORDER BY i.relname, array_position(ix.indkey, a.attnum)
            "#,
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut indexes_map: std::collections::HashMap<String, (bool, String, Vec<String>)> =
            std::collections::HashMap::new();
        for (name, unique, column, idx_type) in rows {
            let entry = indexes_map
                .entry(name)
                .or_insert((unique, idx_type, Vec::new()));
            entry.2.push(column);
        }

        Ok(indexes_map
            .into_iter()
            .map(|(name, (unique, idx_type, columns))| Index {
                name,
                columns,
                unique,
                index_type: idx_type,
            })
            .collect())
    }

    async fn get_foreign_keys(&self, table_name: &str) -> Result<Vec<ForeignKey>> {
        let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT
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
            WHERE tc.table_schema = 'public' AND tc.table_name = $1 AND tc.constraint_type = 'FOREIGN KEY'
            ORDER BY tc.constraint_name, kcu.ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut fks_map: std::collections::HashMap<
            String,
            (String, Vec<String>, Vec<String>, String, String),
        > = std::collections::HashMap::new();
        for (name, col, ref_table, ref_col, on_delete, on_update) in rows {
            let entry = fks_map.entry(name).or_insert((
                ref_table,
                Vec::new(),
                Vec::new(),
                on_delete,
                on_update,
            ));
            entry.1.push(col);
            entry.2.push(ref_col);
        }

        Ok(fks_map
            .into_iter()
            .map(
                |(name, (ref_table, columns, ref_columns, on_delete, on_update))| ForeignKey {
                    name,
                    columns,
                    ref_table,
                    ref_columns,
                    on_delete,
                    on_update,
                },
            )
            .collect())
    }

    async fn get_unique_constraints(&self, table_name: &str) -> Result<Vec<UniqueConstraint>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT tc.constraint_name, kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
            WHERE tc.table_schema = 'public' AND tc.table_name = $1 AND tc.constraint_type = 'UNIQUE'
            ORDER BY tc.constraint_name, kcu.ordinal_position
            "#
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let mut ucs_map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (name, col) in rows {
            ucs_map.entry(name).or_default().push(col);
        }

        Ok(ucs_map
            .into_iter()
            .map(|(name, columns)| UniqueConstraint { name, columns })
            .collect())
    }
}

pub struct PostgresSqlGenerator;

fn validate_fk_action(action: &str) -> &str {
    match action.to_uppercase().as_str() {
        "CASCADE" | "SET NULL" | "SET DEFAULT" | "RESTRICT" | "NO ACTION" => action,
        _ => "NO ACTION",
    }
}

impl SqlGenerator for PostgresSqlGenerator {
    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    fn generate_create_table(&self, table: &TableSchema) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.quote_identifier(&table.name));
        let mut parts: Vec<String> = Vec::new();

        for col in &table.columns {
            let data_type = if col.auto_increment {
                "SERIAL".to_string()
            } else {
                col.data_type.clone()
            };
            let mut col_def = format!("  {} {}", self.quote_identifier(&col.name), data_type);
            if !col.nullable && !col.auto_increment {
                col_def.push_str(" NOT NULL");
            }
            if let Some(default) = &col.default_value {
                col_def.push_str(&format!(" DEFAULT {}", default));
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

        for idx in &table.indexes {
            let cols: Vec<String> = idx
                .columns
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            let idx_type = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
            sql.push_str(&format!(
                "\nCREATE {} {} ON {} ({});",
                idx_type,
                self.quote_identifier(&idx.name),
                self.quote_identifier(&table.name),
                cols.join(", ")
            ));
        }

        sql
    }

    fn generate_drop_table(&self, table_name: &str) -> String {
        format!("DROP TABLE {};", self.quote_identifier(table_name))
    }

    fn generate_add_column(&self, table: &str, column: &Column) -> String {
        let data_type = if column.auto_increment {
            "SERIAL".to_string()
        } else {
            column.data_type.clone()
        };
        let mut sql = format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            self.quote_identifier(table),
            self.quote_identifier(&column.name),
            data_type
        );
        if !column.nullable && !column.auto_increment {
            sql.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default_value {
            sql.push_str(&format!(" DEFAULT {}", default));
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
        let tbl = self.quote_identifier(table);
        let col = self.quote_identifier(&column.name);
        let mut stmts = Vec::new();

        // TYPE — never use SERIAL pseudo-type for ALTER COLUMN
        stmts.push(format!(
            "ALTER TABLE {} ALTER COLUMN {} TYPE {};",
            tbl, col, column.data_type
        ));

        // NOT NULL
        if !column.nullable {
            stmts.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL;",
                tbl, col
            ));
        } else {
            stmts.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL;",
                tbl, col
            ));
        }

        // DEFAULT
        if let Some(default) = &column.default_value {
            stmts.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {};",
                tbl, col, default
            ));
        } else {
            stmts.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;",
                tbl, col
            ));
        }

        stmts.join("\n")
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

    fn generate_drop_index(&self, _table: &str, index_name: &str) -> String {
        format!("DROP INDEX {};", self.quote_identifier(index_name))
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
            "ALTER TABLE {} DROP CONSTRAINT {};",
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
            "ALTER TABLE {} DROP CONSTRAINT {};",
            self.quote_identifier(table),
            self.quote_identifier(uc_name)
        )
    }
}

impl SqlGenerator for PostgresDriver {
    fn quote_identifier(&self, name: &str) -> String {
        PostgresSqlGenerator.quote_identifier(name)
    }
    fn generate_create_table(&self, table: &TableSchema) -> String {
        PostgresSqlGenerator.generate_create_table(table)
    }
    fn generate_drop_table(&self, table_name: &str) -> String {
        PostgresSqlGenerator.generate_drop_table(table_name)
    }
    fn generate_add_column(&self, table: &str, column: &Column) -> String {
        PostgresSqlGenerator.generate_add_column(table, column)
    }
    fn generate_drop_column(&self, table: &str, column_name: &str) -> String {
        PostgresSqlGenerator.generate_drop_column(table, column_name)
    }
    fn generate_modify_column(&self, table: &str, column: &Column) -> String {
        PostgresSqlGenerator.generate_modify_column(table, column)
    }
    fn generate_add_index(&self, table: &str, index: &Index) -> String {
        PostgresSqlGenerator.generate_add_index(table, index)
    }
    fn generate_drop_index(&self, table: &str, index_name: &str) -> String {
        PostgresSqlGenerator.generate_drop_index(table, index_name)
    }
    fn generate_add_foreign_key(&self, table: &str, fk: &ForeignKey) -> String {
        PostgresSqlGenerator.generate_add_foreign_key(table, fk)
    }
    fn generate_drop_foreign_key(&self, table: &str, fk_name: &str) -> String {
        PostgresSqlGenerator.generate_drop_foreign_key(table, fk_name)
    }
    fn generate_add_unique(&self, table: &str, uc: &UniqueConstraint) -> String {
        PostgresSqlGenerator.generate_add_unique(table, uc)
    }
    fn generate_drop_unique(&self, table: &str, uc_name: &str) -> String {
        PostgresSqlGenerator.generate_drop_unique(table, uc_name)
    }
}
