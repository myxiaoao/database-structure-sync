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

  it("should show error message when save fails with Error", async () => {
    const user = userEvent.setup();
    mockOnSave.mockRejectedValue(new Error("Save failed: duplicate name"));

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
      expect(screen.getByText("Save failed: duplicate name")).toBeInTheDocument();
    });

    // onOpenChange should NOT have been called with false since save failed
    expect(mockOnOpenChange).not.toHaveBeenCalledWith(false);
  });

  it("should show stringified error message when save fails with non-Error", async () => {
    const user = userEvent.setup();
    mockOnSave.mockRejectedValue("raw string error");

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
      expect(screen.getByText("raw string error")).toBeInTheDocument();
    });
  });

  it("should show stringified error message when test fails with non-Error", async () => {
    const user = userEvent.setup();
    mockOnTest.mockRejectedValue("test string error");

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
      expect(screen.getByText("test string error")).toBeInTheDocument();
    });
  });

  it("should show private key fields when SSH auth method is PrivateKey", async () => {
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
    });

    // Default auth method is Password, so ssh_password field should be visible
    expect(screen.getByLabelText("connection.sshPasswordField")).toBeInTheDocument();

    // Switch auth method to PrivateKey
    // There are two comboboxes: db_type (in basic tab, hidden) and ssh_auth_method
    const authComboboxes = screen.getAllByRole("combobox");
    // The SSH auth method combobox is the last one visible
    const authSelect = authComboboxes[authComboboxes.length - 1];
    await user.click(authSelect);

    const privateKeyOption = screen.getByRole("option", { name: "connection.sshPrivateKey" });
    await user.click(privateKeyOption);

    await waitFor(() => {
      expect(screen.getByLabelText("connection.sshPrivateKeyPath")).toBeInTheDocument();
      expect(screen.getByLabelText("connection.sshPassphrase")).toBeInTheDocument();
    });

    // Password field should no longer be visible
    expect(screen.queryByLabelText("connection.sshPasswordField")).not.toBeInTheDocument();
  });

  it("should reset form to defaults when connection changes from existing to null", async () => {
    const { rerender } = renderWithProviders(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        connection={mockConnection}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    // Verify the form has connection values
    expect(screen.getByLabelText("connection.name")).toHaveValue("Test Connection");
    expect(screen.getByLabelText("connection.username")).toHaveValue("root");

    // Rerender with connection set to null (simulating dialog close and reopen for new)
    rerender(
      <ConnectionForm
        open={true}
        onOpenChange={mockOnOpenChange}
        connection={null}
        onSave={mockOnSave}
        onTest={mockOnTest}
      />
    );

    // Form should reset to defaults
    await waitFor(() => {
      expect(screen.getByLabelText("connection.name")).toHaveValue("");
      expect(screen.getByLabelText("connection.host")).toHaveValue("localhost");
      expect(screen.getByLabelText("connection.port")).toHaveValue(3306);
      expect(screen.getByLabelText("connection.username")).toHaveValue("");
    });
  });
});
