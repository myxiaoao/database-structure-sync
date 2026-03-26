import { useTranslation } from "react-i18next";
import { Plus, Database, Trash2, Edit } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Connection } from "@/lib/api";
import { DB_TYPE_LABELS } from "@/types";
import type { DbType } from "@/types";

interface SidebarProps {
  connections?: Connection[];
  onNewConnection?: () => void;
  onEditConnection?: (id: string) => void;
  onDeleteConnection?: (id: string) => void;
  onSelectConnection?: (id: string) => void;
  selectedId?: string;
}

export function Sidebar({
  connections = [],
  onNewConnection,
  onEditConnection,
  onDeleteConnection,
  onSelectConnection,
  selectedId,
}: SidebarProps) {
  const { t } = useTranslation();

  return (
    <div className="w-60 border-r bg-muted/20 flex flex-col">
      <div className="px-3 py-2.5 border-b flex items-center justify-between h-[52px]">
        <h2 className="font-semibold text-sm">{t("connection.title")}</h2>
        <Button
          onClick={onNewConnection}
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          aria-label={t("connection.new")}
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-2 space-y-0.5">
          {connections.length === 0 && (
            <div className="text-xs text-muted-foreground text-center py-8 px-4">
              {t("connection.new")}
            </div>
          )}
          {connections.map((conn) => (
            <div
              key={conn.id}
              className={`group flex items-start gap-2.5 py-2 px-2.5 rounded-md cursor-pointer transition-colors hover:bg-muted/80 ${
                selectedId === conn.id ? "bg-muted" : ""
              }`}
              onClick={() => onSelectConnection?.(conn.id)}
            >
              <Database className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
              <div className="flex-1 min-w-0">
                <div className="text-sm font-medium truncate leading-tight">{conn.name}</div>
                <div className="text-[11px] text-muted-foreground mt-0.5 truncate">
                  {DB_TYPE_LABELS[conn.db_type as DbType] || conn.db_type} · {conn.host}:{conn.port}
                </div>
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <span className="sr-only">Actions</span>
                    <span className="text-xs leading-none">···</span>
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onClick={() => onEditConnection?.(conn.id)}>
                    <Edit className="h-3.5 w-3.5 mr-2" />
                    {t("connection.edit")}
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onClick={() => onDeleteConnection?.(conn.id)}
                    className="text-destructive"
                  >
                    <Trash2 className="h-3.5 w-3.5 mr-2" />
                    {t("connection.delete")}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          ))}
        </div>
      </ScrollArea>
    </div>
  );
}
