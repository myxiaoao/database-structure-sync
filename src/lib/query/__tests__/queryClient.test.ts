import { describe, it, expect } from "vitest";
import { queryClient } from "../queryClient";

describe("queryClient", () => {
  it("should be a QueryClient instance", () => {
    expect(queryClient).toBeDefined();
    expect(queryClient.getDefaultOptions).toBeDefined();
  });

  it("should have staleTime configured to 1 minute", () => {
    const options = queryClient.getDefaultOptions();
    expect(options.queries?.staleTime).toBe(1000 * 60);
  });

  it("should have retry set to 1", () => {
    const options = queryClient.getDefaultOptions();
    expect(options.queries?.retry).toBe(1);
  });

  it("should have refetchOnWindowFocus disabled", () => {
    const options = queryClient.getDefaultOptions();
    expect(options.queries?.refetchOnWindowFocus).toBe(false);
  });

  it("should be able to set and get query data", () => {
    queryClient.setQueryData(["test-key"], { value: "test" });
    const data = queryClient.getQueryData(["test-key"]);
    expect(data).toEqual({ value: "test" });

    // Clean up
    queryClient.removeQueries({ queryKey: ["test-key"] });
  });

  it("should be able to invalidate queries", async () => {
    queryClient.setQueryData(["invalidate-test"], { value: "old" });

    await queryClient.invalidateQueries({ queryKey: ["invalidate-test"] });

    const queryState = queryClient.getQueryState(["invalidate-test"]);
    expect(queryState?.isInvalidated).toBe(true);

    // Clean up
    queryClient.removeQueries({ queryKey: ["invalidate-test"] });
  });
});
