import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { connectionsApi, syncApi } from "@/lib/api";
import type { ConnectionInput } from "@/types";
import { connectionKeys } from "./queries";

export function useSaveConnectionMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: ConnectionInput) => connectionsApi.save(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: connectionKeys.list() });
      toast.success("Connection saved successfully");
    },
    onError: (error) => {
      toast.error(`Failed to save connection: ${error.message}`);
    },
  });
}

export function useDeleteConnectionMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => connectionsApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: connectionKeys.list() });
      toast.success("Connection deleted successfully");
    },
    onError: (error) => {
      toast.error(`Failed to delete connection: ${error.message}`);
    },
  });
}

export function useTestConnectionMutation() {
  return useMutation({
    mutationFn: (input: ConnectionInput) => connectionsApi.test(input),
    onSuccess: () => {
      toast.success("Connection test successful");
    },
    onError: (error) => {
      toast.error(`Connection test failed: ${error.message}`);
    },
  });
}

export function useCompareMutation() {
  return useMutation({
    mutationFn: syncApi.compare,
    onError: (error) => {
      toast.error(`Comparison failed: ${error.message}`);
    },
  });
}

export function useExecuteSyncMutation() {
  return useMutation({
    mutationFn: syncApi.execute,
    onSuccess: () => {
      toast.success("Sync executed successfully");
    },
    onError: (error) => {
      toast.error(`Sync failed: ${error.message}`);
    },
  });
}
