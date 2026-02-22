import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MainLayout } from "../MainLayout";
import type { Connection } from "@/types";

vi.mock("@/components/ThemeProvider", () => ({
  useTheme: () => ({
    theme: "system",
    setTheme: vi.fn(),
    resolvedTheme: "light",
  }),
}));

const mockConnections: Connection[] = [
  {
    id: "1",
    name: "Test DB",
    db_type: "MySQL",
    host: "localhost",
    port: 3306,
    username: "root",
    password: "password",
    database: "test",
    ssh_enabled: false,
    ssl_enabled: false,
  },
];

describe("MainLayout", () => {
  it("should render children content", () => {
    render(
      <MainLayout>
        <div>Child content</div>
      </MainLayout>
    );

    expect(screen.getByText("Child content")).toBeInTheDocument();
  });

  it("should render app title", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    expect(screen.getByText("app.title")).toBeInTheDocument();
  });

  it("should render sidebar with connections", () => {
    render(
      <MainLayout connections={mockConnections}>
        <div>Content</div>
      </MainLayout>
    );

    expect(screen.getByText("Test DB")).toBeInTheDocument();
  });

  it("should render theme toggle", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    expect(screen.getByText("common.toggleTheme")).toBeInTheDocument();
  });

  it("should render language toggle", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    expect(screen.getByText("common.language")).toBeInTheDocument();
  });

  it("should render AppLogo", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    // AppLogo renders an img
    const img = screen.getByAltText("Database Structure Sync");
    expect(img).toBeInTheDocument();
  });

  it("should pass connection handlers to sidebar", () => {
    const onNewConnection = vi.fn();
    const onEditConnection = vi.fn();
    const onDeleteConnection = vi.fn();

    render(
      <MainLayout
        connections={mockConnections}
        onNewConnection={onNewConnection}
        onEditConnection={onEditConnection}
        onDeleteConnection={onDeleteConnection}
      >
        <div>Content</div>
      </MainLayout>
    );

    // Sidebar should be rendered with connections
    expect(screen.getByText("Test DB")).toBeInTheDocument();
    // New connection button should be present
    expect(screen.getByRole("button", { name: /connection\.new/i })).toBeInTheDocument();
  });

  it("should render with default empty connections", () => {
    render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    expect(screen.getByText("connection.title")).toBeInTheDocument();
  });

  it("should have proper layout structure", () => {
    const { container } = render(
      <MainLayout>
        <div>Content</div>
      </MainLayout>
    );

    // Main container should be flex
    expect(container.querySelector(".flex.h-screen")).toBeInTheDocument();

    // Header should exist
    expect(container.querySelector("header")).toBeInTheDocument();

    // Main content area should exist
    expect(container.querySelector("main")).toBeInTheDocument();
  });
});
