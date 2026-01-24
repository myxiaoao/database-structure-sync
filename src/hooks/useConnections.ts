import { useState, useEffect, useCallback } from 'react';
import { api, Connection, ConnectionInput } from '@/lib/api';

export function useConnections() {
  const [connections, setConnections] = useState<Connection[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchConnections = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.listConnections();
      setConnections(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchConnections();
  }, [fetchConnections]);

  const saveConnection = useCallback(async (input: ConnectionInput) => {
    const saved = await api.saveConnection(input);
    await fetchConnections();
    return saved;
  }, [fetchConnections]);

  const deleteConnection = useCallback(async (id: string) => {
    await api.deleteConnection(id);
    await fetchConnections();
  }, [fetchConnections]);

  const testConnection = useCallback(async (input: ConnectionInput) => {
    await api.testConnection(input);
  }, []);

  return {
    connections,
    loading,
    error,
    refresh: fetchConnections,
    saveConnection,
    deleteConnection,
    testConnection,
  };
}
