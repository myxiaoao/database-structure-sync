import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ThemeToggle } from "../ThemeToggle";

const mockSetTheme = vi.fn();

vi.mock("@/components/ThemeProvider", () => ({
  useTheme: () => ({
    theme: "system",
    setTheme: mockSetTheme,
    resolvedTheme: "light",
  }),
}));

describe("ThemeToggle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render theme toggle button", () => {
    render(<ThemeToggle />);

    expect(screen.getByRole("button")).toBeInTheDocument();
  });

  it("should have accessible label", () => {
    render(<ThemeToggle />);

    expect(screen.getByText("common.toggleTheme")).toBeInTheDocument();
  });

  it("should open dropdown menu when clicked", async () => {
    const user = userEvent.setup();

    render(<ThemeToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      expect(screen.getByRole("menuitem", { name: /common\.light/ })).toBeInTheDocument();
      expect(screen.getByRole("menuitem", { name: /common\.dark/ })).toBeInTheDocument();
      expect(screen.getByRole("menuitem", { name: /common\.system/ })).toBeInTheDocument();
    });
  });

  it("should call setTheme with light when light option is clicked", async () => {
    const user = userEvent.setup();

    render(<ThemeToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      expect(screen.getByRole("menuitem", { name: /common\.light/ })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("menuitem", { name: /common\.light/ }));

    expect(mockSetTheme).toHaveBeenCalledWith("light");
  });

  it("should call setTheme with dark when dark option is clicked", async () => {
    const user = userEvent.setup();

    render(<ThemeToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      expect(screen.getByRole("menuitem", { name: /common\.dark/ })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("menuitem", { name: /common\.dark/ }));

    expect(mockSetTheme).toHaveBeenCalledWith("dark");
  });

  it("should call setTheme with system when system option is clicked", async () => {
    const user = userEvent.setup();

    render(<ThemeToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      expect(screen.getByRole("menuitem", { name: /common\.system/ })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("menuitem", { name: /common\.system/ }));

    expect(mockSetTheme).toHaveBeenCalledWith("system");
  });

  it("should show checkmark for current theme", async () => {
    const user = userEvent.setup();

    render(<ThemeToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      // System should have checkmark since mocked theme is system
      const systemItem = screen.getByRole("menuitem", { name: /common\.system/ });
      expect(systemItem.textContent).toContain("✓");
    });
  });

  it("should render Sun icon when resolved theme is light", () => {
    const { container } = render(<ThemeToggle />);

    // Check for SVG element
    const svg = container.querySelector("svg");
    expect(svg).toBeInTheDocument();
  });
});
