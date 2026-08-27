import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useUpdateChecker } from "@/hooks/use-update-checker";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  getVersion: vi.fn(),
  listen: vi.fn(),
  showToast: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mocks.invoke,
  Channel: class<T> { onmessage?: (message: T) => void },
}));
vi.mock("@tauri-apps/api/app", () => ({ getVersion: mocks.getVersion }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("@/hooks/use-model-downloads", () => ({
  useModelDownloads: () => ({
    activeDownload: null,
    startDownload: vi.fn(),
    cancelDownload: vi.fn(),
  }),
}));
vi.mock("@/hooks/use-forecast-dev-updates", () => ({
  useForecastDevUpdates: () => ({ forecastDevUpdates: [] }),
}));
vi.mock("@/hooks/use-update-dismissals", () => ({
  useUpdateDismissals: () => ({
    dismiss: vi.fn(),
    visible: <T,>(value: T | null) => value,
    filter: <T,>(values: T[]) => values,
  }),
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast: mocks.showToast }));

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

describe("useUpdateChecker", () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
    mocks.getVersion.mockReset().mockResolvedValue("1.1.7");
    mocks.listen.mockReset().mockResolvedValue(() => {});
    mocks.showToast.mockReset();
  });

  it("coalesces overlapping update checks", async () => {
    const appCheck = deferred<null>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "check_app_update") return appCheck.promise;
      if (command === "check_ollama_updates") return Promise.resolve([]);
      if (command === "check_ollama_binary_update") return Promise.resolve(null);
      if (command === "get_ollama_installed_version") return Promise.resolve("0.32.15");
      return Promise.resolve(undefined);
    });

    const view = renderHook(() => useUpdateChecker());
    act(() => { void view.result.current.checkAll(); });

    await waitFor(() => {
      expect(mocks.invoke.mock.calls.filter(([command]) => command === "check_app_update"))
        .toHaveLength(1);
    });
    appCheck.resolve(null);
    await waitFor(() => expect(view.result.current.checking).toBe(false));
  });

  it("keeps the discovered update visible while its download is active", async () => {
    const updateDownload = deferred<void>();
    let binaryChecks = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "check_app_update") return Promise.resolve(null);
      if (command === "check_ollama_updates") return Promise.resolve([]);
      if (command === "check_ollama_binary_update") {
        binaryChecks += 1;
        return Promise.resolve(binaryChecks === 1
          ? { currentVersion: "0.32.15", latestVersion: "0.33.1" }
          : null);
      }
      if (command === "get_ollama_installed_version") return Promise.resolve("0.32.15");
      if (command === "update_ollama_binary") return updateDownload.promise;
      return Promise.resolve(undefined);
    });

    const view = renderHook(() => useUpdateChecker());
    await waitFor(() => expect(view.result.current.ollamaBinaryUpdate?.latestVersion).toBe("0.33.1"));

    let updateTask!: Promise<void>;
    act(() => { updateTask = view.result.current.updateOllamaBinary(); });
    await waitFor(() => expect(view.result.current.ollamaBinaryUpdating).toBe(true));
    await act(async () => { await view.result.current.checkAll(); });

    expect(view.result.current.ollamaBinaryUpdate?.latestVersion).toBe("0.33.1");
    expect(view.result.current.ollamaBinaryUpdating).toBe(true);

    updateDownload.reject("ollama-download-failed");
    await act(async () => { await updateTask; });
  });

  it("reports a failed manual check without forgetting the known update", async () => {
    let binaryChecks = 0;
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "check_app_update") return Promise.resolve(null);
      if (command === "check_ollama_updates") return Promise.resolve([]);
      if (command === "check_ollama_binary_update") {
        binaryChecks += 1;
        return binaryChecks === 1
          ? Promise.resolve({ currentVersion: "0.32.15", latestVersion: "0.33.1" })
          : Promise.reject(new Error("ollama-update-check-failed"));
      }
      if (command === "get_ollama_installed_version") return Promise.resolve("0.32.15");
      return Promise.resolve(undefined);
    });

    const view = renderHook(() => useUpdateChecker());
    await waitFor(() => expect(view.result.current.ollamaBinaryUpdate?.latestVersion).toBe("0.33.1"));

    await act(async () => { await view.result.current.checkAll(true); });

    expect(view.result.current.ollamaBinaryUpdate?.latestVersion).toBe("0.33.1");
    expect(mocks.showToast).toHaveBeenCalledWith(expect.any(String), "error");
  });

  it("clears cancellation indicators when the backend reports that work already ended", async () => {
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "check_app_update") return Promise.resolve(null);
      if (command === "check_ollama_updates") return Promise.resolve([]);
      if (command === "check_ollama_binary_update") return Promise.resolve(null);
      if (command === "get_ollama_installed_version") return Promise.resolve("0.32.15");
      if (command === "cancel_app_update_download") return Promise.resolve();
      if (command === "cancel_ollama_setup") return Promise.resolve();
      return Promise.resolve(undefined);
    });

    const view = renderHook(() => useUpdateChecker());
    await waitFor(() => expect(view.result.current.checking).toBe(false));

    await act(async () => { await view.result.current.cancelAppUpdate(); });
    await act(async () => { await view.result.current.cancelOllamaBinary(); });

    expect(view.result.current.appCancelling).toBe(false);
    expect(view.result.current.ollamaBinaryCancelling).toBe(false);
  });

  it("keeps the cancellation indicator until the active download has actually stopped", async () => {
    const download = deferred<void>();
    mocks.invoke.mockImplementation((command: string) => {
      if (command === "check_app_update") {
        return Promise.resolve({ version: "1.1.8", assetUrl: "https://example.test/update" });
      }
      if (command === "check_ollama_updates") return Promise.resolve([]);
      if (command === "check_ollama_binary_update") return Promise.resolve(null);
      if (command === "get_ollama_installed_version") return Promise.resolve("0.32.15");
      if (command === "download_app_update") return download.promise;
      if (command === "cancel_app_update_download") return Promise.resolve();
      return Promise.resolve(undefined);
    });

    const view = renderHook(() => useUpdateChecker());
    await waitFor(() => expect(view.result.current.appUpdate?.version).toBe("1.1.8"));

    let downloadTask!: Promise<void>;
    act(() => { downloadTask = view.result.current.downloadAppUpdate("https://example.test/update"); });
    await waitFor(() => expect(view.result.current.appDownloading).toBe(true));
    await act(async () => { await view.result.current.cancelAppUpdate(); });

    expect(view.result.current.appCancelling).toBe(true);

    download.reject("update-download-cancelled");
    await act(async () => { await downloadTask; });
    expect(view.result.current.appCancelling).toBe(false);
  });
});
