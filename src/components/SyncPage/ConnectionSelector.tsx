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
  children?: React.ReactNode;
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
  children,
}: ConnectionSelectorProps) {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2 p-2.5 bg-muted/30 rounded-lg border min-w-0">
      <label className="text-xs font-medium text-muted-foreground whitespace-nowrap shrink-0">
        {label}
      </label>
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
      {children}
    </div>
  );
}
