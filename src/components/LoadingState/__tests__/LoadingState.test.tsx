import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { ConnectionListSkeleton, SyncPageSkeleton, TableSkeleton } from "../LoadingState";

describe("ConnectionListSkeleton", () => {
  it("should render 3 skeleton item containers", () => {
    const { container } = render(<ConnectionListSkeleton />);

    // Should have 3 item containers
    const items = container.querySelectorAll(".flex.items-center.gap-2.p-2");
    expect(items.length).toBe(3);
  });

  it("should render skeleton structure with proper layout", () => {
    const { container } = render(<ConnectionListSkeleton />);

    // Should have space-y-2 container
    const spaceContainer = container.querySelector(".space-y-2");
    expect(spaceContainer).toBeInTheDocument();
  });
});

describe("SyncPageSkeleton", () => {
  it("should render skeleton structure", () => {
    const { container } = render(<SyncPageSkeleton />);

    // Should have main flex column container
    const mainContainer = container.querySelector(".h-full.flex.flex-col");
    expect(mainContainer).toBeInTheDocument();
  });

  it("should render skeleton for diff tree area", () => {
    const { container } = render(<SyncPageSkeleton />);

    // Should have card structures (data-slot="card")
    const cards = container.querySelectorAll('[data-slot="card"]');
    expect(cards.length).toBeGreaterThan(0);
  });

  it("should render skeleton for SQL preview area", () => {
    const { container } = render(<SyncPageSkeleton />);

    // Should have a large skeleton for SQL preview
    const largeSkeletons = container.querySelectorAll(".h-48");
    expect(largeSkeletons.length).toBe(1);
  });

  it("should render in a two-column grid layout", () => {
    const { container } = render(<SyncPageSkeleton />);

    const gridContainer = container.querySelector(".grid-cols-2");
    expect(gridContainer).toBeInTheDocument();
  });
});

describe("TableSkeleton", () => {
  it("should render 5 skeleton rows by default", () => {
    const { container } = render(<TableSkeleton />);

    const skeletons = container.querySelectorAll(".h-10");
    expect(skeletons.length).toBe(5);
  });

  it("should render custom number of rows when specified", () => {
    const { container } = render(<TableSkeleton rows={10} />);

    const skeletons = container.querySelectorAll(".h-10");
    expect(skeletons.length).toBe(10);
  });

  it("should render single row when rows=1", () => {
    const { container } = render(<TableSkeleton rows={1} />);

    const skeletons = container.querySelectorAll(".h-10");
    expect(skeletons.length).toBe(1);
  });

  it("should render no rows when rows=0", () => {
    const { container } = render(<TableSkeleton rows={0} />);

    const skeletons = container.querySelectorAll(".h-10");
    expect(skeletons.length).toBe(0);
  });

  it("should have proper container structure", () => {
    const { container } = render(<TableSkeleton />);

    const spaceContainer = container.querySelector(".space-y-2");
    expect(spaceContainer).toBeInTheDocument();
  });
});
