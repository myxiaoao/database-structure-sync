import { useState, useMemo, useCallback } from "react";
import { useDatabasesQuery, useCompareMutation, useExecuteSyncMutation } from "@/lib/query";
import type { DiffItem, DiffResult, Connection } from "@/types";

interface UseSyncOptions {
  connections: Connection[];
}

export function useSync({ connections }: UseSyncOptions) {
  const [sourceId, setSourceId] = useState<string>("");
  const [targetId, setTargetId] = useState<string>("");
  const [sourceDb, setSourceDb] = useState<string>("");
  const [targetDb, setTargetDb] = useState<string>("");
  const [diffResult, setDiffResult] = useState<DiffResult | null>(null);
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [selectedItem, setSelectedItem] = useState<DiffItem | null>(null);

  const sourceConnection = connections.find((c) => c.id === sourceId);
  const targetConnection = connections.find((c) => c.id === targetId);
  const sourceNeedsDbSelect = sourceConnection && !sourceConnection.database;
  const targetNeedsDbSelect = targetConnection && !targetConnection.database;

  const { data: sourceDatabases = [], isLoading: loadingSourceDbs } = useDatabasesQuery(
    sourceId,
    !!sourceNeedsDbSelect
  );

  const { data: targetDatabases = [], isLoading: loadingTargetDbs } = useDatabasesQuery(
    targetId,
    !!targetNeedsDbSelect
  );

  const compareMutation = useCompareMutation();
  const executeMutation = useExecuteSyncMutation();

  const canCompare =
    sourceId &&
    targetId &&
    (!sourceNeedsDbSelect || sourceDb) &&
    (!targetNeedsDbSelect || targetDb);

  const handleCompare = useCallback(async () => {
    if (!canCompare) return;

    setDiffResult(null);
    setSelectedItems(new Set());
    setSelectedItem(null);

    const result = await compareMutation.mutateAsync({
      sourceId,
      targetId,
      sourceDatabase: sourceNeedsDbSelect ? sourceDb : undefined,
      targetDatabase: targetNeedsDbSelect ? targetDb : undefined,
    });

    setDiffResult(result);
  }, [canCompare, sourceId, targetId, sourceNeedsDbSelect, targetNeedsDbSelect, sourceDb, targetDb, compareMutation]);

  const selectedSql = useMemo(() => {
    if (!diffResult) return "";
    return diffResult.items
      .filter((item) => selectedItems.has(item.id))
      .map((item) => item.sql)
      .join("\n\n");
  }, [diffResult, selectedItems]);

  const handleExecute = useCallback(async () => {
    if (!targetId || !selectedSql) return;

    const statements = selectedSql
      .split("\n\n")
      .filter((s) => s.trim())
      .map((s) => s.trim());

    await executeMutation.mutateAsync({
      targetId,
      sqlStatements: statements,
      targetDatabase: targetNeedsDbSelect ? targetDb : undefined,
    });

    // Refresh comparison after execution
    await handleCompare();
  }, [targetId, selectedSql, targetNeedsDbSelect, targetDb, executeMutation, handleCompare]);

  const handleSelectAll = useCallback(() => {
    if (!diffResult) return;
    setSelectedItems(new Set(diffResult.items.map((item) => item.id)));
  }, [diffResult]);

  const handleDeselectAll = useCallback(() => {
    setSelectedItems(new Set());
  }, []);

  // Reset database selection when connection changes
  const handleSourceChange = useCallback((id: string) => {
    setSourceId(id);
    setSourceDb("");
  }, []);

  const handleTargetChange = useCallback((id: string) => {
    setTargetId(id);
    setTargetDb("");
  }, []);

  return {
    // Selection state
    sourceId,
    targetId,
    sourceDb,
    targetDb,
    setSourceId: handleSourceChange,
    setTargetId: handleTargetChange,
    setSourceDb,
    setTargetDb,

    // Connection info
    sourceConnection,
    targetConnection,
    sourceNeedsDbSelect,
    targetNeedsDbSelect,

    // Databases
    sourceDatabases,
    targetDatabases,
    loadingSourceDbs,
    loadingTargetDbs,

    // Diff result
    diffResult,
    selectedItems,
    setSelectedItems,
    selectedItem,
    setSelectedItem,
    selectedSql,

    // Actions
    canCompare,
    handleCompare,
    handleExecute,
    handleSelectAll,
    handleDeselectAll,

    // Loading states
    isComparing: compareMutation.isPending,
    isExecuting: executeMutation.isPending,
  };
}
