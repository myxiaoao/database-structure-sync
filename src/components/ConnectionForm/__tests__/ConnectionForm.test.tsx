import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConnectionForm } from "../ConnectionForm";
import { renderWithProviders } from "@/test/utils";
import type { Connection } from "@/types";

const mockConnection: Connection = {
  id: "test-id",
  name: "Test Connection",
  db_type: "MySQL",
  host: "localhost",
  port: 3306,
  username: "root",
  password: "password",
  database: "test_db",
  ssh_enabled: false,
  ssl_enabled: false,
};

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

describe("ConnectionForm", () => {
  const mockOnSave = vi.fn();
  const mockOnTest = vi.fn();
  const mockOnOpenChange = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    mockOnSave.mockResolvedValue(undefined);
    mockOnTest.mockResolvedValue(undefined);
  });

  it("should render form with default values for new connection", () => {
    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    expect(screen.getByLabelText("connection.name")).toHaveValue("");
    expect(screen.getByLabelText("connection.host")).toHaveValue("localhost");
    expect(screen.getByLabelText("connection.port")).toHaveValue(3306);
  });

  it("should render form with existing connection values", () => {
    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        connection={mockConnection}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    expect(screen.getByLabelText("connection.name")).toHaveValue("Test Connection");
    expect(screen.getByLabelText("connection.host")).toHaveValue("localhost");
    expect(screen.getByLabelText("connection.port")).toHaveValue(3306);
    expect(screen.getByLabelText("connection.username")).toHaveValue("root");
    expect(screen.getByLabelText("connection.database")).toHaveValue("test_db");
  });

  it("should update form fields on input", async () => {
    const user = userEvent.setup();

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    const nameInput = screen.getByLabelText("connection.name");
    await user.clear(nameInput);
    await user.type(nameInput, "New Connection");

    expect(nameInput).toHaveValue("New Connection");
  });

  it("should call onTest when test button is clicked", async () => {
    const user = userEvent.setup();

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        connection={mockConnection}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    const testButton = screen.getByRole("button", { name: "connection.test" });
    await user.click(testButton);

    await waitFor(() => {
      expect(mockOnTest).toHaveBeenCalledWith(
        expect.objectContaining({
          name: "Test Connection",
          host: "localhost",
          port: 3306,
        })
      );
    });
  });

  it("should show success message after successful test", async () => {
    const user = userEvent.setup();
    mockOnTest.mockResolvedValue(undefined);

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        connection={mockConnection}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    const testButton = screen.getByRole("button", { name: "connection.test" });
    await user.click(testButton);

    await waitFor(() => {
      expect(screen.getByText("connection.testSuccess")).toBeInTheDocument();
    });
  });

  it("should show error message after failed test", async () => {
    const user = userEvent.setup();
    mockOnTest.mockRejectedValue(new Error("Connection refused"));

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        connection={mockConnection}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    const testButton = screen.getByRole("button", { name: "connection.test" });
    await user.click(testButton);

    await waitFor(() => {
      expect(screen.getByText("Connection refused")).toBeInTheDocument();
    });
  });

  it("should call onSave and close dialog on successful save", async () => {
    const user = userEvent.setup();

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        connection={mockConnection}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    const saveButton = screen.getByRole("button", { name: "connection.save" });
    await user.click(saveButton);

    await waitFor(() => {
      expect(mockOnSave).toHaveBeenCalled();
      expect(mockOnOpenChange).toHaveBeenCalledWith(false);
    });
  });

  it("should display tabs for basic, SSH, and SSL settings", () => {
    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    expect(screen.getByRole("tab", { name: "connection.basicTab" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "connection.sshTab" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "connection.sslTab" })).toBeInTheDocument();
  });

  it("should show SSH fields when SSH is enabled", async () => {
    const user = userEvent.setup();

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    // Switch to SSH tab
    const sshTab = screen.getByRole("tab", { name: "connection.sshTab" });
    await user.click(sshTab);

    // Enable SSH
    const sshCheckbox = screen.getByRole("checkbox", { name: "connection.sshEnabled" });
    await user.click(sshCheckbox);

    await waitFor(() => {
      expect(screen.getByLabelText("connection.sshHost")).toBeInTheDocument();
      expect(screen.getByLabelText("connection.sshPort")).toBeInTheDocument();
      expect(screen.getByLabelText("connection.sshUsername")).toBeInTheDocument();
    });
  });

  it("should show SSL fields when SSL is enabled", async () => {
    const user = userEvent.setup();

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    // Switch to SSL tab
    const sslTab = screen.getByRole("tab", { name: "connection.sslTab" });
    await user.click(sslTab);

    // Enable SSL
    const sslCheckbox = screen.getByRole("checkbox", { name: "connection.sslEnabled" });
    await user.click(sslCheckbox);

    await waitFor(() => {
      expect(screen.getByLabelText("connection.sslCaCert")).toBeInTheDocument();
      expect(screen.getByLabelText("connection.sslClientCert")).toBeInTheDocument();
      expect(screen.getByLabelText("connection.sslClientKey")).toBeInTheDocument();
    });
  });

  it("should change default port when database type changes", async () => {
    const user = userEvent.setup();

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    // Initial port should be 3306 for MySQL
    expect(screen.getByLabelText("connection.port")).toHaveValue(3306);

    // Click on db type select and change to PostgreSQL
    const dbTypeSelect = screen.getByRole("combobox");
    await user.click(dbTypeSelect);

    const postgresOption = screen.getByRole("option", { name: "PostgreSQL" });
    await user.click(postgresOption);

    // Port should change to 5432
    await waitFor(() => {
      expect(screen.getByLabelText("connection.port")).toHaveValue(5432);
    });
  });

  it("should disable buttons while testing", async () => {
    const user = userEvent.setup();
    // Make test take a while
    mockOnTest.mockImplementation(() => new Promise((resolve) => setTimeout(resolve, 1000)));

    renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        connection={mockConnection}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    const testButton = screen.getByRole("button", { name: "connection.test" });
    const saveButton = screen.getByRole("button", { name: "connection.save" });

    await user.click(testButton);

    expect(testButton).toBeDisabled();
    expect(saveButton).toBeDisabled();
  });

  it("should not render when open is false", () => {
    renderWithProviders(
      <ConnectionForm
        open={false}
        onOpenChange={mockOnOpenChange}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    expect(screen.queryByLabelText("connection.name")).not.toBeInTheDocument();
  });
});
