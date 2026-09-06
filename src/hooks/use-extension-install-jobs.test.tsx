import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen, type EventCallback } from "@tauri-apps/api/event";
import { ExtensionInstallJobsProvider, useExtensionInstallJobs } from "./use-extension-install-jobs";
import { installJobFixture, installSnapshotFixture } from "@/lib/extension-install-job-fixture.test-support";

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
let event: EventCallback<unknown>;
const unlisten = vi.fn();
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>(done => { resolve = done; });
  return { promise, resolve };
}
function send(payload: unknown) { event({ event: "extension-installs-changed", id: 1, payload }); }
const wrapper = ExtensionInstallJobsProvider;

beforeEach(() => {
  vi.mocked(invoke).mockReset().mockResolvedValue(installSnapshotFixture(0, []));
  unlisten.mockReset();
  vi.mocked(listen).mockReset().mockImplementation((_name, callback) => {
    event = callback;
    return Promise.resolve(unlisten);
  });
});
afterEach(cleanup);

describe("backend installation projection", () => {
  it("subscribes before listing and ignores older snapshots and duplicate events", async () => {
    const subscription = deferred<() => void>();
    vi.mocked(listen).mockImplementation((_name, callback) => { event = callback; return subscription.promise; });
    const initial = deferred<unknown>();
    vi.mocked(invoke).mockReturnValue(initial.promise);
    const view = renderHook(useExtensionInstallJobs, { wrapper });
    expect(invoke).not.toHaveBeenCalled();
    expect(view.result.current.loading).toBe(true);
    await act(async () => { subscription.resolve(unlisten); await subscription.promise; });
    expect(invoke).toHaveBeenCalledWith("list_extension_installs");
    act(() => send(installSnapshotFixture(3)));
    expect(view.result.current.loading).toBe(false);
    await act(async () => { initial.resolve(installSnapshotFixture(1, [])); await initial.promise; });
    act(() => send(installSnapshotFixture(3, [])));
    expect(view.result.current.jobs).toHaveLength(1);
    expect(view.result.current.jobs[0].revision).toBe(3);
    act(() => send(installSnapshotFixture(4, [])));
    expect(view.result.current.jobs).toHaveLength(0);
  });
  it("preserves visible jobs on load errors and rejects malformed events", async () => {
    const view = renderHook(useExtensionInstallJobs, { wrapper });
    await waitFor(() => expect(invoke).toHaveBeenCalled());
    act(() => send(installSnapshotFixture(2)));
    vi.mocked(invoke).mockRejectedValue(new Error("private/path"));
    await act(() => view.result.current.refresh());
    expect(view.result.current.loadError).toBe("extensionInstalls.errors.load");
    expect(view.result.current.loading).toBe(false);
    expect(view.result.current.jobs).toHaveLength(1);
    act(() => send({ revision: 3, jobs: [{ source: "private" }] }));
    expect(view.result.current.jobs[0].revision).toBe(2);
  });
  it("returns admission while listing is delayed and prevents simultaneous double clicks", async () => {
    const admission = deferred<unknown>();
    const listing = deferred<unknown>();
    vi.mocked(invoke).mockImplementation(command => command === "start_extension_install" ? admission.promise : listing.promise);
    const view = renderHook(useExtensionInstallJobs, { wrapper });
    let first!: ReturnType<typeof view.result.current.start>;
    act(() => { first = view.result.current.start({ kind: "npm", locator: "fixture" }); });
    const second = await act(() => view.result.current.start({ kind: "npm", locator: "fixture" }));
    expect(second.errorKey).toBe("extensionInstalls.errors.busy");
    await act(async () => { admission.resolve(installJobFixture()); await admission.promise; });
    const result = await first;
    expect(result.job?.id).toBe(installJobFixture().id);
    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "start_extension_install")).toHaveLength(1);
    await act(async () => { listing.resolve(installSnapshotFixture()); await listing.promise; });
  });
  it("keeps an interrupted job visible when resumption is refused", async () => {
    const job = installJobFixture({ status: "interrupted", canCancel: false, canResume: true });
    vi.mocked(invoke).mockImplementation(command => command === "resume_extension_install"
      ? Promise.reject(new Error("extension-install-unavailable")) : Promise.resolve(installSnapshotFixture(1, [job])));
    const view = renderHook(useExtensionInstallJobs, { wrapper });
    await waitFor(() => expect(view.result.current.jobs).toHaveLength(1));
    await act(() => view.result.current.action("resume_extension_install", job.id));
    expect(view.result.current.jobs[0].status).toBe("interrupted");
    expect(view.result.current.errors[job.id]).toBe("extensionInstalls.errors.unavailable");
    expect(view.result.current.busyIds.size).toBe(0);
  });
  it("releases a subscription that finishes after unmount", async () => {
    const subscription = deferred<() => void>();
    vi.mocked(listen).mockReturnValue(subscription.promise);
    const view = renderHook(useExtensionInstallJobs, { wrapper });
    view.unmount();
    await act(async () => { subscription.resolve(unlisten); await subscription.promise; });
    expect(unlisten).toHaveBeenCalledOnce();
    expect(invoke).not.toHaveBeenCalled();
  });
});
