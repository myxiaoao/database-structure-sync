import { invoke } from "@tauri-apps/api/core";
import type { DiffResult } from "@/types";

export interface CompareOptions {
  sourceId: string;
  targetId: string;
  sourceDatabase?: string;
  targetDatabase?: string;
}

export interface ExecuteOptions {
  targetId: string;
  sqlStatements: string[];
  targetDatabase?: string;
}

export const syncApi = {
  async compare(options: CompareOptions): Promise<DiffResult> {
    return invoke<DiffResult>("compare_databases", {
      sourceId: options.sourceId,
      targetId: options.targetId,
      sourceDatabase: options.sourceDatabase,
      targetDatabase: options.targetDatabase,
    });
  },

  async execute(options: ExecuteOptions): Promise<void> {
    return invoke<void>("execute_sync", {
      targetId: options.targetId,
      sqlStatements: options.sqlStatements,
      targetDatabase: options.targetDatabase,
    });
  },
};
