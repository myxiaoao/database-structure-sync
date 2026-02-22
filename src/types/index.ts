export type DbType = "MySQL" | "PostgreSQL" | "MariaDB";
export type SshAuthMethod = "Password" | "PrivateKey";

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

interface ConnectionBase {
  name: string;
  db_type: DbType;
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  ssh_enabled: boolean;
  ssh_host?: string;
  ssh_port?: number;
  ssh_username?: string;
  ssh_auth_method?: SshAuthMethod;
  ssh_password?: string;
  ssh_private_key_path?: string;
  ssh_passphrase?: string;
  ssl_enabled: boolean;
  ssl_ca_cert_path?: string;
  ssl_client_cert_path?: string;
  ssl_client_key_path?: string;
  ssl_verify_server_cert?: boolean;
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
