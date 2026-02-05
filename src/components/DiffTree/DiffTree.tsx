import React, { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown, ChevronRight, Table, Edit, Key, Link, Fingerprint } from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox";
import { ScrollArea } from "@/components/ui/scroll-area";
import { DiffItem } from "@/lib/api";

interface DiffTreeProps {
  items: DiffItem[];
  selectedItems: Set<string>;
  onSelectionChange: (selected: Set<string>) => void;
  onItemClick?: (item: DiffItem) => void;
  expandedTables: Set<string>;
  onExpandedChange: (expanded: Set<string>) => void;
}

interface GroupedDiff {
  tableName: string;
  items: DiffItem[];
}

function getDiffIcon(diffType: string): React.ReactNode {
  // 根据操作类型获取颜色
  const colorClass = diffType.includes("Added")
    ? "text-green-500"
    : diffType.includes("Removed")
      ? "text-red-500"
      : diffType.includes("Modified")
        ? "text-amber-500"
        : "text-muted-foreground";

  // 根据对象类型获取图标
  const prefix = diffType.replace(/(Added|Removed|Modified)$/, "");

  switch (prefix) {
    case "Table":
      return <Table className={`h-3 w-3 ${colorClass}`} />;
    case "Column":
      return <Edit className={`h-3 w-3 ${colorClass}`} />;
    case "Index":
      return <Key className={`h-3 w-3 ${colorClass}`} />;
    case "ForeignKey":
      return <Link className={`h-3 w-3 ${colorClass}`} />;
    case "UniqueConstraint":
      return <Fingerprint className={`h-3 w-3 ${colorClass}`} />;
    default:
      return <Table className={`h-3 w-3 ${colorClass}`} />;
  }
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
      className="flex items-center gap-1.5 py-1 px-2 rounded cursor-pointer hover:bg-muted/80 ml-5 text-xs"
      onClick={onClick}
    >
      <Checkbox
        checked={isSelected}
        onCheckedChange={() => {}}
        onClick={(e) => {
          e.stopPropagation();
          onToggle();
        }}
        className="h-3.5 w-3.5"
      />
      {getDiffIcon(item.diff_type)}
      <span className="flex-1 truncate">{item.object_name || item.table_name}</span>
      <span className="text-[10px] text-muted-foreground">{getDiffLabel(item.diff_type, t)}</span>
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
    <div className="mb-0.5">
      <div
        className="flex items-center gap-1.5 py-1 px-2 rounded cursor-pointer hover:bg-muted/80"
        onClick={onToggleExpand}
      >
        <span className="p-0">
          {isExpanded ? (
            <ChevronDown className="h-3.5 w-3.5" />
          ) : (
            <ChevronRight className="h-3.5 w-3.5" />
          )}
        </span>
        <Checkbox
          checked={allSelected}
          ref={(el) => {
            if (el) {
              (el as HTMLButtonElement).dataset.indeterminate = someSelected ? "true" : "false";
            }
          }}
          onCheckedChange={() => {}}
          onClick={handleGroupToggle}
          className="h-3.5 w-3.5"
        />
        <Table className="h-3.5 w-3.5 text-muted-foreground" />
        <span className="flex-1 text-xs font-medium truncate">{group.tableName}</span>
        <span className="text-[10px] text-muted-foreground px-1.5 py-0.5 bg-muted rounded">
          {group.items.length}
        </span>
      </div>
      {isExpanded && (
        <div className="border-l border-muted ml-2">
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

export function DiffTree({
  items,
  selectedItems,
  onSelectionChange,
  onItemClick,
  expandedTables,
  onExpandedChange,
}: DiffTreeProps) {
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
    const newSet = new Set(expandedTables);
    if (newSet.has(tableName)) {
      newSet.delete(tableName);
    } else {
      newSet.add(tableName);
    }
    onExpandedChange(newSet);
  };

  return (
    <ScrollArea className="h-full">
      <div className="p-1.5">
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
