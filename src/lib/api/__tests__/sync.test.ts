import { describe, it, expect, vi, beforeEach } from "vitest";
import { syncApi } from "../sync";
import type { DiffResult } from "@/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

const mockDiffResult: DiffResult = {
  items: [
    {
      id: "1",
      diff_type: "TableAdded",
      table_name: "users",
      sql: "CREATE TABLE users (id INT);",
      selected: false,
    },
    {
      id: "2",
      diff_type: "ColumnAdded",
      table_name: "posts",
      object_name: "title",
      sql: "ALTER TABLE posts ADD COLUMN title VARCHAR(255);",
      selected: false,
    },
  ],
  source_tables: 5,
  target_tables: 3,
};

describe("syncApi", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("compare", () => {
    it("should call invoke with compare_databases command", async () => {
      mockInvoke.mockResolvedValue(mockDiffResult);

      const result = await syncApi.compare({
        sourceId: "source-id",
        targetId: "target-id",
      });

      expect(mockInvoke).toHaveBeenCalledWith("compare_databases", {
        sourceId: "source-id",
        targetId: "target-id",
        sourceDatabase: undefined,
        targetDatabase: undefined,
      });
      expect(result).toEqual(mockDiffResult);
    });

    it("should pass optional database parameters", async () => {
      mockInvoke.mockResolvedValue(mockDiffResult);

      await syncApi.compare({
        sourceId: "source-id",
        targetId: "target-id",
        sourceDatabase: "source_db",
        targetDatabase: "target_db",
      });

      expect(mockInvoke).toHaveBeenCalledWith("compare_databases", {
        sourceId: "source-id",
        targetId: "target-id",
        sourceDatabase: "source_db",
        targetDatabase: "target_db",
      });
    });

    it("should return diff result with items", async () => {
      mockInvoke.mockResolvedValue(mockDiffResult);

      const result = await syncApi.compare({
        sourceId: "source-id",
        targetId: "target-id",
      });

      expect(result.items).toHaveLength(2);
      expect(result.source_tables).toBe(5);
      expect(result.target_tables).toBe(3);
    });

    it("should return empty diff when databases are identical", async () => {
      const emptyDiff: DiffResult = {
        items: [],
        source_tables: 5,
        target_tables: 5,
      };
      mockInvoke.mockResolvedValue(emptyDiff);

      const result = await syncApi.compare({
        sourceId: "source-id",
        targetId: "target-id",
      });

      expect(result.items).toHaveLength(0);
    });

    it("should propagate errors from invoke", async () => {
      mockInvoke.mockRejectedValue(new Error("Connection failed"));

      await expect(
        syncApi.compare({
          sourceId: "source-id",
          targetId: "target-id",
        })
      ).rejects.toThrow("Connection failed");
    });
  });

  describe("execute", () => {
    it("should call invoke with execute_sync command", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await syncApi.execute({
        targetId: "target-id",
        sqlStatements: ["CREATE TABLE users (id INT);"],
      });

      expect(mockInvoke).toHaveBeenCalledWith("execute_sync", {
        targetId: "target-id",
        sqlStatements: ["CREATE TABLE users (id INT);"],
        targetDatabase: undefined,
      });
    });

    it("should pass multiple SQL statements", async () => {
      mockInvoke.mockResolvedValue(undefined);

      const statements = [
        "CREATE TABLE users (id INT);",
        "ALTER TABLE posts ADD COLUMN title VARCHAR(255);",
        "CREATE INDEX idx_user_id ON posts(user_id);",
      ];

      await syncApi.execute({
        targetId: "target-id",
        sqlStatements: statements,
      });

      expect(mockInvoke).toHaveBeenCalledWith("execute_sync", {
        targetId: "target-id",
        sqlStatements: statements,
        targetDatabase: undefined,
      });
    });

    it("should pass optional targetDatabase parameter", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await syncApi.execute({
        targetId: "target-id",
        sqlStatements: ["CREATE TABLE test (id INT);"],
        targetDatabase: "custom_db",
      });

      expect(mockInvoke).toHaveBeenCalledWith("execute_sync", {
        targetId: "target-id",
        sqlStatements: ["CREATE TABLE test (id INT);"],
        targetDatabase: "custom_db",
      });
    });

    it("should propagate errors from invoke", async () => {
      mockInvoke.mockRejectedValue(new Error("SQL syntax error"));

      await expect(
        syncApi.execute({
          targetId: "target-id",
          sqlStatements: ["INVALID SQL"],
        })
      ).rejects.toThrow("SQL syntax error");
    });

    it("should handle empty statements array", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await syncApi.execute({
        targetId: "target-id",
        sqlStatements: [],
      });

      expect(mockInvoke).toHaveBeenCalledWith("execute_sync", {
        targetId: "target-id",
        sqlStatements: [],
        targetDatabase: undefined,
      });
    });
  });
});
