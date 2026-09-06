import { installJobFixture } from "@/lib/extension-install-job-fixture.test-support";
import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  ExtensionHostStatus,
  ExtensionRecord,
  ExtensionRecoveryState,
} from "@/types/extensions";
import { ExtensionsProvider, useExtensions } from "./use-extensions";

const record: ExtensionRecord = {
  manifest: {
    id: "com.example.test",
    name: "Test",
    version: "1.0.0",
    beaverApi: "1",
    runtime: "node",
    main: undefined,
    ui: undefined,
    uiLegacy: undefined,
    access: "full",
    apiLevel: "stable",
    essential: false,
    author: undefined,
    homepage: undefined,
    description: undefined,
  },
  kind: "local",
  source: "/extension",
  origin: undefined,
  enabled: false,
  trusted: true,
  trustedAt: undefined,
  uiArtifact: undefined,
  showInChat: false,
  status: "inactive",
  lastError: undefined,
  lastActivatedAt: undefined,
  contributions: { tools: [], events: [], skills: [], resources: [] },
};

const host: ExtensionHostStatus = {
  state: "running",
  nodeVersion: "v24.18.0",
  jitiVersion: "2.7.0",
  apiVersion: "1",
  activeExtensions: 0,
  diagnostics: [],
};

const emptyRecovery: ExtensionRecoveryState = {
  extensionId: null,
  stage: null,
  attempts: null,
  canRetry: false,
  markerInvalid: false,
  recoverySnapshotAvailable: false,
};

