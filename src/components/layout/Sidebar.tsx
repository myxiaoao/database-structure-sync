import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Plus, Database, ChevronDown, ChevronRight, Trash2, Edit } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Connection } from '@/lib/api';

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
  const { t, i18n } = useTranslation();
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});

  const toggleExpand = (id: string) => {
    setExpanded((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const toggleLanguage = () => {
    const newLang = i18n.language === 'en' ? 'zh' : 'en';
    i18n.changeLanguage(newLang);
    localStorage.setItem('language', newLang);
  };

  return (
    <div className="w-64 border-r bg-muted/30 flex flex-col">
      <div className="p-4 border-b flex items-center justify-between">
        <h2 className="font-semibold text-sm">{t('connection.title')}</h2>
        <Button variant="ghost" size="sm" onClick={toggleLanguage}>
          {i18n.language === 'en' ? '中文' : 'EN'}
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-2">
          {connections.map((conn) => (
            <div key={conn.id} className="mb-1">
              <div
                className={`flex items-center gap-2 p-2 rounded-md cursor-pointer hover:bg-muted ${
                  selectedId === conn.id ? 'bg-muted' : ''
                }`}
                onClick={() => onSelectConnection?.(conn.id)}
              >
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleExpand(conn.id);
                  }}
                  className="p-0.5"
                >
                  {expanded[conn.id] ? (
                    <ChevronDown className="h-4 w-4" />
                  ) : (
                    <ChevronRight className="h-4 w-4" />
                  )}
                </button>
                <Database className="h-4 w-4 text-muted-foreground" />
                <span className="flex-1 text-sm truncate">{conn.name}</span>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-6 w-6">
                      <span className="sr-only">Actions</span>
                      ...
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={() => onEditConnection?.(conn.id)}>
                      <Edit className="h-4 w-4 mr-2" />
                      {t('connection.edit')}
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      onClick={() => onDeleteConnection?.(conn.id)}
                      className="text-destructive"
                    >
                      <Trash2 className="h-4 w-4 mr-2" />
                      {t('connection.delete')}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
              {expanded[conn.id] && (
                <div className="ml-8 text-xs text-muted-foreground p-2">
                  <div>{conn.db_type}</div>
                  <div>{conn.host}</div>
                  <div>{conn.database}</div>
                </div>
              )}
            </div>
          ))}
        </div>
      </ScrollArea>

      <div className="p-4 border-t">
        <Button onClick={onNewConnection} className="w-full" size="sm">
          <Plus className="h-4 w-4 mr-2" />
          {t('connection.new')}
        </Button>
      </div>
    </div>
  );
}
