import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import i18next from "i18next";
import { connectionsApi, syncApi } from "@/lib/api";
import type { ConnectionInput } from "@/types";
import { connectionKeys } from "./queries";

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return String(error);
}

export function useSaveConnectionMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: ConnectionInput) => connectionsApi.save(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: connectionKeys.list() });
      toast.success(i18next.t("connection.saveSuccess"));
    },
    onError: (error) => {
      toast.error(i18next.t("connection.saveFailed", { error: getErrorMessage(error) }));
    },
  });
}

export function useUpdateConnectionMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: ConnectionInput }) =>
      connectionsApi.update(id, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: connectionKeys.list() });
      toast.success(i18next.t("connection.saveSuccess"));
    },
    onError: (error) => {
      toast.error(i18next.t("connection.saveFailed", { error: getErrorMessage(error) }));
    },
  });
}

export function useDeleteConnectionMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => connectionsApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: connectionKeys.list() });
      toast.success(i18next.t("connection.deleteSuccess"));
    },
    onError: (error) => {
      toast.error(i18next.t("connection.deleteFailed", { error: getErrorMessage(error) }));
    },
  });
}

export function useTestConnectionMutation() {
  return useMutation({
    mutationFn: (input: ConnectionInput) => connectionsApi.test(input),
    onSuccess: () => {
      toast.success(i18next.t("connection.testSuccess"));
    },
    onError: (error) => {
      toast.error(i18next.t("connection.testFailed", { error: getErrorMessage(error) }));
    },
  });
}

export function useCompareMutation() {
  return useMutation({
    mutationFn: syncApi.compare,
    onError: (error) => {
      toast.error(i18next.t("sync.compareFailed", { error: getErrorMessage(error) }));
    },
  });
}

export function useExecuteSyncMutation() {
  return useMutation({
    mutationFn: syncApi.execute,
    onSuccess: () => {
      toast.success(i18next.t("sync.executeSuccess"));
    },
    onError: (error) => {
      toast.error(i18next.t("sync.executeFailed", { error: getErrorMessage(error) }));
    },
  });
}
