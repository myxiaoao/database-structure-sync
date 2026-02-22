import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SyncPage } from "../SyncPage";
import { renderWithProviders } from "@/test/utils";
import type { Connection, DiffResult } from "@/types";

const mockConnections: Connection[] = [
  {
    id: "source-id",
    name: "Source DB",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "password",
    database: "source_db",
    ssh_enabled: false,
    ssl_enabled: false,
  },
  {
    id: "target-id",
    name: "Target DB",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "password",
    database: "target_db",
    ssh_enabled: false,
    ssl_enabled: false,
  },
];

const mockDiffResult: DiffResult = {
  items: [
    {
      id: "diff-1",
      diff_type: "TableAdded",
      table_name: "users",
      sql: "CREATE TABLE users (id INT PRIMARY KEY);",
      selected: false,
    },
    {
      id: "diff-2",
      diff_type: "ColumnAdded",
      table_name: "posts",
      object_name: "title",
      sql: "ALTER TABLE posts ADD COLUMN title VARCHAR(255);",
      selected: false,
    },
  ],
  source_tables: 5,
  target_tables: 3,
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = vi.mocked(invoke);

describe("SyncPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([]);
  });

  it("should render connection selectors", () => {
    renderWithProviders(<SyncPage connections={mockConnections} />);

    expect(screen.getByText("sync.source")).toBeInTheDocument();
    expect(screen.getByText("sync.target")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "sync.compare" })).toBeInTheDocument();
  });

  it("should render compare button as disabled when no connections selected", () => {
    renderWithProviders(<SyncPage connections={mockConnections} />);

    const compareButton = screen.getByRole("button", { name: "sync.compare" });
    expect(compareButton).toBeDisabled();
  });

  it("should display connections in dropdowns", async () => {
    const user = userEvent.setup();

    renderWithProviders(<SyncPage connections={mockConnections} />);

    // Find and click the first combobox (source)
    const comboboxes = screen.getAllByRole("combobox");
    await user.click(comboboxes[0]);

    await waitFor(() => {
      expect(screen.getByText("Source DB (MySQL)")).toBeInTheDocument();
      expect(screen.getByText("Target DB (MySQL)")).toBeInTheDocument();
    });
  });

  it("should enable compare button when both connections are selected", async () => {
    const user = userEvent.setup();

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select source
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));

    // Select target
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));

    const compareButton = screen.getByRole("button", { name: "sync.compare" });
    expect(compareButton).not.toBeDisabled();
  });

  it("should perform comparison when compare button is clicked", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(mockDiffResult);

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select source
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));

    // Select target
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));

    // Click compare
    const compareButton = screen.getByRole("button", { name: "sync.compare" });
    await user.click(compareButton);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("compare_databases", {
        sourceId: "source-id",
        targetId: "target-id",
        sourceDatabase: undefined,
        targetDatabase: undefined,
      });
    });
  });

  it("should display diff results after comparison", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(mockDiffResult);

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select connections
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));

    // Click compare
    await user.click(screen.getByRole("button", { name: "sync.compare" }));

    await waitFor(() => {
      expect(screen.getByText("2 sync.changes")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "sync.selectAll" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "sync.deselectAll" })).toBeInTheDocument();
    });
  });

  it("should display no changes message when diff is empty", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue({ items: [], source_tables: 5, target_tables: 5 });

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select connections
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));

    // Click compare
    await user.click(screen.getByRole("button", { name: "sync.compare" }));

    await waitFor(() => {
      expect(screen.getByText("sync.noChanges")).toBeInTheDocument();
    });
  });

  it("should have execute button disabled when no items selected", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(mockDiffResult);

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select connections
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));

    // Click compare
    await user.click(screen.getByRole("button", { name: "sync.compare" }));

    await waitFor(() => {
      const executeButton = screen.getByRole("button", { name: "sync.execute" });
      expect(executeButton).toBeDisabled();
    });
  });

  it("should render with empty connections list", () => {
    renderWithProviders(<SyncPage connections={[]} />);

    expect(screen.getByText("sync.source")).toBeInTheDocument();
    expect(screen.getByText("sync.target")).toBeInTheDocument();
  });

  it("should show sql.empty message when no items are selected after comparison", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(mockDiffResult);

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select connections
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));

    // Click compare
    await user.click(screen.getByRole("button", { name: "sync.compare" }));

    // Wait for diff results to appear
    await waitFor(() => {
      expect(screen.getByText("2 sync.changes")).toBeInTheDocument();
    });

    // No items selected, so SQL preview should show empty message
    expect(screen.getByText("sql.empty")).toBeInTheDocument();
  });

  it("should have export SQL button disabled when no SQL to export", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(mockDiffResult);

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select connections
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));

    // Click compare
    await user.click(screen.getByRole("button", { name: "sync.compare" }));

    await waitFor(() => {
      expect(screen.getByText("2 sync.changes")).toBeInTheDocument();
    });

    // No items selected, export button should be disabled
    const exportButton = screen.getByRole("button", { name: /sql\.export/ });
    expect(exportButton).toBeDisabled();
  });

  it("should render expandAll and collapseAll buttons after comparison", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(mockDiffResult);

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select connections
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));

    // Click compare
    await user.click(screen.getByRole("button", { name: "sync.compare" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "sync.expandAll" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "sync.collapseAll" })).toBeInTheDocument();
    });
  });

  it("should expand all items when expandAll is clicked", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(mockDiffResult);

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select connections and compare
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));
    await user.click(screen.getByRole("button", { name: "sync.compare" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "sync.expandAll" })).toBeInTheDocument();
    });

    // Click expand all
    await user.click(screen.getByRole("button", { name: "sync.expandAll" }));

    // After expanding, individual diff items should be visible (e.g., column names)
    await waitFor(() => {
      // The diff items have object_name "title" for diff-2
      expect(screen.getByText("title")).toBeInTheDocument();
    });
  });

  it("should collapse all items when collapseAll is clicked after expanding", async () => {
    const user = userEvent.setup();
    mockInvoke.mockResolvedValue(mockDiffResult);

    renderWithProviders(<SyncPage connections={mockConnections} />);

    const comboboxes = screen.getAllByRole("combobox");

    // Select connections and compare
    await user.click(comboboxes[0]);
    await user.click(screen.getByText("Source DB (MySQL)"));
    await user.click(comboboxes[1]);
    await user.click(screen.getByText("Target DB (MySQL)"));
    await user.click(screen.getByRole("button", { name: "sync.compare" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "sync.expandAll" })).toBeInTheDocument();
    });

    // Expand all first
    await user.click(screen.getByRole("button", { name: "sync.expandAll" }));

    await waitFor(() => {
      expect(screen.getByText("title")).toBeInTheDocument();
    });

    // Now collapse all
    await user.click(screen.getByRole("button", { name: "sync.collapseAll" }));

    // After collapsing, the detail items should no longer be visible
    await waitFor(() => {
      expect(screen.queryByText("title")).not.toBeInTheDocument();
    });
  });
});
