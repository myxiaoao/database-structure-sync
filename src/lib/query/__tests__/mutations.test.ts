import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import {
  useSaveConnectionMutation,
  useDeleteConnectionMutation,
  useTestConnectionMutation,
  useCompareMutation,
  useExecuteSyncMutation,
} from "../mutations";
import { createWrapper } from "@/test/utils";
import type { ConnectionInput, DiffResult } from "@/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

vi.mock("i18next", () => ({
  default: {
    t: (key: string, opts?: Record<string, unknown>) => {
      if (opts && opts.error !== undefined) return `${key}:${opts.error}`;
      return key;
    },
  },
}));

import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

const mockInvoke = vi.mocked(invoke);
const mockToast = vi.mocked(toast);

const mockConnectionInput: ConnectionInput = {
  name: "Test Connection",
  db_type: "mysql",
  host: "localhost",
  port: 3306,
  username: "root",
  password: "password",
  database: "test_db",
};

const mockDiffResult: DiffResult = {
  items: [
    {
      id: "1",
      diff_type: "TableAdded",
      table_name: "users",
      sql: "CREATE TABLE users (id INT);",
      selected: false,
    },
  ],
  source_tables: 5,
  target_tables: 3,
};

describe("useSaveConnectionMutation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should save connection successfully", async () => {
    const savedConnection = { ...mockConnectionInput, id: "new-id" };
    mockInvoke.mockResolvedValue(savedConnection);

    const { result } = renderHook(() => useSaveConnectionMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(mockConnectionInput);
    });

    expect(mockInvoke).toHaveBeenCalledWith("save_connection", {
      input: mockConnectionInput,
    });
    expect(mockToast.success).toHaveBeenCalledWith("connection.saveSuccess");
  });

  it("should show error toast on failure", async () => {
    mockInvoke.mockRejectedValue(new Error("Save failed"));

    const { result } = renderHook(() => useSaveConnectionMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync(mockConnectionInput);
      } catch {
        // Expected to throw
      }
    });

    expect(mockToast.error).toHaveBeenCalledWith("connection.saveFailed:Save failed");
  });
});

describe("useDeleteConnectionMutation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should delete connection successfully", async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useDeleteConnectionMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync("test-id");
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_connection", { id: "test-id" });
    expect(mockToast.success).toHaveBeenCalledWith("connection.deleteSuccess");
  });

  it("should show error toast on failure", async () => {
    mockInvoke.mockRejectedValue(new Error("Delete failed"));

    const { result } = renderHook(() => useDeleteConnectionMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync("test-id");
      } catch {
        // Expected to throw
      }
    });

    expect(mockToast.error).toHaveBeenCalledWith("connection.deleteFailed:Delete failed");
  });
});

describe("useTestConnectionMutation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should test connection successfully", async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useTestConnectionMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync(mockConnectionInput);
    });

    expect(mockInvoke).toHaveBeenCalledWith("test_connection", {
      input: mockConnectionInput,
    });
    expect(mockToast.success).toHaveBeenCalledWith("connection.testSuccess");
  });

  it("should show error toast on failure", async () => {
    mockInvoke.mockRejectedValue(new Error("Connection refused"));

    const { result } = renderHook(() => useTestConnectionMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync(mockConnectionInput);
      } catch {
        // Expected to throw
      }
    });

    expect(mockToast.error).toHaveBeenCalledWith("connection.testFailed:Connection refused");
  });
});

describe("useCompareMutation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should compare databases successfully", async () => {
    mockInvoke.mockResolvedValue(mockDiffResult);

    const { result } = renderHook(() => useCompareMutation(), {
      wrapper: createWrapper(),
    });

    let compareResult: DiffResult | undefined;
    await act(async () => {
      compareResult = await result.current.mutateAsync({
        sourceId: "source-id",
        targetId: "target-id",
      });
    });

    expect(mockInvoke).toHaveBeenCalledWith("compare_databases", {
      sourceId: "source-id",
      targetId: "target-id",
      sourceDatabase: undefined,
      targetDatabase: undefined,
    });
    expect(compareResult).toEqual(mockDiffResult);
  });

  it("should show error toast on failure", async () => {
    mockInvoke.mockRejectedValue(new Error("Comparison failed"));

    const { result } = renderHook(() => useCompareMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          sourceId: "source-id",
          targetId: "target-id",
        });
      } catch {
        // Expected to throw
      }
    });

    expect(mockToast.error).toHaveBeenCalledWith("sync.compareFailed:Comparison failed");
  });
});

describe("useExecuteSyncMutation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should execute sync successfully", async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useExecuteSyncMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        targetId: "target-id",
        sqlStatements: ["CREATE TABLE users (id INT);"],
      });
    });

    expect(mockInvoke).toHaveBeenCalledWith("execute_sync", {
      targetId: "target-id",
      sqlStatements: ["CREATE TABLE users (id INT);"],
      targetDatabase: undefined,
    });
    expect(mockToast.success).toHaveBeenCalledWith("sync.executeSuccess");
  });

  it("should show error toast on failure", async () => {
    mockInvoke.mockRejectedValue(new Error("Sync failed"));

    const { result } = renderHook(() => useExecuteSyncMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          targetId: "target-id",
          sqlStatements: ["INVALID SQL"],
        });
      } catch {
        // Expected to throw
      }
    });

    expect(mockToast.error).toHaveBeenCalledWith("sync.executeFailed:Sync failed");
  });

  it("should handle string errors from Tauri invoke", async () => {
    mockInvoke.mockRejectedValue("Failed to execute: ALTER TABLE ...\nError: Access denied");

    const { result } = renderHook(() => useExecuteSyncMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      try {
        await result.current.mutateAsync({
          targetId: "target-id",
          sqlStatements: ["ALTER TABLE ..."],
        });
      } catch {
        // Expected to throw
      }
    });

    expect(mockToast.error).toHaveBeenCalledWith(
      "sync.executeFailed:Failed to execute: ALTER TABLE ...\nError: Access denied"
    );
  });

  it("should pass targetDatabase parameter", async () => {
    mockInvoke.mockResolvedValue(undefined);

    const { result } = renderHook(() => useExecuteSyncMutation(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.mutateAsync({
        targetId: "target-id",
        sqlStatements: ["CREATE TABLE test (id INT);"],
        targetDatabase: "custom_db",
      });
    });

    expect(mockInvoke).toHaveBeenCalledWith("execute_sync", {
      targetId: "target-id",
      sqlStatements: ["CREATE TABLE test (id INT);"],
      targetDatabase: "custom_db",
    });
  });
});
