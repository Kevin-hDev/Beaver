/* @vitest-environment jsdom */
import { act, renderHook } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import { useAdvancedLoader } from "./use-advanced-loader";

const fixture = vi.hoisted(() => ({
  load: vi.fn(), refresh: vi.fn(() => Promise.resolve(true)),
  state: { mode: { kind: "retryInterruptedUi", extensionId: "com.example.failed", attempts: 2 } },
}));
vi.mock("@/hooks/use-extensions", () => ({ useExtensions: () => ({ extensions: [] }) }));
vi.mock("@/hooks/use-extension-ui-startup", () => ({
  useExtensionUiStartupContext: () => ({ state: fixture.state, refresh: fixture.refresh }),
}));
vi.mock("./advanced-loader", () => ({ loadAdvancedModules: fixture.load }));

beforeEach(() => vi.clearAllMocks());

it("refreshes the backend recovery state after an aborted activation", async () => {
  fixture.load.mockRejectedValueOnce(new Error("fixture"));
  const hook = renderHook(useAdvancedLoader);
  await act(async () => { await Promise.resolve(); });
  expect(fixture.refresh).toHaveBeenCalledOnce();
  hook.unmount();
});

it("does not refresh from a failure belonging to an unmounted generation", async () => {
  let reject!: (error: Error) => void;
  fixture.load.mockReturnValueOnce(new Promise((_resolve, fail) => { reject = fail; }));
  const hook = renderHook(useAdvancedLoader);
  hook.unmount();
  await act(() => { reject(new Error("fixture")); return Promise.resolve(); });
  expect(fixture.refresh).not.toHaveBeenCalled();
});
