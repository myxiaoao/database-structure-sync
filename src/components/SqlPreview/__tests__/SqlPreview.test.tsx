import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { SqlPreview } from "../SqlPreview";

describe("SqlPreview", () => {
  it("should render SQL content when provided", () => {
    const sql = "CREATE TABLE users (id INT PRIMARY KEY);";
    render(<SqlPreview sql={sql} />);

    expect(screen.getByText(sql)).toBeInTheDocument();
  });

  it("should display title", () => {
    render(<SqlPreview sql="SELECT * FROM users" />);

    expect(screen.getByText("sql.preview")).toBeInTheDocument();
  });

  it("should show empty message when no SQL provided", () => {
    render(<SqlPreview sql="" />);

    expect(screen.getByText("sql.empty")).toBeInTheDocument();
  });

  it("should render SQL in a pre element for proper formatting", () => {
    const sql = "SELECT * FROM users;";
    render(<SqlPreview sql={sql} />);

    const preElement = screen.getByText(sql);
    expect(preElement.tagName).toBe("PRE");
  });

  it("should preserve whitespace in SQL", () => {
    const sql = "SELECT * FROM users;";
    const { container } = render(<SqlPreview sql={sql} />);

    const preElement = container.querySelector("pre");
    expect(preElement).toBeInTheDocument();
    expect(preElement?.textContent).toBe(sql);
  });

  it("should handle multiline SQL statements", () => {
    const sql = "CREATE TABLE users (id INT);";
    const { container } = render(<SqlPreview sql={sql} />);

    const preElement = container.querySelector("pre");
    expect(preElement?.textContent).toContain("CREATE TABLE");
  });

  it("should handle special characters in SQL", () => {
    const sql = "SELECT * FROM users WHERE name LIKE '%test%' AND age > 18;";
    render(<SqlPreview sql={sql} />);

    expect(screen.getByText(sql)).toBeInTheDocument();
  });
});
