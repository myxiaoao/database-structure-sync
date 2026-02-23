import { describe, it, expect, vi, beforeEach } from "vitest";
import { connectionsApi } from "../connections";
import type { Connection, ConnectionInput } from "@/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

const mockConnection: Connection = {
  id: "test-id",
  name: "Test Connection",
  db_type: "mysql",
  host: "localhost",
  port: 3306,
  username: "root",
  password: "password",
  database: "test_db",
};

const mockConnectionInput: ConnectionInput = {
  name: "Test Connection",
  db_type: "mysql",
  host: "localhost",
  port: 3306,
  username: "root",
  password: "password",
  database: "test_db",
};

describe("connectionsApi", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("list", () => {
    it("should call invoke with list_connections command", async () => {
      mockInvoke.mockResolvedValue([mockConnection]);

      const result = await connectionsApi.list();

      expect(mockInvoke).toHaveBeenCalledWith("list_connections");
      expect(result).toEqual([mockConnection]);
    });

    it("should return empty array when no connections exist", async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await connectionsApi.list();

      expect(result).toEqual([]);
    });

    it("should propagate errors from invoke", async () => {
      mockInvoke.mockRejectedValue(new Error("Failed to list connections"));

      await expect(connectionsApi.list()).rejects.toThrow("Failed to list connections");
    });
  });

  describe("get", () => {
    it("should call invoke with get_connection command and id", async () => {
      mockInvoke.mockResolvedValue(mockConnection);

      const result = await connectionsApi.get("test-id");

      expect(mockInvoke).toHaveBeenCalledWith("get_connection", { id: "test-id" });
      expect(result).toEqual(mockConnection);
    });

    it("should return null when connection not found", async () => {
      mockInvoke.mockResolvedValue(null);

      const result = await connectionsApi.get("non-existent-id");

      expect(result).toBeNull();
    });
  });

  describe("save", () => {
    it("should call invoke with save_connection command and input", async () => {
      mockInvoke.mockResolvedValue(mockConnection);

      const result = await connectionsApi.save(mockConnectionInput);

      expect(mockInvoke).toHaveBeenCalledWith("save_connection", {
        input: mockConnectionInput,
      });
      expect(result).toEqual(mockConnection);
    });

    it("should handle update (input with id)", async () => {
      const inputWithId = { ...mockConnectionInput, id: "test-id" };
      mockInvoke.mockResolvedValue(mockConnection);

      await connectionsApi.save(inputWithId);

      expect(mockInvoke).toHaveBeenCalledWith("save_connection", {
        input: inputWithId,
      });
    });
  });

  describe("delete", () => {
    it("should call invoke with delete_connection command and id", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await connectionsApi.delete("test-id");

      expect(mockInvoke).toHaveBeenCalledWith("delete_connection", { id: "test-id" });
    });

    it("should propagate errors from invoke", async () => {
      mockInvoke.mockRejectedValue(new Error("Connection not found"));

      await expect(connectionsApi.delete("non-existent-id")).rejects.toThrow(
        "Connection not found"
      );
    });
  });

  describe("test", () => {
    it("should call invoke with test_connection command and input", async () => {
      mockInvoke.mockResolvedValue(undefined);

      await connectionsApi.test(mockConnectionInput);

      expect(mockInvoke).toHaveBeenCalledWith("test_connection", {
        input: mockConnectionInput,
      });
    });

    it("should throw error when connection test fails", async () => {
      mockInvoke.mockRejectedValue(new Error("Connection refused"));

      await expect(connectionsApi.test(mockConnectionInput)).rejects.toThrow("Connection refused");
    });
  });

  describe("listDatabases", () => {
    it("should call invoke with list_databases command and connectionId", async () => {
      const databases = ["db1", "db2", "db3"];
      mockInvoke.mockResolvedValue(databases);

      const result = await connectionsApi.listDatabases("test-id");

      expect(mockInvoke).toHaveBeenCalledWith("list_databases", {
        connectionId: "test-id",
      });
      expect(result).toEqual(databases);
    });

    it("should return empty array when no databases found", async () => {
      mockInvoke.mockResolvedValue([]);

      const result = await connectionsApi.listDatabases("test-id");

      expect(result).toEqual([]);
    });
  });
});
