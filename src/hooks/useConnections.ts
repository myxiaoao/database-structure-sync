import { useState, useCallback } from "react";
import {
  useConnectionsQuery,
  useSaveConnectionMutation,
  useDeleteConnectionMutation,
  useTestConnectionMutation,
} from "@/lib/query";
import type { Connection, ConnectionInput } from "@/types";

export function useConnections() {
  const [editingConnection, setEditingConnection] = useState<Connection | null>(null);
  const [isFormOpen, setIsFormOpen] = useState(false);

  const { data: connections = [], isLoading, error, refetch } = useConnectionsQuery();

  const saveMutation = useSaveConnectionMutation();
  const deleteMutation = useDeleteConnectionMutation();
  const testMutation = useTestConnectionMutation();

  const openNewConnection = useCallback(() => {
    setEditingConnection(null);
    setIsFormOpen(true);
  }, []);

  const openEditConnection = useCallback((connection: Connection) => {
    setEditingConnection(connection);
    setIsFormOpen(true);
  }, []);

  const closeForm = useCallback(() => {
    setIsFormOpen(false);
    setEditingConnection(null);
  }, []);

  const saveConnection = useCallback(
    async (input: ConnectionInput) => {
      await saveMutation.mutateAsync(input);
      closeForm();
    },
    [saveMutation, closeForm]
  );

  const deleteConnection = useCallback(
    async (id: string) => {
      await deleteMutation.mutateAsync(id);
    },
    [deleteMutation]
  );

  const testConnection = useCallback(
    async (input: ConnectionInput) => {
      await testMutation.mutateAsync(input);
    },
    [testMutation]
  );

  return {
    connections,
    isLoading,
    error: error?.message ?? null,
    refetch,
    editingConnection,
    isFormOpen,
    openNewConnection,
    openEditConnection,
    closeForm,
    saveConnection,
    deleteConnection,
    testConnection,
    isSaving: saveMutation.isPending,
    isDeleting: deleteMutation.isPending,
    isTesting: testMutation.isPending,
  };
}
