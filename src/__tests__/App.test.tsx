import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { renderWithProviders } from "@/test/utils";
import App from "../App";
import type { Connection } from "@/types";

const mockConnections: Connection[] = [
  {
    id: "conn-1",
    name: "Dev DB",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "password",
    database: "dev_db",
  },
  {
    id: "conn-2",
    name: "Staging DB",
    db_type: "PostgreSQL",
    host: "staging.example.com",
    port: 5432,
    username: "admin",
    password: "secret",
    database: "staging_db",
  },
];

const mockOpenNewConnection = vi.fn();
const mockOpenEditConnection = vi.fn();
const mockCloseForm = vi.fn();
const mockSaveConnection = vi.fn();
const mockDeleteConnection = vi.fn();
const mockTestConnection = vi.fn();

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
}));

vi.mock("../hooks", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../hooks")>();
  return {
    ...actual,
    useConnections: vi.fn(),
  };
});

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

import { useConnections } from "../hooks";

const mockUseConnections = vi.mocked(useConnections);

function setupMock(overrides: Partial<ReturnType<typeof useConnections>> = {}) {
  mockUseConnections.mockReturnValue({
    connections: mockConnections,
    isLoading: false,
    error: null,
    refetch: vi.fn(),
    editingConnection: null,
    isFormOpen: false,
    openNewConnection: mockOpenNewConnection,
    openEditConnection: mockOpenEditConnection,
    closeForm: mockCloseForm,
    saveConnection: mockSaveConnection,
    deleteConnection: mockDeleteConnection,
    testConnection: mockTestConnection,
    isSaving: false,
    isDeleting: false,
    isTesting: false,
    ...overrides,
  });
}

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setupMock();
  });

  it("renders without crashing", () => {
    renderWithProviders(<App />);
    expect(screen.getByText("app.title")).toBeInTheDocument();
  });

  it("passes connections to MainLayout and renders connection names in sidebar", () => {
    renderWithProviders(<App />);
    expect(screen.getByText("Dev DB")).toBeInTheDocument();
    expect(screen.getByText("Staging DB")).toBeInTheDocument();
  });

  it("opens new connection form when new connection button is clicked", async () => {
    const user = userEvent.setup();
    renderWithProviders(<App />);

    const newButton = screen.getByRole("button", { name: /connection\.new/i });
    await user.click(newButton);
    expect(mockOpenNewConnection).toHaveBeenCalled();
  });

  it("opens edit connection form for existing connection id via dropdown", async () => {
    const user = userEvent.setup();
    renderWithProviders(<App />);

    // Open dropdown for first connection
    const actionsButtons = screen.getAllByRole("button", { name: /actions/i });
    await user.click(actionsButtons[0]);

    // Click edit
    await user.click(screen.getByText("connection.edit"));
    expect(mockOpenEditConnection).toHaveBeenCalledWith(mockConnections[0]);
  });

  it("does not crash when rendered with empty connections", () => {
    setupMock({ connections: [] });
    renderWithProviders(<App />);
    expect(screen.getByText("app.title")).toBeInTheDocument();
    expect(mockOpenEditConnection).not.toHaveBeenCalled();
  });

  it("passes deleteConnection to MainLayout via dropdown", async () => {
    const user = userEvent.setup();
    renderWithProviders(<App />);

    // Open dropdown for first connection
    const actionsButtons = screen.getAllByRole("button", { name: /actions/i });
    await user.click(actionsButtons[0]);

    // Click delete
    await user.click(screen.getByText("connection.delete"));
    expect(mockDeleteConnection).toHaveBeenCalledWith("conn-1");
  });

  it("renders SyncPage with source and target labels", () => {
    renderWithProviders(<App />);
    expect(screen.getByText("sync.source")).toBeInTheDocument();
    expect(screen.getByText("sync.target")).toBeInTheDocument();
  });

  it("does not show connection form when isFormOpen is false", () => {
    renderWithProviders(<App />);
    expect(screen.queryByLabelText("connection.name")).not.toBeInTheDocument();
  });

  it("shows connection form when isFormOpen is true", () => {
    setupMock({ isFormOpen: true });
    renderWithProviders(<App />);
    expect(screen.getByLabelText("connection.name")).toBeInTheDocument();
  });
});
