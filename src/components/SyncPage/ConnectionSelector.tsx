import { useTranslation } from "react-i18next";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { DB_TYPE_LABELS } from "@/types";
import type { Connection, DbType } from "@/types";

interface ConnectionSelectorProps {
  label: string;
  connections: Connection[];
  connectionId: string;
  onConnectionChange: (id: string) => void;
  needsDbSelect: boolean | "" | undefined;
  databases: string[];
  loadingDbs: boolean;
  selectedDb: string;
  onDbChange: (db: string) => void;
}

export function ConnectionSelector({
  label,
  connections,
  connectionId,
  onConnectionChange,
  needsDbSelect,
  databases,
  loadingDbs,
  selectedDb,
  onDbChange,
}: ConnectionSelectorProps) {
  const { t } = useTranslation();
  const selectedConn = connections.find((c) => c.id === connectionId);

  return (
    <div className="border rounded-lg p-3 flex flex-col gap-2">
      <Label className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
        {label}
      </Label>
      <div className="space-y-2">
        <Select value={connectionId} onValueChange={onConnectionChange}>
          <SelectTrigger className="h-9 text-sm">
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
        {needsDbSelect && (
          <Select value={selectedDb} onValueChange={onDbChange} disabled={loadingDbs}>
            <SelectTrigger className="h-9 text-sm">
              <SelectValue
                placeholder={loadingDbs ? t("common.loading") : t("sync.selectDatabase")}
              />
            </SelectTrigger>
            <SelectContent>
              {databases.map((db) => (
                <SelectItem key={db} value={db}>
                  {db}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </div>
      {selectedConn && (
        <div className="text-[11px] text-muted-foreground">
          {DB_TYPE_LABELS[selectedConn.db_type as DbType] || selectedConn.db_type} ·{" "}
          {selectedConn.host}:{selectedConn.port}
        </div>
      )}
    </div>
  );
}
