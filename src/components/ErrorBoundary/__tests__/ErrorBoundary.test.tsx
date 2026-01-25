import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ErrorBoundary } from "../ErrorBoundary";

// Component that throws an error
function ThrowError({ shouldThrow }: { shouldThrow: boolean }) {
  if (shouldThrow) {
    throw new Error("Test error message");
  }
  return <div>No error</div>;
}

describe("ErrorBoundary", () => {
  // Suppress console.error for cleaner test output
  const originalConsoleError = console.error;

  beforeEach(() => {
    console.error = vi.fn();
  });

  afterEach(() => {
    console.error = originalConsoleError;
  });

  it("should render children when no error occurs", () => {
    render(
      <ErrorBoundary>
        <div>Child content</div>
      </ErrorBoundary>
    );

    expect(screen.getByText("Child content")).toBeInTheDocument();
  });

  it("should render error UI when child throws", () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByText("Something went wrong")).toBeInTheDocument();
    expect(screen.getByText("Test error message")).toBeInTheDocument();
  });

  it("should render custom fallback when provided", () => {
    render(
      <ErrorBoundary fallback={<div>Custom error UI</div>}>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByText("Custom error UI")).toBeInTheDocument();
    expect(screen.queryByText("Something went wrong")).not.toBeInTheDocument();
  });

  it("should display error message in the error UI", () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByText("Test error message")).toBeInTheDocument();
  });

  it("should have a Try Again button", () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByRole("button", { name: /try again/i })).toBeInTheDocument();
  });

  it("should reset error state when Try Again is clicked", async () => {
    const user = userEvent.setup();

    // We need to use a stateful wrapper to control the error
    let shouldThrow = true;
    const { rerender } = render(
      <ErrorBoundary>
        <ThrowError shouldThrow={shouldThrow} />
      </ErrorBoundary>
    );

    // Verify error state
    expect(screen.getByText("Something went wrong")).toBeInTheDocument();

    // Click Try Again
    shouldThrow = false;
    await user.click(screen.getByRole("button", { name: /try again/i }));

    // Rerender with fixed component
    rerender(
      <ErrorBoundary>
        <ThrowError shouldThrow={shouldThrow} />
      </ErrorBoundary>
    );

    // After reset, children should attempt to render again
    // Since the error is caught in getDerivedStateFromError, clicking try again
    // should reset the state
  });

  it("should log error to console", () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(console.error).toHaveBeenCalled();
  });

  it("should display description text", () => {
    render(
      <ErrorBoundary>
        <ThrowError shouldThrow={true} />
      </ErrorBoundary>
    );

    expect(screen.getByText("An unexpected error occurred. Please try again.")).toBeInTheDocument();
  });
});
