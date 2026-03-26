import { useTranslation } from "react-i18next";
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
    <div className="flex-1 min-w-0 border rounded-md px-2.5 py-2">
      <div className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wide mb-1.5">
        {label}
      </div>
      <div className="flex items-center gap-2">
        <Select value={connectionId} onValueChange={onConnectionChange}>
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
        {needsDbSelect && (
          <Select value={selectedDb} onValueChange={onDbChange} disabled={loadingDbs}>
            <SelectTrigger className="h-8 text-sm min-w-0 flex-1">
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
        <div className="text-[10px] text-muted-foreground mt-1">
          {DB_TYPE_LABELS[selectedConn.db_type as DbType] || selectedConn.db_type} ·{" "}
          {selectedConn.host}:{selectedConn.port}
        </div>
      )}
    </div>
  );
}
