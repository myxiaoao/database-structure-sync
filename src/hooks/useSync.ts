import { useState, useMemo, useCallback } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { useDatabasesQuery, useCompareMutation, useExecuteSyncMutation } from "@/lib/query";
import { syncApi } from "@/lib/api/sync";
import type { DiffItem, DiffResult, Connection } from "@/types";

const APP_VERSION = "0.1.0";

function generateSqlHeader(
  sourceConn: Connection | undefined,
  targetConn: Connection | undefined,
  sourceDb: string,
  targetDb: string,
  itemCount: number
): string {
  const now = new Date();
  const timestamp = now
    .toISOString()
    .replace("T", " ")
    .replace(/\.\d+Z$/, "");
  const sourceName = sourceConn
    ? `${sourceConn.name} (${sourceConn.host}:${sourceConn.port}/${sourceDb || sourceConn.database})`
    : "N/A";
  const targetName = targetConn
    ? `${targetConn.name} (${targetConn.host}:${targetConn.port}/${targetDb || targetConn.database})`
    : "N/A";
  const dbType = targetConn?.db_type || sourceConn?.db_type || "Unknown";
  const isMySQL = dbType === "MySQL" || dbType === "MariaDB";

  const lines = [
    "-- ---------------------------------------------------------",
    `-- Database Structure Sync v${APP_VERSION}`,
    "--",
    `-- Generation Time: ${timestamp}`,
    `-- Database Type:   ${dbType}`,
    `-- Source:          ${sourceName}`,
    `-- Target:          ${targetName}`,
    `-- Changes:         ${itemCount} item(s)`,
    "-- ---------------------------------------------------------",
    "",
  ];

  if (isMySQL) {
    lines.push(
      "/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;",
      "/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;",
      "/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;",
      "/*!40101 SET NAMES utf8mb4 */;",
      "/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;",
      "/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;",
      ""
    );
  } else {
    lines.push(
      "SET statement_timeout = 0;",
      "SET lock_timeout = 0;",
      "SET client_encoding = 'UTF8';",
      ""
    );
  }

  return lines.join("\n");
}

function generateSqlFooter(dbType: string): string {
  const isMySQL = dbType === "MySQL" || dbType === "MariaDB";

  const lines = [""];

  if (isMySQL) {
    lines.push(
      "/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;",
      "/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;",
      "/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;",
      "/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;",
      "/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;"
    );
  }

  lines.push(
    "",
    "-- ---------------------------------------------------------",
    "-- End of synchronization script",
    "-- ---------------------------------------------------------",
    ""
  );

  return lines.join("\n");
}

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
  }, [
    canCompare,
    sourceId,
    targetId,
    sourceNeedsDbSelect,
    targetNeedsDbSelect,
    sourceDb,
    targetDb,
    compareMutation,
  ]);

  const selectedSql = useMemo(() => {
    if (!diffResult) return "";
    const selectedDiffs = diffResult.items.filter((item) => selectedItems.has(item.id));
    if (selectedDiffs.length === 0) return "";
    const dbType = targetConnection?.db_type || sourceConnection?.db_type || "Unknown";
    const header = generateSqlHeader(
      sourceConnection,
      targetConnection,
      sourceDb,
      targetDb,
      selectedDiffs.length
    );
    const body = selectedDiffs.map((item) => item.sql).join("\n\n");
    const footer = generateSqlFooter(dbType);
    return header + "\n" + body + footer;
  }, [diffResult, selectedItems, sourceConnection, targetConnection, sourceDb, targetDb]);

  const handleExecute = useCallback(async () => {
    if (!targetId || !diffResult) return;
    if (executeMutation.isPending || compareMutation.isPending) return;

    const statements = diffResult.items
      .filter((item) => selectedItems.has(item.id))
      .map((item) => item.sql)
      .filter((s) => s.trim());

    if (statements.length === 0) return;

    await executeMutation.mutateAsync({
      targetId,
      sqlStatements: statements,
      targetDatabase: targetNeedsDbSelect ? targetDb : undefined,
    });

    // Refresh comparison after execution
    await handleCompare();
  }, [
    targetId,
    diffResult,
    selectedItems,
    targetNeedsDbSelect,
    targetDb,
    executeMutation,
    handleCompare,
  ]);

  const handleSelectAll = useCallback(() => {
    if (!diffResult) return;
    setSelectedItems(new Set(diffResult.items.map((item) => item.id)));
  }, [diffResult]);

  const handleDeselectAll = useCallback(() => {
    setSelectedItems(new Set());
  }, []);

  const [isExporting, setIsExporting] = useState(false);

  const handleExportSql = useCallback(async (): Promise<boolean> => {
    if (!selectedSql) return false;

    setIsExporting(true);
    try {
      const filePath = await save({
        defaultPath: "sync.sql",
        filters: [{ name: "SQL", extensions: ["sql"] }],
      });

      if (!filePath) {
        return false;
      }

      await syncApi.saveSqlFile(filePath, selectedSql);
      return true;
    } finally {
      setIsExporting(false);
    }
  }, [selectedSql]);

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
    handleExportSql,

    // Loading states
    isComparing: compareMutation.isPending,
    isExecuting: executeMutation.isPending,
    isExporting,
  };
}
