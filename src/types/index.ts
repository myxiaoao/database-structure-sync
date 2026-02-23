export type DbType = "mysql" | "postgresql" | "mariadb";

export const DB_TYPE_LABELS: Record<DbType, string> = {
  mysql: "MySQL",
  postgresql: "PostgreSQL",
  mariadb: "MariaDB",
};

export type DiffType =
  | "TableAdded"
  | "TableRemoved"
  | "ColumnAdded"
  | "ColumnRemoved"
  | "ColumnModified"
  | "IndexAdded"
  | "IndexRemoved"
  | "IndexModified"
  | "ForeignKeyAdded"
  | "ForeignKeyRemoved"
  | "ForeignKeyModified"
  | "UniqueConstraintAdded"
  | "UniqueConstraintRemoved"
  | "UniqueConstraintModified";

export type SshAuthMethodConfig =
  | { password: { password: string } }
  | { privatekey: { private_key_path: string; passphrase?: string } };

export interface SshConfig {
  enabled: boolean;
  host: string;
  port: number;
  username: string;
  auth_method: SshAuthMethodConfig;
}

export interface SslConfig {
  enabled: boolean;
  ca_cert_path?: string;
  client_cert_path?: string;
  client_key_path?: string;
  verify_server: boolean;
}

interface ConnectionBase {
  name: string;
  db_type: DbType;
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  ssh_config?: SshConfig;
  ssl_config?: SslConfig;
}

export interface Connection extends ConnectionBase {
  id: string;
}

export interface ConnectionInput extends ConnectionBase {
  id?: string;
}

export interface DiffItem {
  id: string;
  diff_type: DiffType;
  table_name: string;
  object_name?: string;
  source_def?: string;
  target_def?: string;
  sql: string;
  selected: boolean;
}

export interface DiffResult {
  items: DiffItem[];
  source_tables: number;
  target_tables: number;
}
