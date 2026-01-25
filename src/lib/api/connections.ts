import { invoke } from "@tauri-apps/api/core";
import type { Connection, ConnectionInput } from "@/types";

export const connectionsApi = {
  async list(): Promise<Connection[]> {
    return invoke<Connection[]>("list_connections");
  },

  async get(id: string): Promise<Connection | null> {
    return invoke<Connection | null>("get_connection", { id });
  },

  async save(input: ConnectionInput): Promise<Connection> {
    return invoke<Connection>("save_connection", { input });
  },

  async delete(id: string): Promise<void> {
    return invoke<void>("delete_connection", { id });
  },

  async test(input: ConnectionInput): Promise<void> {
    return invoke<void>("test_connection", { input });
  },

  async listDatabases(connectionId: string): Promise<string[]> {
    return invoke<string[]>("list_databases", { connectionId });
  },
};
