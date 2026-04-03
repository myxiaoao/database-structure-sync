export type DbType = "mysql" | "postgresql" | "mariadb";

export const DB_TYPE_LABELS: Record<DbType, string> = {
  mysql: "MySQL",
  postgresql: "PostgreSQL",
  mariadb: "MariaDB",
};

export type DiffType =
  | "table_added"
  | "table_removed"
  | "column_added"
  | "column_removed"
  | "column_modified"
  | "index_added"
  | "index_removed"
  | "index_modified"
  | "foreign_key_added"
  | "foreign_key_removed"
  | "foreign_key_modified"
  | "unique_constraint_added"
  | "unique_constraint_removed"
  | "unique_constraint_modified";

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

export type WarningSeverity = "degraded" | "skipped";

export interface TypeWarning {
  column_name: string;
  source_type: string;
  target_type: string;
  message: string;
  severity: WarningSeverity;
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
  warnings?: TypeWarning[];
}

export interface DiffResult {
  items: DiffItem[];
  source_tables: number;
  target_tables: number;
}
