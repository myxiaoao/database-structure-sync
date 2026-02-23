import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { DiffTree } from "@/components/DiffTree";
import { useSync } from "@/hooks";
import { DB_TYPE_LABELS } from "@/types";
import type { Connection, DbType } from "@/types";

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

  // 展开状态管理
  const [expandedTables, setExpandedTables] = useState<Set<string>>(new Set());

  // 获取所有表名用于全部展开
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

  // 只显示勾选项的 SQL，没有勾选则显示空
  const previewSql = selectedSql;

  return (
    <div className="h-full flex flex-col gap-3">
      {/* Database Selectors - Two column layout matching diff results */}
      <div className="grid grid-cols-2 gap-3 shrink-0">
        {/* Source Database */}
        <div className="flex items-center gap-2 p-2.5 bg-muted/30 rounded-lg border min-w-0">
          <label className="text-xs font-medium text-muted-foreground whitespace-nowrap shrink-0">
            {t("sync.source")}
          </label>
          <Select value={sourceId} onValueChange={setSourceId}>
            <SelectTrigger className="h-8 text-sm min-w-0 flex-1">
              <SelectValue placeholder={t("sync.selectConnection")} />
            </SelectTrigger>
            <SelectContent>
              {connections.map((conn) => (
                <SelectItem key={conn.id} value={conn.id}>
                  {conn.name} ({DB_TYPE_LABELS[conn.db_type as DbType] || conn.db_type})
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {sourceNeedsDbSelect && (
            <Select value={sourceDb} onValueChange={setSourceDb} disabled={loadingSourceDbs}>
              <SelectTrigger className="h-8 text-sm min-w-0 flex-1">
                <SelectValue
                  placeholder={loadingSourceDbs ? t("common.loading") : t("sync.selectDatabase")}
                />
              </SelectTrigger>
              <SelectContent>
                {sourceDatabases.map((db) => (
                  <SelectItem key={db} value={db}>
                    {db}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
        </div>

        {/* Target Database + Compare Button */}
        <div className="flex items-center gap-2 p-2.5 bg-muted/30 rounded-lg border min-w-0">
          <label className="text-xs font-medium text-muted-foreground whitespace-nowrap shrink-0">
            {t("sync.target")}
          </label>
          <Select value={targetId} onValueChange={setTargetId}>
            <SelectTrigger className="h-8 text-sm min-w-0 flex-1">
              <SelectValue placeholder={t("sync.selectConnection")} />
            </SelectTrigger>
            <SelectContent>
              {connections.map((conn) => (
                <SelectItem key={conn.id} value={conn.id}>
                  {conn.name} ({DB_TYPE_LABELS[conn.db_type as DbType] || conn.db_type})
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {targetNeedsDbSelect && (
            <Select value={targetDb} onValueChange={setTargetDb} disabled={loadingTargetDbs}>
              <SelectTrigger className="h-8 text-sm min-w-0 flex-1">
                <SelectValue
                  placeholder={loadingTargetDbs ? t("common.loading") : t("sync.selectDatabase")}
                />
              </SelectTrigger>
              <SelectContent>
                {targetDatabases.map((db) => (
                  <SelectItem key={db} value={db}>
                    {db}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
          <Button
            onClick={handleCompare}
            disabled={!canCompare || isComparing}
            size="sm"
            className="h-8 shrink-0"
          >
            {isComparing ? t("common.loading") : t("sync.compare")}
          </Button>
        </div>
      </div>

      {/* Diff Results */}
      {diffResult && (
        <div className="flex-1 grid grid-cols-2 gap-3 min-h-0">
          {/* Left: Diff Tree */}
          <Card className="flex flex-col overflow-hidden">
            <div className="flex items-center justify-between px-3 h-9 border-b bg-muted/30">
              <span className="text-xs font-medium">
                {diffResult.items.length} {t("sync.changes")}
              </span>
              <div className="flex gap-1">
                <Separator orientation="vertical" className="h-4" />
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
                <Separator orientation="vertical" className="h-4" />
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
                <p className="p-3 text-sm text-muted-foreground">{t("sync.noChanges")}</p>
              )}
            </div>
          </Card>

          {/* Right: SQL Preview */}
          <div className="flex flex-col gap-2 min-h-0">
            <Card className="flex-1 flex flex-col overflow-hidden min-h-0">
              <div className="flex items-center justify-between px-3 h-9 border-b bg-muted/30 shrink-0">
                <span className="text-xs font-medium">{t("sql.preview")}</span>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={onExportSql}
                  disabled={!previewSql || isExporting}
                  className="h-6 px-2 text-xs"
                >
                  <Download className="h-3 w-3 mr-1" />
                  {t("sql.export")}
                </Button>
              </div>
              <div className="flex-1 overflow-auto min-h-0">
                {previewSql ? (
                  <pre className="p-3 text-xs font-mono whitespace-pre-wrap break-all bg-muted/20">
                    {previewSql}
                  </pre>
                ) : (
                  <div className="flex items-center justify-center h-full text-sm text-muted-foreground">
                    {t("sql.empty")}
                  </div>
                )}
              </div>
            </Card>
            <Button
              onClick={handleExecute}
              disabled={!selectedSql || isExecuting}
              size="sm"
              className="h-8 shrink-0"
            >
              {isExecuting ? t("common.loading") : t("sync.execute")}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
