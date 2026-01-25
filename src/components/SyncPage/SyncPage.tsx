import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { DiffTree } from "@/components/DiffTree";
import { SqlPreview } from "@/components/SqlPreview";
import { useSync } from "@/hooks";
import type { Connection } from "@/types";

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
    selectedItem,
    setSelectedItem,
    selectedSql,
    canCompare,
    handleCompare,
    handleExecute,
    handleSelectAll,
    handleDeselectAll,
    isComparing,
    isExecuting,
  } = useSync({ connections });

  const previewSql = selectedItem?.sql || selectedSql;

  return (
    <div className="h-full flex flex-col gap-4">
      {/* Database Selectors */}
      <Card>
        <CardContent className="pt-4">
          <div className="flex items-end gap-4">
            <div className="flex-1 space-y-2">
              <label className="text-sm font-medium">{t("sync.source")}</label>
              <div className="flex gap-2">
                <Select value={sourceId} onValueChange={setSourceId}>
                  <SelectTrigger className={sourceNeedsDbSelect ? "flex-1" : ""}>
                    <SelectValue placeholder={t("sync.selectConnection")} />
                  </SelectTrigger>
                  <SelectContent>
                    {connections.map((conn) => (
                      <SelectItem key={conn.id} value={conn.id}>
                        {conn.name} ({conn.db_type})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {sourceNeedsDbSelect && (
                  <Select value={sourceDb} onValueChange={setSourceDb} disabled={loadingSourceDbs}>
                    <SelectTrigger className="flex-1">
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
            </div>
            <div className="flex-1 space-y-2">
              <label className="text-sm font-medium">{t("sync.target")}</label>
              <div className="flex gap-2">
                <Select value={targetId} onValueChange={setTargetId}>
                  <SelectTrigger className={targetNeedsDbSelect ? "flex-1" : ""}>
                    <SelectValue placeholder={t("sync.selectConnection")} />
                  </SelectTrigger>
                  <SelectContent>
                    {connections.map((conn) => (
                      <SelectItem key={conn.id} value={conn.id}>
                        {conn.name} ({conn.db_type})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {targetNeedsDbSelect && (
                  <Select value={targetDb} onValueChange={setTargetDb} disabled={loadingTargetDbs}>
                    <SelectTrigger className="flex-1">
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
              </div>
            </div>
            <Button onClick={handleCompare} disabled={!canCompare || isComparing}>
              {isComparing ? t("common.loading") : t("sync.compare")}
            </Button>
          </div>
        </CardContent>
      </Card>


      {/* Diff Results */}
      {diffResult && (
        <div className="flex-1 grid grid-cols-2 gap-4 min-h-0">
          {/* Left: Diff Tree */}
          <Card className="flex flex-col">
            <CardHeader className="py-3 px-4 flex-row items-center justify-between space-y-0">
              <CardTitle className="text-sm">
                {diffResult.items.length} {t("sync.changes")}
              </CardTitle>
              <div className="flex gap-2">
                <Button variant="outline" size="sm" onClick={handleSelectAll}>
                  {t("sync.selectAll")}
                </Button>
                <Button variant="outline" size="sm" onClick={handleDeselectAll}>
                  {t("sync.deselectAll")}
                </Button>
              </div>
            </CardHeader>
            <Separator />
            <CardContent className="flex-1 p-0 overflow-hidden">
              {diffResult.items.length > 0 ? (
                <DiffTree
                  items={diffResult.items}
                  selectedItems={selectedItems}
                  onSelectionChange={setSelectedItems}
                  onItemClick={setSelectedItem}
                />
              ) : (
                <p className="p-4 text-sm text-muted-foreground">{t("sync.noChanges")}</p>
              )}
            </CardContent>
          </Card>

          {/* Right: SQL Preview */}
          <div className="flex flex-col gap-4">
            <div className="flex-1 min-h-0">
              <SqlPreview sql={previewSql} />
            </div>
            <Button onClick={handleExecute} disabled={!selectedSql || isExecuting} className="w-full">
              {isExecuting ? t("common.loading") : t("sync.execute")}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
