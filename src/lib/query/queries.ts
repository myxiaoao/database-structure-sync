import { useQuery } from "@tanstack/react-query";
import { connectionsApi } from "@/lib/api";

export const connectionKeys = {
  all: ["connections"] as const,
  list: () => [...connectionKeys.all, "list"] as const,
  detail: (id: string) => [...connectionKeys.all, "detail", id] as const,
  databases: (id: string) => [...connectionKeys.all, "databases", id] as const,
};

export function useConnectionsQuery() {
  return useQuery({
    queryKey: connectionKeys.list(),
    queryFn: () => connectionsApi.list(),
  });
}

export function useConnectionQuery(id: string | null) {
  return useQuery({
    queryKey: connectionKeys.detail(id ?? ""),
    queryFn: () => (id ? connectionsApi.get(id) : null),
    enabled: !!id,
  });
}

export function useDatabasesQuery(connectionId: string | null, enabled = true) {
  return useQuery({
    queryKey: connectionKeys.databases(connectionId ?? ""),
    queryFn: () => (connectionId ? connectionsApi.listDatabases(connectionId) : []),
    enabled: !!connectionId && enabled,
  });
}
