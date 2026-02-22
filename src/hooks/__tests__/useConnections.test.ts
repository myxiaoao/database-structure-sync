import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useConnections } from "../useConnections";
import { createWrapper } from "@/test/utils";
import type { Connection, ConnectionInput } from "@/types";

const mockConnection: Connection = {
  id: "test-id-1",
  name: "Test Connection",
  db_type: "MySQL",
  host: "localhost",
  port: 3306,
  username: "root",
  password: "password",
  database: "test_db",
  ssh_enabled: false,
  ssl_enabled: false,
};

const mockConnections: Connection[] = [
  mockConnection,
  {
    id: "test-id-2",
    name: "Second Connection",
    db_type: "PostgreSQL",
    host: "localhost",
    port: 5432,
    username: "postgres",
    password: "password",
    database: "test_db",
    ssh_enabled: false,
    ssl_enabled: false,
  },
];

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

describe("useConnections", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([]);
  });

  it("should initialize with default state", async () => {
    mockInvoke.mockResolvedValue(mockConnections);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isFormOpen).toBe(false);
    expect(result.current.editingConnection).toBe(null);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.connections).toEqual(mockConnections);
  });

  it("should fetch connections on mount", async () => {
    mockInvoke.mockResolvedValue(mockConnections);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(mockInvoke).toHaveBeenCalledWith("list_connections");
    expect(result.current.connections).toEqual(mockConnections);
  });

  it("should handle error when fetching connections fails", async () => {
    const error = new Error("Failed to fetch connections");
    mockInvoke.mockRejectedValue(error);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.error).toBe("Failed to fetch connections");
    });
  });

  it("should open form for new connection", async () => {
    mockInvoke.mockResolvedValue([]);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.openNewConnection();
    });

    expect(result.current.isFormOpen).toBe(true);
    expect(result.current.editingConnection).toBe(null);
  });

  it("should open form for editing connection", async () => {
    mockInvoke.mockResolvedValue(mockConnections);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    act(() => {
      result.current.openEditConnection(mockConnection);
    });

    expect(result.current.isFormOpen).toBe(true);
    expect(result.current.editingConnection).toEqual(mockConnection);
  });

  it("should close form and reset editing connection", async () => {
    mockInvoke.mockResolvedValue([]);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.openEditConnection(mockConnection);
    });

    expect(result.current.isFormOpen).toBe(true);

    act(() => {
      result.current.closeForm();
    });

    expect(result.current.isFormOpen).toBe(false);
    expect(result.current.editingConnection).toBe(null);
  });

  it("should save connection and close form on success", async () => {
    const newConnection: ConnectionInput = {
      name: "New Connection",
      db_type: "MySQL",
      host: "localhost",
      port: 3306,
      username: "root",
      password: "password",
      database: "new_db",
      ssh_enabled: false,
      ssl_enabled: false,
    };

    mockInvoke
      .mockResolvedValueOnce([]) // Initial list
      .mockResolvedValueOnce({ ...newConnection, id: "new-id" }) // Save
      .mockResolvedValueOnce([{ ...newConnection, id: "new-id" }]); // Refetch

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.openNewConnection();
    });

    await act(async () => {
      await result.current.saveConnection(newConnection);
    });

    expect(result.current.isFormOpen).toBe(false);
    expect(mockInvoke).toHaveBeenCalledWith("save_connection", { input: newConnection });
  });

  it("should delete connection", async () => {
    mockInvoke
      .mockResolvedValueOnce(mockConnections) // Initial list
      .mockResolvedValueOnce(undefined) // Delete
      .mockResolvedValueOnce([mockConnections[1]]); // Refetch

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.deleteConnection("test-id-1");
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_connection", { id: "test-id-1" });
  });

  it("should test connection", async () => {
    const connectionInput: ConnectionInput = {
      name: "Test Connection",
      db_type: "MySQL",
      host: "localhost",
      port: 3306,
      username: "root",
      password: "password",
      database: "test_db",
      ssh_enabled: false,
      ssl_enabled: false,
    };

    mockInvoke.mockResolvedValueOnce([]).mockResolvedValueOnce(undefined);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.testConnection(connectionInput);
    });

    expect(mockInvoke).toHaveBeenCalledWith("test_connection", { input: connectionInput });
  });

  it("should provide loading states for mutations", async () => {
    mockInvoke.mockResolvedValue([]);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    expect(result.current.isSaving).toBe(false);
    expect(result.current.isDeleting).toBe(false);
    expect(result.current.isTesting).toBe(false);
  });

  it("should keep form open when saveConnection fails", async () => {
    const newConnection: ConnectionInput = {
      name: "New Connection",
      db_type: "MySQL",
      host: "localhost",
      port: 3306,
      username: "root",
      password: "password",
      database: "new_db",
      ssh_enabled: false,
      ssl_enabled: false,
    };

    mockInvoke
      .mockResolvedValueOnce([]) // Initial list
      .mockRejectedValueOnce(new Error("Save failed")); // Save fails

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.openNewConnection();
    });

    expect(result.current.isFormOpen).toBe(true);

    // Confirm the error actually propagates (not silently swallowed)
    await expect(
      act(async () => {
        await result.current.saveConnection(newConnection);
      })
    ).rejects.toThrow("Save failed");

    // Form should still be open because save failed — closeForm was never called
    expect(result.current.isFormOpen).toBe(true);
  });

  it("should propagate error when deleteConnection fails", async () => {
    mockInvoke
      .mockResolvedValueOnce(mockConnections) // Initial list
      .mockRejectedValueOnce(new Error("Delete failed")); // Delete fails

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await expect(
      act(async () => {
        await result.current.deleteConnection("test-id-1");
      })
    ).rejects.toThrow("Delete failed");
  });

  it("should reset editingConnection when openNewConnection is called after openEditConnection", async () => {
    mockInvoke.mockResolvedValue(mockConnections);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    act(() => {
      result.current.openEditConnection(mockConnection);
    });

    expect(result.current.editingConnection).toEqual(mockConnection);

    act(() => {
      result.current.openNewConnection();
    });

    expect(result.current.isFormOpen).toBe(true);
    expect(result.current.editingConnection).toBe(null);
  });

  it("should call save_connection with id for update path", async () => {
    const updateInput: ConnectionInput = {
      id: "test-id-1",
      name: "Updated Connection",
      db_type: "MySQL",
      host: "localhost",
      port: 3306,
      username: "root",
      password: "password",
      database: "test_db",
      ssh_enabled: false,
      ssl_enabled: false,
    };

    mockInvoke
      .mockResolvedValueOnce([]) // Initial list
      .mockResolvedValueOnce({ ...updateInput, id: "test-id-1" }) // Save (update)
      .mockResolvedValueOnce([{ ...updateInput, id: "test-id-1" }]); // Refetch

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    act(() => {
      result.current.openEditConnection(mockConnection);
    });

    await act(async () => {
      await result.current.saveConnection(updateInput);
    });

    expect(mockInvoke).toHaveBeenCalledWith("save_connection", { input: updateInput });
    expect(result.current.isFormOpen).toBe(false);
  });

  it("should handle test connection failure", async () => {
    const connectionInput: ConnectionInput = {
      name: "Test Connection",
      db_type: "MySQL",
      host: "localhost",
      port: 3306,
      username: "root",
      password: "wrong_password",
      database: "test_db",
      ssh_enabled: false,
      ssl_enabled: false,
    };

    mockInvoke
      .mockResolvedValueOnce([]) // Initial list
      .mockRejectedValueOnce(new Error("Connection refused"));

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await expect(
      act(async () => {
        await result.current.testConnection(connectionInput);
      })
    ).rejects.toThrow("Connection refused");
  });

  it("should return null error when no fetch error exists", async () => {
    mockInvoke.mockResolvedValue(mockConnections);

    const { result } = renderHook(() => useConnections(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBeNull();
  });
});
