import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { useExtensionInstall, useExtensionUpdate } from "./use-extension-install";

const { start } = vi.hoisted(() => ({ start: vi.fn() }));
vi.mock("./use-extension-install-jobs", () => ({ useExtensionInstallJobs: () => ({ start }) }));
afterEach(cleanup);
it("leaves dialog admission errors to the dialog and banners only managed update errors", async () => {
  start.mockResolvedValue({ job: null, errorKey: "extensionInstalls.errors.start" });
  const banner = vi.fn();
  const view = renderHook(() => {
    const install = useExtensionInstall();
    return { install, update: useExtensionUpdate(install, banner) };
  });
  const result = await act(() => view.result.current.install({ kind: "npm", locator: "fixture" }));
  expect(result.errorKey).toBe("extensionInstalls.errors.start");
  expect(banner).not.toHaveBeenCalled();
  await act(() => view.result.current.update("fixture"));
  expect(banner.mock.calls).toEqual([[null], ["extensionInstalls.errors.start"]]);
});
