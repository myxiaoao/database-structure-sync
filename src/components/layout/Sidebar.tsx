import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Database, ChevronDown, ChevronRight, Trash2, Edit } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Connection } from "@/lib/api";

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
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});

  const toggleExpand = (id: string) => {
    setExpanded((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  return (
    <div className="w-56 border-r bg-muted/20 flex flex-col">
      <div className="px-3 py-2 border-b flex items-center h-[45px]">
        <h2 className="font-semibold text-sm">{t("connection.title")}</h2>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-1.5">
          {connections.map((conn) => (
            <div key={conn.id} className="mb-0.5">
              <div
                className={`flex items-center gap-1.5 py-1.5 px-2 rounded cursor-pointer hover:bg-muted/80 ${
                  selectedId === conn.id ? "bg-muted" : ""
                }`}
                onClick={() => onSelectConnection?.(conn.id)}
              >
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleExpand(conn.id);
                  }}
                  className="p-0"
                >
                  {expanded[conn.id] ? (
                    <ChevronDown className="h-3.5 w-3.5" />
                  ) : (
                    <ChevronRight className="h-3.5 w-3.5" />
                  )}
                </button>
                <Database className="h-3.5 w-3.5 text-muted-foreground" />
                <span className="flex-1 text-xs truncate">{conn.name}</span>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-5 w-5 flex items-center justify-center"
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
              {expanded[conn.id] && (
                <div className="ml-6 text-[10px] text-muted-foreground py-1 px-2 space-y-0.5 border-l border-muted">
                  <div>{conn.db_type}</div>
                  <div>
                    {conn.host}:{conn.port}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      </ScrollArea>

      <div className="p-2 border-t">
        <Button
          onClick={onNewConnection}
          className="w-full h-7 text-xs flex items-center justify-center"
          size="sm"
        >
          <Plus className="h-3.5 w-3.5 mr-1.5 shrink-0" />
          <span>{t("connection.new")}</span>
        </Button>
      </div>
    </div>
  );
}
