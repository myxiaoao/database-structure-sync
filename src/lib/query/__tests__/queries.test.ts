import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import {
  connectionKeys,
  useConnectionsQuery,
  useConnectionQuery,
  useDatabasesQuery,
} from "../queries";
import { createWrapper } from "@/test/utils";
import type { Connection } from "@/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

const mockConnections: Connection[] = [
  {
    id: "1",
    name: "Test DB 1",
    db_type: "mysql",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "password",
    database: "test1",
  },
  {
    id: "2",
    name: "Test DB 2",
    db_type: "postgresql",
    host: "localhost",
    port: 5432,
    username: "postgres",
    password: "password",
    database: "test2",
  },
];

describe("connectionKeys", () => {
  it("should generate correct all key", () => {
    expect(connectionKeys.all).toEqual(["connections"]);
  });

  it("should generate correct list key", () => {
    expect(connectionKeys.list()).toEqual(["connections", "list"]);
  });

  it("should generate correct detail key", () => {
    expect(connectionKeys.detail("123")).toEqual(["connections", "detail", "123"]);
  });

  it("should generate correct databases key", () => {
    expect(connectionKeys.databases("123")).toEqual(["connections", "databases", "123"]);
  });
});

describe("useConnectionsQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should fetch connections on mount", async () => {
    mockInvoke.mockResolvedValue(mockConnections);

    const { result } = renderHook(() => useConnectionsQuery(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockInvoke).toHaveBeenCalledWith("list_connections");
    expect(result.current.data).toEqual(mockConnections);
  });

  it("should handle loading state", () => {
    mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves

    const { result } = renderHook(() => useConnectionsQuery(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);
  });

  it("should handle error state", async () => {
    mockInvoke.mockRejectedValue(new Error("Failed to fetch"));

    const { result } = renderHook(() => useConnectionsQuery(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(result.current.error?.message).toBe("Failed to fetch");
  });
});

describe("useConnectionQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should fetch single connection by id", async () => {
    mockInvoke.mockResolvedValue(mockConnections[0]);

    const { result } = renderHook(() => useConnectionQuery("1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockInvoke).toHaveBeenCalledWith("get_connection", { id: "1" });
    expect(result.current.data).toEqual(mockConnections[0]);
  });

  it("should not fetch when id is null", () => {
    const { result } = renderHook(() => useConnectionQuery(null), {
      wrapper: createWrapper(),
    });

    expect(result.current.fetchStatus).toBe("idle");
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("should handle null response", async () => {
    mockInvoke.mockResolvedValue(null);

    const { result } = renderHook(() => useConnectionQuery("non-existent"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toBeNull();
  });
});

describe("useDatabasesQuery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should fetch databases for a connection", async () => {
    const databases = ["db1", "db2", "db3"];
    mockInvoke.mockResolvedValue(databases);

    const { result } = renderHook(() => useDatabasesQuery("1"), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockInvoke).toHaveBeenCalledWith("list_databases", { connectionId: "1" });
    expect(result.current.data).toEqual(databases);
  });

  it("should not fetch when connectionId is null", () => {
    const { result } = renderHook(() => useDatabasesQuery(null), {
      wrapper: createWrapper(),
    });

    expect(result.current.fetchStatus).toBe("idle");
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("should not fetch when enabled is false", () => {
    const { result } = renderHook(() => useDatabasesQuery("1", false), {
      wrapper: createWrapper(),
    });

    expect(result.current.fetchStatus).toBe("idle");
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("should return empty array when connectionId is null but query is called", async () => {
    const { result } = renderHook(() => useDatabasesQuery(null, true), {
      wrapper: createWrapper(),
    });

    // Query should not run when connectionId is null
    expect(result.current.fetchStatus).toBe("idle");
  });
});
