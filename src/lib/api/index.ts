export { connectionsApi } from "./connections";
export { syncApi } from "./sync";
export type { CompareOptions, ExecuteOptions } from "./sync";

// Re-export types for convenience
export type {
  Connection,
  ConnectionInput,
  DiffResult,
  DiffItem,
  DbType,
  SshAuthMethod,
} from "@/types";
