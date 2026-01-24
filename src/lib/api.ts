import { invoke } from "@tauri-apps/api/core";
import type { Connection, ConnectionInput, DiffResult } from "@/types";

export type {
  Connection,
  ConnectionInput,
  DiffResult,
  DiffItem,
  DbType,
  SshAuthMethod,
} from "@/types";

export const api = {
  async listConnections(): Promise<Connection[]> {
    return invoke<Connection[]>("list_connections");
  },

  async getConnection(id: string): Promise<Connection | null> {
    return invoke<Connection | null>("get_connection", { id });
  },

  async saveConnection(input: ConnectionInput): Promise<Connection> {
    return invoke<Connection>("save_connection", { input });
  },

  async deleteConnection(id: string): Promise<void> {
    return invoke<void>("delete_connection", { id });
  },

  async testConnection(input: ConnectionInput): Promise<void> {
    return invoke<void>("test_connection", { input });
  },

  async compareDatabases(sourceId: string, targetId: string): Promise<DiffResult> {
    return invoke<DiffResult>("compare_databases", { sourceId, targetId });
  },

  async executeSync(targetId: string, sqlStatements: string[]): Promise<void> {
    return invoke<void>("execute_sync", { targetId, sqlStatements });
  },
};
