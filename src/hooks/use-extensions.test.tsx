import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ExtensionHostStatus, ExtensionRecord } from "@/types/extensions";
import { ExtensionsProvider, useExtensions } from "./use-extensions";

const record: ExtensionRecord = {
  manifest: {
    id: "com.example.test",
    name: "Test",
    version: "1.0.0",
    beaverApi: "1",
    runtime: "node",
    access: "full",
    apiLevel: "stable",
    essential: false,
  },
  kind: "local",
  source: "/extension",
  enabled: false,
  trusted: true,
  showInChat: false,
  status: "inactive",
  contributions: { tools: [], events: [] },
};

const host: ExtensionHostStatus = {
  state: "running",
  nodeVersion: "v24.18.0",
  jitiVersion: "2.7.0",
  apiVersion: "1",
  activeExtensions: 0,
  diagnostics: [],
};

describe("useExtensions", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      return Promise.resolve(undefined);
    });
  });

  it("utilise le registre Rust comme unique source de vérité", async () => {
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    expect(view.result.current.extensions).toEqual([record]);
    expect(view.result.current.host).toEqual(host);
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

  it("installe Git et npm avec des commandes distinctes et validées au retour", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "install_git_extension" || command === "install_npm_extension") {
        return Promise.resolve(record);
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    const gitOutcome = await act(() =>
      view.result.current.installGit("https://github.com/example/ext.git"));
    expect(gitOutcome.record).toEqual(record);
    expect(invoke).toHaveBeenCalledWith("install_git_extension", {
      url: "https://github.com/example/ext.git",
    });

    await act(() => view.result.current.installNpm("@example/ext@latest"));
    expect(invoke).toHaveBeenCalledWith("install_npm_extension", {
      packageSpec: "@example/ext@latest",
    });
  });

  it("retourne le code traduit exact lorsqu’une installation échoue", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "list_extensions") return Promise.resolve([record]);
      if (command === "get_extension_host_status") return Promise.resolve(host);
      if (command === "get_extension_discovery_preferences") {
        return Promise.resolve({ protectedPluginIds: [] });
      }
      if (command === "install_git_extension") {
        return Promise.reject(new Error("extensions_git_download_failed"));
      }
      return Promise.resolve(undefined);
    });
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    const outcome = await act(() =>
      view.result.current.installGit("https://github.com/example/ext.git"));

    expect(outcome).toEqual({
      record: null,
      errorKey: "extensions.errors.codes.extensions_git_download_failed",
    });
  });

  it("marque une extension occupée pendant sa mise à jour", async () => {
    const view = renderHook(() => useExtensions(), { wrapper: ExtensionsProvider });
    await waitFor(() => expect(view.result.current.loading).toBe(false));

    await act(() => view.result.current.update(record.manifest.id));

    expect(invoke).toHaveBeenCalledWith("update_extension", {
      extensionId: record.manifest.id,
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
