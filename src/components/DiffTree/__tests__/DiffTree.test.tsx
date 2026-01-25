import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DiffTree } from "../DiffTree";
import type { DiffItem } from "@/types";

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

describe("DiffTree", () => {
  it("should render grouped items by table name", () => {
    render(<DiffTree items={mockItems} selectedItems={new Set()} onSelectionChange={vi.fn()} />);

    expect(screen.getByText("users")).toBeInTheDocument();
    expect(screen.getByText("posts")).toBeInTheDocument();
    expect(screen.getByText("comments")).toBeInTheDocument();
  });

  it("should show change count for each table group", () => {
    render(<DiffTree items={mockItems} selectedItems={new Set()} onSelectionChange={vi.fn()} />);

    // users has 2 items, posts has 2 items, comments has 1 item
    expect(screen.getAllByText("2 changes")).toHaveLength(2); // users and posts
    expect(screen.getAllByText("1 change")).toHaveLength(1); // comments
  });

  it("should expand table group when clicked", async () => {
    const user = userEvent.setup();

    render(<DiffTree items={mockItems} selectedItems={new Set()} onSelectionChange={vi.fn()} />);

    // Initially, items should not be visible (collapsed)
    expect(screen.queryByText("email")).not.toBeInTheDocument();

    // Click on the users table group
    await user.click(screen.getByText("users"));

    // Now items should be visible
    expect(screen.getByText("email")).toBeInTheDocument();
  });

  it("should call onSelectionChange when item checkbox is clicked", async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();

    render(
      <DiffTree items={mockItems} selectedItems={new Set()} onSelectionChange={onSelectionChange} />
    );

    // Expand users group
    await user.click(screen.getByText("users"));

    // Click on the first checkbox in the expanded group
    const checkboxes = screen.getAllByRole("checkbox");
    await user.click(checkboxes[1]); // First item checkbox (index 0 is group checkbox)

    expect(onSelectionChange).toHaveBeenCalled();
  });

  it("should call onItemClick when item row is clicked", async () => {
    const user = userEvent.setup();
    const onItemClick = vi.fn();

    render(
      <DiffTree
        items={mockItems}
        selectedItems={new Set()}
        onSelectionChange={vi.fn()}
        onItemClick={onItemClick}
      />
    );

    // Expand users group
    await user.click(screen.getByText("users"));

    // Click on the email item row
    await user.click(screen.getByText("email"));

    expect(onItemClick).toHaveBeenCalledWith(mockItems[1]);
  });

  it("should select all items in group when group checkbox is clicked", async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();

    render(
      <DiffTree items={mockItems} selectedItems={new Set()} onSelectionChange={onSelectionChange} />
    );

    // Click on the first group checkbox (users)
    const checkboxes = screen.getAllByRole("checkbox");
    await user.click(checkboxes[0]);

    expect(onSelectionChange).toHaveBeenCalledWith(new Set(["1", "2"]));
  });

  it("should deselect all items in group when all are selected and group checkbox is clicked", async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();

    render(
      <DiffTree
        items={mockItems}
        selectedItems={new Set(["1", "2"])}
        onSelectionChange={onSelectionChange}
      />
    );

    // Click on the first group checkbox (users)
    const checkboxes = screen.getAllByRole("checkbox");
    await user.click(checkboxes[0]);

    expect(onSelectionChange).toHaveBeenCalledWith(new Set());
  });

  it("should show selected items as checked", () => {
    render(
      <DiffTree items={mockItems} selectedItems={new Set(["1", "2"])} onSelectionChange={vi.fn()} />
    );

    const checkboxes = screen.getAllByRole("checkbox");
    // First checkbox (users group) should be checked since all items are selected
    expect(checkboxes[0]).toHaveAttribute("data-state", "checked");
  });

  it("should render empty when no items provided", () => {
    const { container } = render(
      <DiffTree items={[]} selectedItems={new Set()} onSelectionChange={vi.fn()} />
    );

    expect(container.querySelector("[data-radix-scroll-area-viewport]")).toBeInTheDocument();
  });

  it("should display diff type labels", async () => {
    const user = userEvent.setup();

    render(<DiffTree items={mockItems} selectedItems={new Set()} onSelectionChange={vi.fn()} />);

    // Expand users group
    await user.click(screen.getByText("users"));

    expect(screen.getByText("diff.columnAdded")).toBeInTheDocument();
  });

  it("should toggle individual item selection", async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();

    render(
      <DiffTree
        items={mockItems}
        selectedItems={new Set(["1"])}
        onSelectionChange={onSelectionChange}
      />
    );

    // Expand users group
    await user.click(screen.getByText("users"));

    // Toggle second item
    const checkboxes = screen.getAllByRole("checkbox");
    await user.click(checkboxes[2]); // email item

    // Should add the second item to selection
    expect(onSelectionChange).toHaveBeenCalledWith(new Set(["1", "2"]));
  });
});
