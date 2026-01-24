import { useState, useMemo } from "react";
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
import { api, Connection, DiffItem, DiffResult } from "@/lib/api";

interface SyncPageProps {
  connections: Connection[];
}

export function SyncPage({ connections }: SyncPageProps) {
  const { t } = useTranslation();
  const [sourceId, setSourceId] = useState<string>("");
  const [targetId, setTargetId] = useState<string>("");
  const [diffResult, setDiffResult] = useState<DiffResult | null>(null);
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [comparing, setComparing] = useState(false);
  const [executing, setExecuting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedItem, setSelectedItem] = useState<DiffItem | null>(null);

  const handleCompare = async () => {
    if (!sourceId || !targetId) return;

    setComparing(true);
    setError(null);
    setDiffResult(null);
    setSelectedItems(new Set());
    setSelectedItem(null);

    try {
      const result = await api.compareDatabases(sourceId, targetId);
      setDiffResult(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setComparing(false);
    }
  };

  const handleSelectAll = () => {
    if (!diffResult) return;
    const allIds = new Set<string>(diffResult.items.map((item) => item.id));
    setSelectedItems(allIds);
  };

  const handleDeselectAll = () => {
    setSelectedItems(new Set());
  };

  const selectedSql = useMemo(() => {
    if (!diffResult) return "";
    return diffResult.items
      .filter((item) => selectedItems.has(item.id))
      .map((item) => item.sql)
      .join("\n\n");
  }, [diffResult, selectedItems]);

  const handleExecute = async () => {
    if (!targetId || !selectedSql) return;

    setExecuting(true);
    setError(null);

    try {
      const statements = selectedSql
        .split("\n\n")
        .filter((s) => s.trim())
        .map((s) => s.trim());
      await api.executeSync(targetId, statements);
      // Refresh comparison after execution
      await handleCompare();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setExecuting(false);
    }
  };

  const previewSql = selectedItem?.sql || selectedSql;

  return (
    <div className="h-full flex flex-col gap-4">
      {/* Database Selectors */}
      <Card>
        <CardContent className="pt-4">
          <div className="flex items-end gap-4">
            <div className="flex-1 space-y-2">
              <label className="text-sm font-medium">{t("sync.source")}</label>
              <Select value={sourceId} onValueChange={setSourceId}>
                <SelectTrigger>
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
            </div>
            <div className="flex-1 space-y-2">
              <label className="text-sm font-medium">{t("sync.target")}</label>
              <Select value={targetId} onValueChange={setTargetId}>
                <SelectTrigger>
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
            </div>
            <Button onClick={handleCompare} disabled={!sourceId || !targetId || comparing}>
              {comparing ? t("common.loading") : t("sync.compare")}
            </Button>
          </div>
        </CardContent>
      </Card>

      {error && (
        <div className="p-3 rounded-md bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200 text-sm">
          {error}
        </div>
      )}

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
            <Button onClick={handleExecute} disabled={!selectedSql || executing} className="w-full">
              {executing ? t("common.loading") : t("sync.execute")}
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
