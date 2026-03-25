use crate::db::traits::SqlGenerator;
use crate::db::validate_fk_action;
use crate::models::*;

use super::reader::MySqlDriver;

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

crate::db::impl_sql_generator_delegation!(MySqlDriver, MySqlSqlGenerator);
