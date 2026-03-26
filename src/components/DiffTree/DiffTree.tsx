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
  const colorClass = diffType.includes("Added")
    ? "text-emerald-500"
    : diffType.includes("Removed")
      ? "text-red-500"
      : diffType.includes("Modified")
        ? "text-amber-500"
        : "text-muted-foreground";

  const prefix = diffType.replace(/(Added|Removed|Modified)$/, "");

  switch (prefix) {
    case "Table":
      return <Table className={`h-3.5 w-3.5 ${colorClass}`} />;
    case "Column":
      return <Edit className={`h-3.5 w-3.5 ${colorClass}`} />;
    case "Index":
      return <Key className={`h-3.5 w-3.5 ${colorClass}`} />;
    case "ForeignKey":
      return <Link className={`h-3.5 w-3.5 ${colorClass}`} />;
    case "UniqueConstraint":
      return <Fingerprint className={`h-3.5 w-3.5 ${colorClass}`} />;
    default:
      return <Table className={`h-3.5 w-3.5 ${colorClass}`} />;
  }
}

function getDiffBadge(
  diffType: string,
  t: (key: string) => string
): { label: string; className: string } {
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
    ForeignKeyModified: t("diff.foreignKeyModified"),
    UniqueConstraintAdded: t("diff.uniqueAdded"),
    UniqueConstraintRemoved: t("diff.uniqueRemoved"),
    UniqueConstraintModified: t("diff.uniqueModified"),
  };

  const colorClass = diffType.includes("Added")
    ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
    : diffType.includes("Removed")
      ? "bg-red-500/10 text-red-600 dark:text-red-400"
      : "bg-amber-500/10 text-amber-600 dark:text-amber-400";

  return { label: labels[diffType] || diffType, className: colorClass };
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
  const badge = getDiffBadge(item.diff_type, t);

  return (
    <div
      className="flex items-center gap-2 py-1.5 pl-2 pr-2.5 rounded cursor-pointer hover:bg-muted/80 text-xs"
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
      <span className="flex-1 truncate font-medium">{item.object_name || item.table_name}</span>
      <span className={`text-[10px] px-1.5 py-0.5 rounded font-medium shrink-0 ${badge.className}`}>
        {badge.label}
      </span>
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
        className="flex items-center gap-2 py-1.5 px-2 rounded-md cursor-pointer hover:bg-muted/80"
        onClick={onToggleExpand}
      >
        <span className="p-0">
          {isExpanded ? (
            <ChevronDown className="h-4 w-4 text-muted-foreground" />
          ) : (
            <ChevronRight className="h-4 w-4 text-muted-foreground" />
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
        <Table className="h-4 w-4 text-muted-foreground" />
        <span className="flex-1 text-sm font-semibold truncate">{group.tableName}</span>
        <span className="text-[10px] text-muted-foreground bg-muted px-1.5 py-0.5 rounded-full font-medium">
          {group.items.length}
        </span>
      </div>
      {isExpanded && (
        <div className="border-l-2 border-muted ml-5 pl-2 mt-0.5 mb-1">
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
