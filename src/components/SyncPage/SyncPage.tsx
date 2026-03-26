import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Download, Play, DatabaseZap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { DiffTree } from "@/components/DiffTree";
import { ConnectionSelector } from "./ConnectionSelector";
import { useSync } from "@/hooks";
import { DB_TYPE_LABELS } from "@/types";
import type { Connection, DbType } from "@/types";

const MYSQL_FAMILY = new Set<string>(["mysql", "mariadb"]);

function isCrossDbType(source: Connection | undefined, target: Connection | undefined): boolean {
  if (!source || !target) return false;
  if (source.db_type === target.db_type) return false;
  return !(MYSQL_FAMILY.has(source.db_type) && MYSQL_FAMILY.has(target.db_type));
}

interface SyncPageProps {
  connections: Connection[];
}

export function SyncPage({ connections }: SyncPageProps) {
  const { t } = useTranslation();

  const {
    sourceId,
    targetId,
    sourceDb,
    targetDb,
    setSourceId,
    setTargetId,
    setSourceDb,
    setTargetDb,
    sourceNeedsDbSelect,
    targetNeedsDbSelect,
    sourceDatabases,
    targetDatabases,
    loadingSourceDbs,
    loadingTargetDbs,
    diffResult,
    selectedItems,
    setSelectedItems,
    setSelectedItem,
    selectedSql,
    canCompare,
    handleCompare,
    handleExecute,
    handleSelectAll,
    handleDeselectAll,
    handleExportSql,
    isComparing,
    isExecuting,
    isExporting,
  } = useSync({ connections });

  const sourceConnection = connections.find((c) => c.id === sourceId);
  const targetConnection = connections.find((c) => c.id === targetId);

  const onCompare = useCallback(() => {
    if (isCrossDbType(sourceConnection, targetConnection)) {
      toast.warning(
        t("sync.crossDbWarning", {
          source:
            DB_TYPE_LABELS[(sourceConnection?.db_type as DbType) || ""] ||
            sourceConnection?.db_type,
          target:
            DB_TYPE_LABELS[(targetConnection?.db_type as DbType) || ""] ||
            targetConnection?.db_type,
        })
      );
      return;
    }
    handleCompare();
  }, [sourceConnection, targetConnection, handleCompare, t]);

  const onExportSql = useCallback(async () => {
    try {
      const success = await handleExportSql();
      if (success) {
        toast.success(t("sql.exportSuccess"));
      }
    } catch {
      toast.error(t("sql.exportFailed"));
    }
  }, [handleExportSql, t]);

  const [expandedTables, setExpandedTables] = useState<Set<string>>(new Set());

  const allTableNames = useMemo(() => {
    if (!diffResult) return [];
    const names = new Set<string>();
    diffResult.items.forEach((item) => names.add(item.table_name));
    return Array.from(names);
  }, [diffResult]);

  const handleExpandAll = useCallback(() => {
    setExpandedTables(new Set(allTableNames));
  }, [allTableNames]);

  const handleCollapseAll = useCallback(() => {
    setExpandedTables(new Set());
  }, []);

  const selectedCount = selectedItems.size;

  return (
    <div className="h-full flex flex-col">
      {/* Endpoint Selection */}
      <div className="shrink-0 px-4 pt-3 pb-2">
        <div className="border rounded-lg px-3 py-2.5">
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-baseline gap-2">
              <h2 className="text-xs font-semibold">{t("app.subtitle")}</h2>
              <span className="text-[10px] text-muted-foreground">{t("sync.endpointHint")}</span>
            </div>
          </div>
          <div className="flex items-start gap-3">
            <ConnectionSelector
              label={t("sync.source")}
              connections={connections}
              connectionId={sourceId}
              onConnectionChange={setSourceId}
              needsDbSelect={sourceNeedsDbSelect}
              databases={sourceDatabases}
              loadingDbs={loadingSourceDbs}
              selectedDb={sourceDb}
              onDbChange={setSourceDb}
            />
            <ConnectionSelector
              label={t("sync.target")}
              connections={connections}
              connectionId={targetId}
              onConnectionChange={setTargetId}
              needsDbSelect={targetNeedsDbSelect}
              databases={targetDatabases}
              loadingDbs={loadingTargetDbs}
              selectedDb={targetDb}
              onDbChange={setTargetDb}
            />
            <Button
              onClick={onCompare}
              disabled={!canCompare || isComparing}
              size="sm"
              className="h-8 px-5 shrink-0 mt-4"
            >
              {isComparing ? t("common.loading") : t("sync.compare")}
            </Button>
          </div>
        </div>
      </div>

      <Separator />

      {/* Step 2: Results */}
      {diffResult ? (
        <>
          <div className="flex-1 grid grid-cols-[2fr_3fr] gap-0 min-h-0 overflow-hidden">
            {/* Left: Diff Tree (40%) */}
            <div className="flex flex-col border-r overflow-hidden">
              <div className="flex items-center justify-between px-3 h-10 border-b bg-muted/30 shrink-0">
                <span className="text-xs font-semibold">
                  {diffResult.items.length} {t("sync.changes")}
                </span>
                <div className="flex gap-0.5">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleExpandAll}
                    className="h-6 px-2 text-xs"
                  >
                    {t("sync.expandAll")}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleCollapseAll}
                    className="h-6 px-2 text-xs"
                  >
                    {t("sync.collapseAll")}
                  </Button>
                  <Separator orientation="vertical" className="h-4 mx-0.5" />
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleSelectAll}
                    className="h-6 px-2 text-xs"
                  >
                    {t("sync.selectAll")}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleDeselectAll}
                    className="h-6 px-2 text-xs"
                  >
                    {t("sync.deselectAll")}
                  </Button>
                </div>
              </div>
              <div className="flex-1 overflow-hidden">
                {diffResult.items.length > 0 ? (
                  <DiffTree
                    items={diffResult.items}
                    selectedItems={selectedItems}
                    onSelectionChange={setSelectedItems}
                    onItemClick={setSelectedItem}
                    expandedTables={expandedTables}
                    onExpandedChange={setExpandedTables}
                  />
                ) : (
                  <div className="flex items-center justify-center h-full text-sm text-muted-foreground">
                    {t("sync.noChanges")}
                  </div>
                )}
              </div>
            </div>

            {/* Right: SQL Preview (60%) */}
            <div className="flex flex-col overflow-hidden">
              <div className="flex items-center justify-between px-3 h-10 border-b bg-muted/30 shrink-0">
                <span className="text-xs font-semibold">{t("sql.preview")}</span>
              </div>
              <div className="flex-1 overflow-auto min-h-0">
                {selectedSql ? (
                  <pre className="p-3 text-xs font-mono whitespace-pre-wrap break-all leading-relaxed">
                    {selectedSql}
                  </pre>
                ) : (
                  <div className="flex items-center justify-center h-full text-sm text-muted-foreground">
                    {t("sql.empty")}
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Bottom Action Bar */}
          <div className="shrink-0 border-t bg-muted/20 px-4 py-2.5 flex items-center justify-between">
            <span className="text-xs text-muted-foreground">
              {selectedCount > 0
                ? t("sync.selectedCount", { count: selectedCount })
                : t("sync.noSelected")}
            </span>
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={onExportSql}
                disabled={!selectedSql || isExporting}
                className="h-8"
              >
                <Download className="h-3.5 w-3.5 mr-1.5" />
                {t("sql.export")}
              </Button>
              <Button
                size="sm"
                onClick={handleExecute}
                disabled={!selectedSql || isExecuting}
                className="h-8"
              >
                <Play className="h-3.5 w-3.5 mr-1.5" />
                {isExecuting ? t("common.loading") : t("sync.execute")}
              </Button>
            </div>
          </div>
        </>
      ) : (
        /* Empty State */
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center max-w-sm">
            <DatabaseZap className="h-12 w-12 text-muted-foreground/40 mx-auto mb-4" />
            <h3 className="text-sm font-medium mb-1.5">{t("sync.emptyTitle")}</h3>
            <p className="text-xs text-muted-foreground leading-relaxed">{t("sync.emptyDesc")}</p>
          </div>
        </div>
      )}
    </div>
  );
}
