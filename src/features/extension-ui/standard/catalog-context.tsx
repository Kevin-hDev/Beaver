import { invoke } from "@tauri-apps/api/core";
import { createContext, useCallback, useContext, useEffect, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { showToast } from "@/lib/toast-emitter";
import { useExtensionUiStartupContext } from "@/hooks/use-extension-ui-startup";
import type { ExtensionUiActionPayload, ExtensionUiStartupMode } from "@/types/extensions";
import { parseStandardActionResult } from "./action-result-parser";
import { createMountCoordinator, type MountPermit } from "./mount-coordinator";
import { useCatalogSync, type CatalogSyncState } from "./use-catalog-sync";
import type {
  StandardActionResult,
  StandardCatalogEntry,
  StandardCatalogSnapshot,
} from "./types";

interface StandardCatalogContextValue {
  state: CatalogSyncState;
  snapshot: StandardCatalogSnapshot | null;
  entry: (extensionId: string, contributionId: string) => StandardCatalogEntry | undefined;
  prepareMount: (entry: StandardCatalogEntry) => Promise<MountPermit>;
  invokeAction: (
    entry: StandardCatalogEntry,
    actionId: string,
    payload: ExtensionUiActionPayload,
  ) => Promise<StandardActionResult>;
  openExtension: (extensionId: string) => void;
  reportMountFailure: (entry: StandardCatalogEntry) => void;
}

const StandardCatalogContext = createContext<StandardCatalogContextValue | null>(null);

export function StandardCatalogProvider({
  children,
  onOpenExtension,
}: {
  children: React.ReactNode;
  onOpenExtension: (extensionId: string) => void;
}) {
  const state = useCatalogSync();
  const { i18n, t } = useTranslation();
  const startup = useExtensionUiStartupContext();
  const startupMode = startup?.state.mode;
  const refreshStartup = startup?.refresh;
  const coordinator = useMemo(() => createMountCoordinator(), []);
  const snapshot = useMemo(
    () => filterSnapshot(state.snapshot, startupMode),
    [startupMode, state.snapshot],
  );
  const byKey = useMemo(() => new Map(
    snapshot?.contributions.map((entry) => [keyOf(entry.extensionId, entry.contributionId), entry]),
  ), [snapshot]);
  const previousError = useRef(false);

  useEffect(() => {
    const failed = state.kind === "error" || state.kind === "stale-error";
    if (failed && !previousError.current) showToast(t("extensions.ui.catalogError"), "error");
    previousError.current = failed;
  }, [state.kind, t]);

  const prepareMount = useCallback(async (entry: StandardCatalogEntry) => {
    if (entry.extensionId.startsWith("beaver.")) {
      return { commit: async () => {}, cancel: () => {} };
    }
    const attempts = retryAttempts(startupMode, entry.extensionId);
    const permit = await coordinator.prepare(
      `${snapshot?.revision ?? 0}:${keyOf(entry.extensionId, entry.contributionId)}`,
      entry.extensionId,
      attempts,
    );
    return {
      cancel: permit.cancel,
      commit: async () => {
        await permit.commit();
        await refreshStartup?.();
      },
    };
  }, [coordinator, refreshStartup, snapshot?.revision, startupMode]);

  const invokeAction = useCallback(async (
    entry: StandardCatalogEntry,
    actionId: string,
    payload: ExtensionUiActionPayload,
  ) => parseStandardActionResult(entry.extensionId, await invoke<unknown>(
    "invoke_extension_ui_action",
    {
      extensionId: entry.extensionId,
      contributionId: entry.contributionId,
      actionId,
      payload,
      locale: activeLocale(i18n.resolvedLanguage ?? i18n.language),
    },
  )), [i18n.language, i18n.resolvedLanguage]);

  const reportMountFailure = useCallback((entry: StandardCatalogEntry) => {
    void invoke("report_extension_ui_mount_failure", {
      extensionId: entry.extensionId,
      contributionId: entry.contributionId,
    }).catch(() => {});
  }, []);

  const value = useMemo<StandardCatalogContextValue>(() => ({
    state,
    snapshot,
    entry: (extensionId, contributionId) => byKey.get(keyOf(extensionId, contributionId)),
    prepareMount,
    invokeAction,
    openExtension: onOpenExtension,
    reportMountFailure,
  }), [byKey, invokeAction, onOpenExtension, prepareMount, reportMountFailure, snapshot, state]);
  return <StandardCatalogContext.Provider value={value}>{children}</StandardCatalogContext.Provider>;
}

export function useStandardCatalog(): StandardCatalogContextValue {
  const value = useContext(StandardCatalogContext);
  if (!value) throw new Error("Standard UI consumers require StandardCatalogProvider.");
  return value;
}

export function useOptionalStandardCatalog(): StandardCatalogContextValue | null {
  return useContext(StandardCatalogContext);
}

function filterSnapshot(
  snapshot: StandardCatalogSnapshot | null,
  mode: ExtensionUiStartupMode | undefined,
): StandardCatalogSnapshot | null {
  if (!snapshot) return null;
  const retryId = mode?.kind === "retryInterruptedUi" ? mode.extensionId : null;
  const allowAll = !mode || mode.kind === "normal";
  return {
    revision: snapshot.revision,
    contributions: snapshot.contributions.filter(({ extensionId }) =>
      extensionId.startsWith("beaver.") || allowAll || extensionId === retryId),
  };
}

function retryAttempts(
  mode: ExtensionUiStartupMode | undefined,
  extensionId: string,
): number {
  return mode?.kind === "retryInterruptedUi" && mode.extensionId === extensionId
    ? mode.attempts
    : 1;
}

function activeLocale(language: string): string {
  const locale = language.toLowerCase().split("-")[0];
  return ["fr", "en", "es", "de", "it", "zh", "ja"].includes(locale) ? locale : "en";
}

function keyOf(extensionId: string, contributionId: string): string {
  return `${extensionId}\u0000${contributionId}`;
}
