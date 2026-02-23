import { describe, it, expect, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Sidebar } from "../Sidebar";
import type { Connection } from "@/types";

const mockConnections: Connection[] = [
  {
    id: "1",
    name: "Production DB",
    db_type: "MySQL",
    host: "prod.example.com",
    port: 3306,
    username: "admin",
    password: "secret",
    database: "app_prod",
    ssl_config: {
      enabled: true,
      verify_server: true,
    },
  },
  {
    id: "2",
    name: "Development DB",
    db_type: "PostgreSQL",
    host: "localhost",
    port: 5432,
    username: "dev",
    password: "dev123",
    database: "app_dev",
  },
];

describe("Sidebar", () => {
  it("should render sidebar with title", () => {
    render(<Sidebar connections={[]} />);

    expect(screen.getByText("connection.title")).toBeInTheDocument();
  });

  it("should render connection list", () => {
    render(<Sidebar connections={mockConnections} />);

    expect(screen.getByText("Production DB")).toBeInTheDocument();
    expect(screen.getByText("Development DB")).toBeInTheDocument();
  });

  it("should render new connection button", () => {
    render(<Sidebar connections={[]} />);

    expect(screen.getByRole("button", { name: /connection\.new/i })).toBeInTheDocument();
  });

  it("should call onNewConnection when new button is clicked", async () => {
    const user = userEvent.setup();
    const onNewConnection = vi.fn();

    render(<Sidebar connections={[]} onNewConnection={onNewConnection} />);

    await user.click(screen.getByRole("button", { name: /connection\.new/i }));

    expect(onNewConnection).toHaveBeenCalled();
  });

  it("should call onSelectConnection when connection is clicked", async () => {
    const user = userEvent.setup();
    const onSelectConnection = vi.fn();

    render(<Sidebar connections={mockConnections} onSelectConnection={onSelectConnection} />);

    await user.click(screen.getByText("Production DB"));

    expect(onSelectConnection).toHaveBeenCalledWith("1");
  });

  it("should highlight selected connection", () => {
    const { container } = render(<Sidebar connections={mockConnections} selectedId="1" />);

    const selectedItem = container.querySelector(".bg-muted");
    expect(selectedItem).toBeInTheDocument();
  });

  it("should expand connection details when chevron is clicked", async () => {
    const user = userEvent.setup();

    render(<Sidebar connections={mockConnections} />);

    // Initially details should not be visible
    expect(screen.queryByText("prod.example.com:3306")).not.toBeInTheDocument();

    // Find the chevron button within the first connection row
    // The connection row contains: chevron button, database icon, name, dropdown button
    const prodRow = screen.getByText("Production DB").closest("div[class*='flex items-center']")!;
    const chevronButton = within(prodRow).getAllByRole("button")[0];
    await user.click(chevronButton);

    // Now details should be visible (format: host:port)
    expect(screen.getByText("MySQL")).toBeInTheDocument();
    expect(screen.getByText("prod.example.com:3306")).toBeInTheDocument();
  });

  it("should show edit option in dropdown menu", async () => {
    const user = userEvent.setup();

    render(<Sidebar connections={mockConnections} />);

    // Find and click the actions button (...)
    const actionsButtons = screen.getAllByRole("button", { name: /actions/i });
    await user.click(actionsButtons[0]);

    expect(screen.getByText("connection.edit")).toBeInTheDocument();
  });

  it("should show delete option in dropdown menu", async () => {
    const user = userEvent.setup();

    render(<Sidebar connections={mockConnections} />);

    // Find and click the actions button (...)
    const actionsButtons = screen.getAllByRole("button", { name: /actions/i });
    await user.click(actionsButtons[0]);

    expect(screen.getByText("connection.delete")).toBeInTheDocument();
  });

  it("should call onEditConnection when edit is clicked", async () => {
    const user = userEvent.setup();
    const onEditConnection = vi.fn();

    render(<Sidebar connections={mockConnections} onEditConnection={onEditConnection} />);

    // Open dropdown
    const actionsButtons = screen.getAllByRole("button", { name: /actions/i });
    await user.click(actionsButtons[0]);

    // Click edit
    await user.click(screen.getByText("connection.edit"));

    expect(onEditConnection).toHaveBeenCalledWith("1");
  });

  it("should call onDeleteConnection when delete is clicked", async () => {
    const user = userEvent.setup();
    const onDeleteConnection = vi.fn();

    render(<Sidebar connections={mockConnections} onDeleteConnection={onDeleteConnection} />);

    // Open dropdown
    const actionsButtons = screen.getAllByRole("button", { name: /actions/i });
    await user.click(actionsButtons[0]);

    // Click delete
    await user.click(screen.getByText("connection.delete"));

    expect(onDeleteConnection).toHaveBeenCalledWith("1");
  });

  it("should render with empty connections array", () => {
    render(<Sidebar connections={[]} />);

    expect(screen.getByText("connection.title")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /connection\.new/i })).toBeInTheDocument();
  });

  it("should collapse expanded connection when clicked again", async () => {
    const user = userEvent.setup();

    render(<Sidebar connections={mockConnections} />);

    // Find the chevron button
    const prodRow = screen.getByText("Production DB").closest("div[class*='flex items-center']")!;
    const chevronButton = within(prodRow).getAllByRole("button")[0];

    // Expand
    await user.click(chevronButton);
    expect(screen.getByText("prod.example.com:3306")).toBeInTheDocument();

    // Collapse
    await user.click(chevronButton);
    expect(screen.queryByText("prod.example.com:3306")).not.toBeInTheDocument();
  });
});
