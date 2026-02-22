import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSync } from "../useSync";
import { createWrapper } from "@/test/utils";
import type { Connection, DiffResult } from "@/types";

const mockConnections: Connection[] = [
  {
    id: "source-id",
    name: "Source DB",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "password",
    database: "source_db",
    ssh_enabled: false,
    ssl_enabled: false,
  },
  {
    id: "target-id",
    name: "Target DB",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "password",
    database: "target_db",
    ssh_enabled: false,
    ssl_enabled: false,
  },
  {
    id: "no-db-id",
    name: "No DB Connection",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "password",
    database: "",
    ssh_enabled: false,
    ssl_enabled: false,
  },
];

const mockDiffResult: DiffResult = {
  items: [
    {
      id: "diff-1",
      diff_type: "TableAdded",
      table_name: "users",
      sql: "CREATE TABLE users (id INT PRIMARY KEY);",
      selected: false,
    },
    {
      id: "diff-2",
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

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

describe("useSync", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([]);
  });

  it("should initialize with default state", () => {
    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    expect(result.current.sourceId).toBe("");
    expect(result.current.targetId).toBe("");
    expect(result.current.sourceDb).toBe("");
    expect(result.current.targetDb).toBe("");
    expect(result.current.diffResult).toBe(null);
    expect(result.current.selectedItems.size).toBe(0);
    expect(result.current.selectedItem).toBe(null);
    expect(result.current.canCompare).toBeFalsy();
  });

  it("should update source and target IDs", () => {
    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setSourceId("source-id");
    });

    expect(result.current.sourceId).toBe("source-id");
    expect(result.current.sourceConnection).toEqual(mockConnections[0]);

    act(() => {
      result.current.setTargetId("target-id");
    });

    expect(result.current.targetId).toBe("target-id");
    expect(result.current.targetConnection).toEqual(mockConnections[1]);
  });

  it("should enable comparison when both connections with databases are selected", () => {
    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    expect(result.current.canCompare).toBeFalsy();

    act(() => {
      result.current.setSourceId("source-id");
    });

    expect(result.current.canCompare).toBeFalsy();

    act(() => {
      result.current.setTargetId("target-id");
    });

    expect(result.current.canCompare).toBeTruthy();
  });

  it("should require database selection when connection has no database", () => {
    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setSourceId("no-db-id");
      result.current.setTargetId("target-id");
    });

    expect(result.current.sourceNeedsDbSelect).toBe(true);
    expect(result.current.canCompare).toBeFalsy();

    act(() => {
      result.current.setSourceDb("selected_db");
    });

    expect(result.current.canCompare).toBeTruthy();
  });

  it("should reset database selection when connection changes", () => {
    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setSourceId("no-db-id");
      result.current.setSourceDb("selected_db");
    });

    expect(result.current.sourceDb).toBe("selected_db");

    act(() => {
      result.current.setSourceId("source-id");
    });

    expect(result.current.sourceDb).toBe("");
  });

  it("should perform comparison and store results", async () => {
    mockInvoke.mockResolvedValue(mockDiffResult);

    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setSourceId("source-id");
      result.current.setTargetId("target-id");
    });

    await act(async () => {
      await result.current.handleCompare();
    });

    expect(mockInvoke).toHaveBeenCalledWith("compare_databases", {
      sourceId: "source-id",
      targetId: "target-id",
      sourceDatabase: undefined,
      targetDatabase: undefined,
    });

    expect(result.current.diffResult).toEqual(mockDiffResult);
  });

  it("should select and deselect items", async () => {
    mockInvoke.mockResolvedValue(mockDiffResult);

    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setSourceId("source-id");
      result.current.setTargetId("target-id");
    });

    await act(async () => {
      await result.current.handleCompare();
    });

    act(() => {
      result.current.handleSelectAll();
    });

    expect(result.current.selectedItems.size).toBe(2);
    expect(result.current.selectedItems.has("diff-1")).toBe(true);
    expect(result.current.selectedItems.has("diff-2")).toBe(true);

    act(() => {
      result.current.handleDeselectAll();
    });

    expect(result.current.selectedItems.size).toBe(0);
  });

  it("should generate selected SQL from selected items", async () => {
    mockInvoke.mockResolvedValue(mockDiffResult);

    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setSourceId("source-id");
      result.current.setTargetId("target-id");
    });

    await act(async () => {
      await result.current.handleCompare();
    });

    act(() => {
      result.current.setSelectedItems(new Set(["diff-1"]));
    });

    // Should contain header, SQL body, and footer
    expect(result.current.selectedSql).toContain("-- Database Structure Sync v");
    expect(result.current.selectedSql).toContain("-- Generation Time:");
    expect(result.current.selectedSql).toContain("-- Source:");
    expect(result.current.selectedSql).toContain("-- Target:");
    expect(result.current.selectedSql).toContain("CREATE TABLE users (id INT PRIMARY KEY);");
    expect(result.current.selectedSql).toContain("-- End of synchronization script");

    act(() => {
      result.current.handleSelectAll();
    });

    expect(result.current.selectedSql).toContain("CREATE TABLE users");
    expect(result.current.selectedSql).toContain("ALTER TABLE posts");
    expect(result.current.selectedSql).toContain("Changes:         2 item(s)");
  });

  it("should execute sync and refresh comparison", async () => {
    mockInvoke
      .mockResolvedValueOnce(mockDiffResult) // compare
      .mockResolvedValueOnce(undefined) // execute
      .mockResolvedValueOnce({ ...mockDiffResult, items: [] }); // refresh compare

    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setSourceId("source-id");
      result.current.setTargetId("target-id");
    });

    await act(async () => {
      await result.current.handleCompare();
    });

    act(() => {
      result.current.setSelectedItems(new Set(["diff-1"]));
    });

    await act(async () => {
      await result.current.handleExecute();
    });

    expect(mockInvoke).toHaveBeenCalledWith("execute_sync", {
      targetId: "target-id",
      sqlStatements: ["CREATE TABLE users (id INT PRIMARY KEY);"],
      targetDatabase: undefined,
    });
  });

  it("should set selected item for detail view", async () => {
    mockInvoke.mockResolvedValue(mockDiffResult);

    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.setSourceId("source-id");
      result.current.setTargetId("target-id");
    });

    await act(async () => {
      await result.current.handleCompare();
    });

    act(() => {
      result.current.setSelectedItem(mockDiffResult.items[0]);
    });

    expect(result.current.selectedItem).toEqual(mockDiffResult.items[0]);
  });

  it("should provide loading states", () => {
    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    expect(result.current.isComparing).toBe(false);
    expect(result.current.isExecuting).toBe(false);
    expect(result.current.loadingSourceDbs).toBe(false);
    expect(result.current.loadingTargetDbs).toBe(false);
  });

  it("should not compare when canCompare is false", async () => {
    const { result } = renderHook(() => useSync({ connections: mockConnections }), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.handleCompare();
    });

    expect(mockInvoke).not.toHaveBeenCalledWith("compare_databases", expect.anything());
  });
});
