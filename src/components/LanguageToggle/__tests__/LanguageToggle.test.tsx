import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LanguageToggle } from "../LanguageToggle";

const mockChangeLanguage = vi.fn();

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      language: "en-US",
      changeLanguage: mockChangeLanguage,
    },
  }),
}));

describe("LanguageToggle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render language toggle button", () => {
    render(<LanguageToggle />);

    expect(screen.getByRole("button")).toBeInTheDocument();
  });

  it("should have accessible label", () => {
    render(<LanguageToggle />);

    expect(screen.getByText("common.language")).toBeInTheDocument();
  });

  it("should open dropdown menu when clicked", async () => {
    const user = userEvent.setup();

    render(<LanguageToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      expect(screen.getByRole("menuitem", { name: /English/ })).toBeInTheDocument();
      expect(screen.getByRole("menuitem", { name: /简体中文/ })).toBeInTheDocument();
    });
  });

  it("should show checkmark for current language", async () => {
    const user = userEvent.setup();

    render(<LanguageToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      // English should have checkmark since mocked language is en-US
      const englishItem = screen.getByRole("menuitem", { name: /English/ });
      expect(englishItem.textContent).toContain("✓");
    });
  });

  it("should call changeLanguage when English is selected", async () => {
    const user = userEvent.setup();

    render(<LanguageToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      expect(screen.getByRole("menuitem", { name: /English/ })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("menuitem", { name: /English/ }));

    expect(mockChangeLanguage).toHaveBeenCalledWith("en-US");
  });

  it("should call changeLanguage when Chinese is selected", async () => {
    const user = userEvent.setup();

    render(<LanguageToggle />);

    await user.click(screen.getByRole("button"));

    await waitFor(() => {
      expect(screen.getByRole("menuitem", { name: /简体中文/ })).toBeInTheDocument();
    });

    await user.click(screen.getByRole("menuitem", { name: /简体中文/ }));

    expect(mockChangeLanguage).toHaveBeenCalledWith("zh-CN");
  });

  it("should render Globe icon", () => {
    const { container } = render(<LanguageToggle />);

    // Check for SVG element (Globe icon)
    const svg = container.querySelector("svg");
    expect(svg).toBeInTheDocument();
  });
});