describe("useExtensions", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") {
        return Promise.resolve(emptyRecovery);
      }
      return Promise.resolve(undefined);
    });
  });

  it("utilise le registre Rust comme unique source de vérité", async () => {
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    expect(view.result.current.extensions).toEqual([record]);
    expect(view.result.current.host).toEqual(host);
    expect(view.result.current.hostLoaded).toBe(true);
  });

  it("garde enabled et showInChat comme deux mutations indépendantes", async () => {
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    await act(() => view.result.current.setEnabled(record.manifest.id, true));
    expect(invoke).toHaveBeenCalledWith("set_extension_enabled", {
      extensionId: record.manifest.id,
      enabled: true,
      trustConfirmed: false,
    });

    await act(() => view.result.current.setShowInChat(record.manifest.id, true));
    expect(invoke).toHaveBeenCalledWith("set_extension_show_in_chat", {
      extensionId: record.manifest.id,
      showInChat: true,
    });
  });

  it("garde la confirmation ouverte lorsque l'activation est refusée", async () => {
    const untrusted = { ...record, trusted: false };
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([untrusted]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "set_extension_enabled") {
        return Promise.reject(new Error("extensions_fingerprint_changed"));
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));
    await act(() => view.result.current.setEnabled(record.manifest.id, true));

    await act(() => view.result.current.confirmActivation());

    expect(view.result.current.pendingActivation).toEqual(untrusted);
    expect(view.result.current.operationError)
      .toBe("extensions.errors.codes.extensions_fingerprint_changed");
    expect(view.result.current.busyIds.has(record.manifest.id)).toBe(false);
  });

  it("n'affiche pas une ancienne erreur dans une nouvelle confirmation", async () => {
    const untrusted = { ...record, trusted: false };
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([untrusted]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") {
        return Promise.resolve(emptyRecovery);
      }
      if (command === "remove_extension") {
        return Promise.reject(new Error("extensions_operation_failed"));
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));
    await act(() => view.result.current.remove(record.manifest.id));
    expect(view.result.current.operationError)
      .toBe("extensions.errors.codes.extensions_operation_failed");

    await act(() => view.result.current.setEnabled(record.manifest.id, true));

    expect(view.result.current.pendingActivation).toEqual(untrusted);
    expect(view.result.current.operationError).toBeNull();
  });

  it("affiche le rappel traduit après révocation d’un accès sensible", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "set_extension_enabled") return Promise.resolve(true);
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    await act(() => view.result.current.setEnabled(record.manifest.id, false));

    expect(view.result.current.operationError)
      .toBe("extensions.sensitiveAccessReminder");
  });

  it("expose une erreur traduisible sans afficher le détail Rust", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") {
        return Promise.reject(new Error("extensions_builtin_catalog_invalid"));
      }
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });

    await waitFor(() => expect(view.result.current.loading).toBe(false));
    expect(view.result.current.loadError)
      .toBe("extensions.errors.codes.extensions_builtin_catalog_invalid");
  });

  it("conserve le contenu précédent lorsqu'un rafraîchissement échoue", async () => {
    let refreshFails = false;
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") {
        return refreshFails
          ? Promise.reject(new Error("extensions_operation_failed"))
          : Promise.resolve([record]);
      }
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") {
        return Promise.resolve({
          extensionId: null,
          stage: null,
          attempts: null,
          canRetry: false,
          markerInvalid: false,
          recoverySnapshotAvailable: false,
        });
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));
    refreshFails = true;

    await act(() => view.result.current.refresh());

    expect(view.result.current.extensions).toEqual([record]);
    expect(view.result.current.host).toEqual(host);
    expect(view.result.current.loadError)
      .toBe("extensions.errors.codes.extensions_operation_failed");
  });

  it("ignore une réponse de rafraîchissement devenue obsolète", async () => {
    const latest = {
      ...record,
      manifest: { ...record.manifest, name: "Latest" },
    };
    let listCalls = 0;
    let resolveOld!: (value: ExtensionRecord[]) => void;
    let resolveLatest!: (value: ExtensionRecord[]) => void;
    const oldResponse = new Promise<ExtensionRecord[]>((resolve) => {
      resolveOld = resolve;
    });
    const latestResponse = new Promise<ExtensionRecord[]>((resolve) => {
      resolveLatest = resolve;
    });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") {
        listCalls += 1;
        if (listCalls === 1) return Promise.resolve([record]);
        return listCalls === 2 ? oldResponse : latestResponse;
      }
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") {
        return Promise.resolve(emptyRecovery);
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    let oldRefresh!: Promise<void>;
    let latestRefresh!: Promise<void>;
    act(() => { oldRefresh = view.result.current.refresh(); });
    act(() => { latestRefresh = view.result.current.refresh(); });
    resolveLatest([latest]);
    await act(() => latestRefresh);
    expect(view.result.current.extensions[0].manifest.name).toBe("Latest");
    resolveOld([record]);
    await act(() => oldRefresh);

    expect(view.result.current.extensions[0].manifest.name).toBe("Latest");
    expect(view.result.current.loading).toBe(false);
  });

  it("retire l'ancienne erreur dès qu'un nouveau rafraîchissement commence", async () => {
    let mode: "success" | "failure" | "pending" = "success";
    let resolvePending!: (value: ExtensionRecord[]) => void;
    const pending = new Promise<ExtensionRecord[]>((resolve) => {
      resolvePending = resolve;
    });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") {
        if (mode === "failure") {
          return Promise.reject(new Error("extensions_operation_failed"));
        }
        if (mode === "pending") return pending;
        return Promise.resolve([record]);
      }
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") {
        return Promise.resolve(emptyRecovery);
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));
    mode = "failure";
    await act(() => view.result.current.refresh());
    expect(view.result.current.loadError)
      .toBe("extensions.errors.codes.extensions_operation_failed");

    mode = "pending";
    let retry!: Promise<void>;
    act(() => { retry = view.result.current.refresh(); });

    expect(view.result.current.loading).toBe(true);
    expect(view.result.current.loadError).toBeNull();
    resolvePending([record]);
    await act(() => retry);
  });

  it("conserve la reprise et expose une erreur si son rafraîchissement échoue", async () => {
    let recoveryFails = false;
    const interrupted = {
      ...emptyRecovery,
      extensionId: record.manifest.id,
      stage: "activate",
      attempts: 1,
      canRetry: true,
    };
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") {
        return recoveryFails
          ? Promise.reject(new Error("extensions_operation_failed"))
          : Promise.resolve(interrupted);
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.recovery.extensionId)
      .toBe(record.manifest.id));
    recoveryFails = true;

    await act(() => view.result.current.refreshRecovery());

    expect(view.result.current.recovery.extensionId).toBe(record.manifest.id);
    expect(view.result.current.operationError)
      .toBe("extensions.errors.codes.extensions_operation_failed");
  });

  it("ignore un état de reprise devenu obsolète", async () => {
    let recoveryCalls = 0;
    let resolveOld!: (value: typeof emptyRecovery) => void;
    let resolveLatest!: (value: typeof emptyRecovery) => void;
    const oldResponse = new Promise<typeof emptyRecovery>((resolve) => {
      resolveOld = resolve;
    });
    const latestResponse = new Promise<typeof emptyRecovery>((resolve) => {
      resolveLatest = resolve;
    });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") {
        recoveryCalls += 1;
        if (recoveryCalls === 1) return Promise.resolve(emptyRecovery);
        return recoveryCalls === 2 ? oldResponse : latestResponse;
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    let oldRefresh!: Promise<void>;
    let latestRefresh!: Promise<void>;
    act(() => { oldRefresh = view.result.current.refreshRecovery(); });
    act(() => { latestRefresh = view.result.current.refreshRecovery(); });
    resolveLatest(emptyRecovery);
    await act(() => latestRefresh);
    resolveOld({
      ...emptyRecovery,
      extensionId: record.manifest.id,
      stage: "activate",
      attempts: 1,
      canRetry: true,
    });
    await act(() => oldRefresh);

    expect(view.result.current.recovery).toEqual(emptyRecovery);
  });

  it("verrouille les actions Hôte et câble les cinq commandes de reprise", async () => {
    let finishReload!: () => void;
    const reload = new Promise<void>((resolve) => { finishReload = resolve; });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") {
        return Promise.resolve({
          extensionId: record.manifest.id,
          stage: "activate",
          attempts: 1,
          canRetry: true,
          markerInvalid: false,
          recoverySnapshotAvailable: true,
        });
      }
      if (command === "reload_extension_host") return reload;
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    act(() => { void view.result.current.reload(); });
    await waitFor(() => expect(view.result.current.hostBusy).toBe(true));
    finishReload();
    await waitFor(() => expect(view.result.current.hostBusy).toBe(false));

    await act(() => view.result.current.keepDisabled(record.manifest.id));
    await act(() => view.result.current.retryLoad(record.manifest.id));
    await act(() => view.result.current.discardLoadingMarker());
    await act(() => view.result.current.restoreRecoverySnapshot());
    expect(invoke).toHaveBeenCalledWith("keep_extension_disabled", {
      extensionId: record.manifest.id,
    });
    expect(invoke).toHaveBeenCalledWith("retry_extension_load", {
      extensionId: record.manifest.id,
    });
    expect(invoke).toHaveBeenCalledWith("discard_extension_loading_marker", {});
    expect(invoke).toHaveBeenCalledWith("restore_extension_recovery_snapshot", {});
    expect(view.result.current.recovery.extensionId).toBe(record.manifest.id);
  });

  it("refuse une réponse IPC incomplète avant de l’exposer à React", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") {
        return Promise.resolve([{ ...record, contributions: undefined }]);
      }
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });

    await waitFor(() => expect(view.result.current.loading).toBe(false));
    expect(view.result.current.extensions).toEqual([]);
    expect(view.result.current.host.state).toBe("stopped");
    expect(view.result.current.loadError).toBe("extensions.errors.load");
  });

  it("refuse un statut Hôte inconnu et conserve le dernier statut validé", async () => {
    let invalidHost = false;
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") {
        return Promise.resolve(invalidHost ? { ...host, state: "unknown" } : host);
      }
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") return Promise.resolve(emptyRecovery);
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.hostLoaded).toBe(true));
    invalidHost = true;

    await act(() => view.result.current.refresh());

    expect(view.result.current.host).toEqual(host);
    expect(view.result.current.loadError).toBe("extensions.errors.load");
  });

  it("sérialise les actions de reprise déclenchées simultanément", async () => {
    let finishKeep!: () => void;
    const keep = new Promise<void>((resolve) => { finishKeep = resolve; });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "get_extension_recovery_state") return Promise.resolve(emptyRecovery);
      if (command === "keep_extension_disabled") return keep;
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    act(() => { void view.result.current.keepDisabled(record.manifest.id); });
    await waitFor(() => expect(view.result.current.recoveryBusy).toBe(true));
    await act(() => view.result.current.retryLoad(record.manifest.id));

    expect(invoke).not.toHaveBeenCalledWith("retry_extension_load", {
      extensionId: record.manifest.id,
    });
    finishKeep();
    await waitFor(() => expect(view.result.current.recoveryBusy).toBe(false));
  });

  it("admet Git et npm dans la même autorité avec des sources distinctes", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "start_extension_install") {
        return Promise.resolve(installJobFixture());
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    const gitOutcome = await act(() =>
      view.result.current.installGit("https://github.com/example/ext.git"));
    expect(gitOutcome.jobId).toEqual(installJobFixture().id);
    expect(invoke).toHaveBeenCalledWith("start_extension_install", {
      request: { kind: "git", locator: "https://github.com/example/ext.git" },
    });

    await act(() => view.result.current.installNpm("@example/ext@latest"));
    expect(invoke).toHaveBeenCalledWith("start_extension_install", {
      request: { kind: "npm", locator: "@example/ext@latest" },
    });
  });

  it("retourne le code traduit exact lorsqu’une installation échoue", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "start_extension_install") {
        return Promise.reject(new Error("extensions_git_download_failed"));
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    const outcome = await act(() =>
      view.result.current.installGit("https://github.com/example/ext.git"));

    expect(outcome).toEqual({
      jobId: null,
      errorKey: "extensions.errors.codes.extensions_git_download_failed",
    });
  });

  it("admet une mise à jour dans le suivi commun", async () => {
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    await act(() => view.result.current.update(record.manifest.id));

    expect(invoke).toHaveBeenCalledWith("start_extension_install", {
      request: { kind: "update", extensionId: record.manifest.id },
    });
  });

  it("enregistre la liste bornée des plugins prioritaires", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "set_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [record.manifest.id] });
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    const saved = await act(() =>
      view.result.current.setPriorityPlugins([record.manifest.id]));

    expect(saved).toBe(true);
    expect(view.result.current.protectedPluginIds).toEqual([record.manifest.id]);
  });
});
