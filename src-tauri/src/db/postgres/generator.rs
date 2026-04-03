use crate::db::traits::SqlGenerator;
use crate::db::validate_fk_action;
use crate::models::*;

use super::reader::PostgresDriver;

pub struct PostgresSqlGenerator;

/// Choose the correct SERIAL variant based on the column's data type.
/// Falls back to SERIAL if the type doesn't clearly map to a size.
fn serial_type_for(data_type: &str) -> &'static str {
    let lower = data_type.to_lowercase();
    if lower.contains("big") || lower == "int8" {
        "BIGSERIAL"
    } else if lower.contains("small") || lower == "int2" {
        "SMALLSERIAL"
    } else {
        "SERIAL"
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
                serial_type_for(&col.data_type).to_string()
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
            serial_type_for(&column.data_type).to_string()
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

        // Auto-increment: create sequence and wire it up
        if column.auto_increment {
            let seq_name = format!("{}_{}_seq", table, column.name);
            let quoted_seq = self.quote_identifier(&seq_name);
            stmts.push(format!("CREATE SEQUENCE IF NOT EXISTS {};", quoted_seq));
            stmts.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT nextval('{}');",
                tbl, col, seq_name
            ));
            stmts.push(format!(
                "ALTER SEQUENCE {} OWNED BY {}.{};",
                quoted_seq, tbl, col
            ));
        }

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

        // DEFAULT (only if not auto_increment, which is handled above)
        if !column.auto_increment {
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

crate::db::impl_sql_generator_delegation!(PostgresDriver, PostgresSqlGenerator);
