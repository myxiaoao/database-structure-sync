import { invoke } from '@tauri-apps/api/core';

export interface Connection {
  id: string;
  name: string;
  db_type: 'MySQL' | 'PostgreSQL' | 'MariaDB';
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  ssh_enabled: boolean;
  ssh_host?: string;
  ssh_port?: number;
  ssh_username?: string;
  ssh_auth_method?: 'Password' | 'PrivateKey';
  ssh_password?: string;
  ssh_private_key_path?: string;
  ssh_passphrase?: string;
  ssl_enabled: boolean;
  ssl_ca_cert_path?: string;
  ssl_client_cert_path?: string;
  ssl_client_key_path?: string;
  ssl_verify_server_cert?: boolean;
}

export interface ConnectionInput {
  id?: string;
  name: string;
  db_type: 'MySQL' | 'PostgreSQL' | 'MariaDB';
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
  ssh_enabled: boolean;
  ssh_host?: string;
  ssh_port?: number;
  ssh_username?: string;
  ssh_auth_method?: 'Password' | 'PrivateKey';
  ssh_password?: string;
  ssh_private_key_path?: string;
  ssh_passphrase?: string;
  ssl_enabled: boolean;
  ssl_ca_cert_path?: string;
  ssl_client_cert_path?: string;
  ssl_client_key_path?: string;
  ssl_verify_server_cert?: boolean;
}

export interface DiffItem {
  id: string;
  diff_type:
    | 'TableAdded'
    | 'TableRemoved'
    | 'ColumnAdded'
    | 'ColumnRemoved'
    | 'ColumnModified'
    | 'IndexAdded'
    | 'IndexRemoved'
    | 'IndexModified'
    | 'ForeignKeyAdded'
    | 'ForeignKeyRemoved'
    | 'UniqueConstraintAdded'
    | 'UniqueConstraintRemoved';
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

export const api = {
  async listConnections(): Promise<Connection[]> {
    return invoke<Connection[]>('list_connections');
  },

  async getConnection(id: string): Promise<Connection | null> {
    return invoke<Connection | null>('get_connection', { id });
  },

  async saveConnection(input: ConnectionInput): Promise<Connection> {
    return invoke<Connection>('save_connection', { input });
  },

  async deleteConnection(id: string): Promise<void> {
    return invoke<void>('delete_connection', { id });
  },

  async testConnection(input: ConnectionInput): Promise<void> {
    return invoke<void>('test_connection', { input });
  },

  async compareDatabases(sourceId: string, targetId: string): Promise<DiffResult> {
    return invoke<DiffResult>('compare_databases', { sourceId, targetId });
  },

  async executeSync(targetId: string, sqlStatements: string[]): Promise<void> {
    return invoke<void>('execute_sync', { targetId, sqlStatements });
  },
};
