import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { InstallJobView } from "@/types/extension-install-jobs.generated";
import { ExtensionInstallJobRow } from "./extension-install-job-row";

vi.mock("react-i18next", () => ({ useTranslation: () => ({
  i18n: { language: "fr" }, t: (key: string, values?: Record<string, string>) => values?.name ? `${key}: ${values.name}` : key,
}) }));
afterEach(cleanup);
const base: InstallJobView = {
  id: "10000000-0000-4000-8000-000000000001", revision: 1, kind: "npm", displayName: "Fixture",
  status: "running", phase: "dependencies", downloadedBytes: null, downloadTotalBytes: null,
  occupiedBytes: 2048, freeBytes: 4096, confirmationId: null, errorCode: null, extensionId: null,
  canCancel: true, canResume: false, queueBlocker: null,
};
function setup(change: Partial<InstallJobView>, errorKey?: string) {
  const actions = { cancel: vi.fn(), continue: vi.fn(), resume: vi.fn(), retry: vi.fn(), dismiss: vi.fn(), open: vi.fn() };
  const show = vi.fn();
  render(<ExtensionInstallJobRow job={{ ...base, ...change }} actions={actions} onShowRequest={show}
    blockerName="Grande extension" errorKey={errorKey} />);
  return { actions, show };
}
describe("extension install row", () => {
  it("shows backend phase without making up a percentage", () => {
    const { actions } = setup({});
    expect(screen.getByRole("progressbar")).not.toHaveAttribute("aria-valuenow");
    fireEvent.click(screen.getByRole("button", { name: "extensionInstalls.cancel" }));
    expect(actions.cancel).toHaveBeenCalledWith(base.id);
  });
  it("explains the queue blocker and reveals its request without cancelling", () => {
    const { actions, show } = setup({ status: "queued", queueBlocker: { kind: "confirmation", jobId: "other" } });
    expect(screen.getByText("extensionInstalls.queueBlocked: Grande extension")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "extensionInstalls.showRequest" }));
    expect(show).toHaveBeenCalledWith("other");
    expect(actions.cancel).not.toHaveBeenCalled();
    expect(screen.queryByRole("progressbar")).toBeNull();
  });
  it("sends only the current volume confirmation and keeps cancellation available", () => {
    const { actions } = setup({ status: "awaitingConfirmation", confirmationId: "current" });
    fireEvent.click(screen.getByRole("button", { name: "extensionInstalls.continue" }));
    expect(actions.continue).toHaveBeenCalledWith(base.id, "current");
    expect(screen.getByRole("button", { name: "extensionInstalls.cancel" })).toBeEnabled();
    expect(screen.queryByRole("progressbar")).toBeNull();
  });
  it.each([true, false])("offers only safe recovery for canResume=%s", (canResume) => {
    const { actions } = setup({ status: "interrupted", canCancel: false, canResume });
    const action = canResume ? "resume" : "retry";
    fireEvent.click(screen.getByRole("button", { name: `extensionInstalls.${action}` }));
    expect(actions[action]).toHaveBeenCalledWith(base.id);
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(screen.queryByRole("button", { name: "extensionInstalls.continue" })).toBeNull();
  });
  it("shows a revalidation error without losing the interrupted result", () => {
    setup({ status: "interrupted", canCancel: false, canResume: true }, "extensionInstalls.errors.unavailable");
    expect(screen.getByRole("alert")).toHaveTextContent("extensionInstalls.errors.unavailable");
    expect(screen.getByRole("button", { name: "extensionInstalls.resume" })).toBeEnabled();
  });
  it("opens a completed extension only on explicit click", () => {
    const { actions } = setup({ status: "completed", extensionId: "fixture", canCancel: false });
    expect(actions.open).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "extensionInstalls.open" }));
    expect(actions.open).toHaveBeenCalledWith("fixture");
  });
});
