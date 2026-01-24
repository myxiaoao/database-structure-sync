import React, { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  ChevronDown,
  ChevronRight,
  Table,
  Plus,
  Minus,
  Edit,
  Key,
  Link,
  Fingerprint,
} from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox";
import { ScrollArea } from "@/components/ui/scroll-area";
import { DiffItem } from "@/lib/api";

interface DiffTreeProps {
  items: DiffItem[];
  selectedItems: Set<string>;
  onSelectionChange: (selected: Set<string>) => void;
  onItemClick?: (item: DiffItem) => void;
}

interface GroupedDiff {
  tableName: string;
  items: DiffItem[];
}

function getDiffIcon(diffType: string): React.ReactNode {
  if (diffType.includes("Added")) return <Plus className="h-4 w-4 text-green-500" />;
  if (diffType.includes("Removed")) return <Minus className="h-4 w-4 text-red-500" />;
  if (diffType.includes("Modified")) return <Edit className="h-4 w-4 text-yellow-500" />;
  return <Table className="h-4 w-4" />;
}

const typeIconMap: Record<string, React.ReactNode> = {
  Table: <Table className="h-4 w-4 text-muted-foreground" />,
  Column: <Edit className="h-4 w-4 text-muted-foreground" />,
  Index: <Key className="h-4 w-4 text-muted-foreground" />,
  ForeignKey: <Link className="h-4 w-4 text-muted-foreground" />,
  UniqueConstraint: <Fingerprint className="h-4 w-4 text-muted-foreground" />,
};

function getTypeIcon(diffType: string): React.ReactNode {
  const prefix = diffType.replace(/(Added|Removed|Modified)$/, "");
  return typeIconMap[prefix] || typeIconMap.Table;
}

function getDiffLabel(diffType: string, t: (key: string) => string): string {
  const labels: Record<string, string> = {
    TableAdded: t("diff.tableAdded"),
    TableRemoved: t("diff.tableRemoved"),
    ColumnAdded: t("diff.columnAdded"),
    ColumnRemoved: t("diff.columnRemoved"),
    ColumnModified: t("diff.columnModified"),
    IndexAdded: t("diff.indexAdded"),
    IndexRemoved: t("diff.indexRemoved"),
    IndexModified: t("diff.indexModified"),
    ForeignKeyAdded: t("diff.foreignKeyAdded"),
    ForeignKeyRemoved: t("diff.foreignKeyRemoved"),
    UniqueConstraintAdded: t("diff.uniqueAdded"),
    UniqueConstraintRemoved: t("diff.uniqueRemoved"),
  };
  return labels[diffType] || diffType;
}

function DiffItemRow({
  item,
  isSelected,
  onToggle,
  onClick,
}: {
  item: DiffItem;
  isSelected: boolean;
  onToggle: () => void;
  onClick: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div
      className="flex items-center gap-2 py-1.5 px-2 rounded-md cursor-pointer hover:bg-muted ml-6"
      onClick={onClick}
    >
      <Checkbox
        checked={isSelected}
        onCheckedChange={() => {}}
        onClick={(e) => {
          e.stopPropagation();
          onToggle();
        }}
      />
      {getDiffIcon(item.diff_type)}
      {getTypeIcon(item.diff_type)}
      <span className="flex-1 text-sm truncate">{item.object_name || item.table_name}</span>
      <span className="text-xs text-muted-foreground">{getDiffLabel(item.diff_type, t)}</span>
    </div>
  );
}

function TableGroup({
  group,
  selectedItems,
  onSelectionChange,
  onItemClick,
  isExpanded,
  onToggleExpand,
}: {
  group: GroupedDiff;
  selectedItems: Set<string>;
  onSelectionChange: (selected: Set<string>) => void;
  onItemClick?: (item: DiffItem) => void;
  isExpanded: boolean;
  onToggleExpand: () => void;
}) {
  const allSelected = group.items.every((item) => selectedItems.has(item.id));
  const someSelected = !allSelected && group.items.some((item) => selectedItems.has(item.id));

  const handleGroupToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    const newSelected = new Set(selectedItems);
    if (allSelected) {
      group.items.forEach((item) => newSelected.delete(item.id));
    } else {
      group.items.forEach((item) => newSelected.add(item.id));
    }
    onSelectionChange(newSelected);
  };

  const handleItemToggle = (item: DiffItem) => {
    const newSelected = new Set(selectedItems);
    if (newSelected.has(item.id)) {
      newSelected.delete(item.id);
    } else {
      newSelected.add(item.id);
    }
    onSelectionChange(newSelected);
  };

  return (
    <div className="mb-1">
      <div
        className="flex items-center gap-2 py-1.5 px-2 rounded-md cursor-pointer hover:bg-muted"
        onClick={onToggleExpand}
      >
        <button onClick={(e) => e.stopPropagation()} className="p-0.5">
          {isExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
        </button>
        <Checkbox
          checked={allSelected}
          ref={(el) => {
            if (el) {
              (el as HTMLButtonElement).dataset.indeterminate = someSelected ? "true" : "false";
            }
          }}
          onCheckedChange={() => {}}
          onClick={handleGroupToggle}
        />
        <Table className="h-4 w-4 text-muted-foreground" />
        <span className="flex-1 text-sm font-medium truncate">{group.tableName}</span>
        <span className="text-xs text-muted-foreground">
          {group.items.length} change{group.items.length > 1 ? "s" : ""}
        </span>
      </div>
      {isExpanded && (
        <div>
          {group.items.map((item) => (
            <DiffItemRow
              key={item.id}
              item={item}
              isSelected={selectedItems.has(item.id)}
              onToggle={() => handleItemToggle(item)}
              onClick={() => onItemClick?.(item)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export function DiffTree({ items, selectedItems, onSelectionChange, onItemClick }: DiffTreeProps) {
  const [expandedTables, setExpandedTables] = useState<Set<string>>(new Set());

  // Group items by table name
  const groupedItems = useMemo(() => {
    const groups: Map<string, DiffItem[]> = new Map();

    items.forEach((item) => {
      const existing = groups.get(item.table_name) || [];
      existing.push(item);
      groups.set(item.table_name, existing);
    });

    return Array.from(groups.entries()).map(([tableName, tableItems]) => ({
      tableName,
      items: tableItems,
    }));
  }, [items]);

  const handleToggleExpand = (tableName: string) => {
    setExpandedTables((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(tableName)) {
        newSet.delete(tableName);
      } else {
        newSet.add(tableName);
      }
      return newSet;
    });
  };

  return (
    <ScrollArea className="h-full">
      <div className="p-2">
        {groupedItems.map((group) => (
          <TableGroup
            key={group.tableName}
            group={group}
            selectedItems={selectedItems}
            onSelectionChange={onSelectionChange}
            onItemClick={onItemClick}
            isExpanded={expandedTables.has(group.tableName)}
            onToggleExpand={() => handleToggleExpand(group.tableName)}
          />
        ))}
      </div>
    </ScrollArea>
  );
}
