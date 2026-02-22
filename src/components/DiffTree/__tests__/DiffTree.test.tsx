import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DiffTree } from "../DiffTree";
import type { DiffItem } from "@/types";
import { useState } from "react";

const mockItems: DiffItem[] = [
  {
    id: "1",
    diff_type: "TableAdded",
    table_name: "users",
    sql: "CREATE TABLE users ...",
    selected: false,
  },
  {
    id: "2",
    diff_type: "ColumnAdded",
    table_name: "users",
    object_name: "email",
    sql: "ALTER TABLE users ADD COLUMN email ...",
    selected: false,
  },
  {
    id: "3",
    diff_type: "ColumnRemoved",
    table_name: "posts",
    object_name: "legacy_field",
    sql: "ALTER TABLE posts DROP COLUMN legacy_field",
    selected: false,
  },
  {
    id: "4",
    diff_type: "IndexAdded",
    table_name: "posts",
    object_name: "idx_title",
    sql: "CREATE INDEX idx_title ON posts(title)",
    selected: false,
  },
  {
    id: "5",
    diff_type: "ColumnModified",
    table_name: "comments",
    object_name: "content",
    sql: "ALTER TABLE comments MODIFY content TEXT",
    selected: false,
  },
];

// Wrapper component to manage expandedTables state
function DiffTreeWrapper({
  items = mockItems,
  selectedItems = new Set<string>(),
  onSelectionChange = vi.fn(),
  onItemClick,
  initialExpanded = new Set<string>(),
}: {
  items?: DiffItem[];
  selectedItems?: Set<string>;
  onSelectionChange?: (selected: Set<string>) => void;
  onItemClick?: (item: DiffItem) => void;
  initialExpanded?: Set<string>;
}) {
  const [expandedTables, setExpandedTables] = useState<Set<string>>(initialExpanded);

  return (
    <DiffTree
      items={items}
      selectedItems={selectedItems}
      onSelectionChange={onSelectionChange}
      onItemClick={onItemClick}
      expandedTables={expandedTables}
      onExpandedChange={setExpandedTables}
    />
  );
}

describe("DiffTree", () => {
  it("should render grouped items by table name", () => {
    render(<DiffTreeWrapper />);

    expect(screen.getByText("users")).toBeInTheDocument();
    expect(screen.getByText("posts")).toBeInTheDocument();
    expect(screen.getByText("comments")).toBeInTheDocument();
  });

  it("should show change count for each table group", () => {
    render(<DiffTreeWrapper />);

    // The component shows the count as a number (e.g., "2"), not "2 changes"
    // users has 2 items, posts has 2 items, comments has 1 item
    expect(screen.getAllByText("2")).toHaveLength(2); // users and posts
    expect(screen.getAllByText("1")).toHaveLength(1); // comments
  });

  it("should expand table group when clicked", async () => {
    const user = userEvent.setup();

    render(<DiffTreeWrapper />);

    // Initially, items should not be visible (collapsed)
    expect(screen.queryByText("email")).not.toBeInTheDocument();

    // Click on the users table group to expand it
    await user.click(screen.getByText("users"));

    // Now items should be visible
    expect(screen.getByText("email")).toBeInTheDocument();
  });

  it("should call onSelectionChange when item checkbox is clicked", async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();

    render(
      <DiffTreeWrapper onSelectionChange={onSelectionChange} initialExpanded={new Set(["users"])} />
    );

    // Click on the first item checkbox in the expanded group
    const checkboxes = screen.getAllByRole("checkbox");
    // checkboxes[0] is the group checkbox, checkboxes[1] is first item
    await user.click(checkboxes[1]);

    expect(onSelectionChange).toHaveBeenCalled();
  });

  it("should call onItemClick when item row is clicked", async () => {
    const user = userEvent.setup();
    const onItemClick = vi.fn();

    render(<DiffTreeWrapper onItemClick={onItemClick} initialExpanded={new Set(["users"])} />);

    // Click on the email item row
    await user.click(screen.getByText("email"));

    expect(onItemClick).toHaveBeenCalledWith(mockItems[1]);
  });

  it("should select all items in group when group checkbox is clicked", async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();

    render(<DiffTreeWrapper onSelectionChange={onSelectionChange} />);

    // Click on the first group checkbox (users)
    const checkboxes = screen.getAllByRole("checkbox");
    await user.click(checkboxes[0]);

    expect(onSelectionChange).toHaveBeenCalledWith(new Set(["1", "2"]));
  });

  it("should deselect all items in group when all are selected and group checkbox is clicked", async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();

    render(
      <DiffTreeWrapper selectedItems={new Set(["1", "2"])} onSelectionChange={onSelectionChange} />
    );

    // Click on the first group checkbox (users)
    const checkboxes = screen.getAllByRole("checkbox");
    await user.click(checkboxes[0]);

    expect(onSelectionChange).toHaveBeenCalledWith(new Set());
  });

  it("should show selected items as checked", () => {
    render(<DiffTreeWrapper selectedItems={new Set(["1", "2"])} />);

    const checkboxes = screen.getAllByRole("checkbox");
    // First checkbox (users group) should be checked since all items are selected
    expect(checkboxes[0]).toHaveAttribute("data-state", "checked");
  });

  it("should render empty when no items provided", () => {
    const { container } = render(<DiffTreeWrapper items={[]} />);

    expect(container.querySelector("[data-radix-scroll-area-viewport]")).toBeInTheDocument();
  });

  it("should display diff type labels", () => {
    render(<DiffTreeWrapper initialExpanded={new Set(["users"])} />);

    expect(screen.getByText("diff.columnAdded")).toBeInTheDocument();
  });

  it("should toggle individual item selection", async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();

    render(
      <DiffTreeWrapper
        selectedItems={new Set(["1"])}
        onSelectionChange={onSelectionChange}
        initialExpanded={new Set(["users"])}
      />
    );

    // Toggle second item (email)
    const checkboxes = screen.getAllByRole("checkbox");
    // checkboxes: [0]=group, [1]=item "1" (users table), [2]=item "2" (email)
    await user.click(checkboxes[2]);

    // Should add the second item to selection
    expect(onSelectionChange).toHaveBeenCalledWith(new Set(["1", "2"]));
  });
});
