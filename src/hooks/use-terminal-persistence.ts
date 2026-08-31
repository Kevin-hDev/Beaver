import { useEffect, useRef, useState } from "react";
import type { Dispatch, SetStateAction } from "react";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";
import { loadSavedGroups, saveGroups } from "./terminal-persistence";
import type { TerminalTabsDocument } from "./terminal-persistence";
import { TerminalPersistenceQueue } from "./terminal-persistence-queue";
import { generateId } from "./terminal-types";
import type { TerminalGroup } from "./terminal-types";

export type TerminalPersistenceStatus = "loading" | "healthy" | "error";

interface UseTerminalPersistenceOptions {
  groups: Map<string, TerminalGroup>;
  setGroups: Dispatch<SetStateAction<Map<string, TerminalGroup>>>;
  canSave: boolean;
}

function restoreGroups(document: TerminalTabsDocument): Map<string, TerminalGroup> {
  const groups = new Map<string, TerminalGroup>();
  for (const [groupKey, savedTabs] of Object.entries(document.groups)) {
    const tabs = savedTabs.map(({ label }) => ({
      id: generateId(),
      ptyId: null,
      ptyToken: null,
      label,
      hasActivity: false,
    }));
    groups.set(groupKey, { tabs, activeTabId: tabs[0]?.id ?? null });
  }
  return groups;
}

function projectGroups(groups: Map<string, TerminalGroup>): TerminalTabsDocument {
  const durableGroups: TerminalTabsDocument["groups"] = {};
  for (const [groupKey, group] of groups) {
    if (group.tabs.length > 0) {
      durableGroups[groupKey] = group.tabs.map(({ label }) => ({ label }));
    }
  }
  return { version: 1, groups: durableGroups };
}

export function useTerminalPersistence({
  groups,
  setGroups,
  canSave,
}: UseTerminalPersistenceOptions) {
  const [loaded, setLoaded] = useState(false);
  const [persistenceStatus, setPersistenceStatus] =
    useState<TerminalPersistenceStatus>("loading");
  const queueRef = useRef<TerminalPersistenceQueue | null>(null);
  const monitoringRef = useRef(false);
  const lastProjectionRef = useRef<string | null>(null);
  const loadStartedRef = useRef(false);
  if (queueRef.current === null) queueRef.current = new TerminalPersistenceQueue(saveGroups);

  useEffect(() => {
    if (loadStartedRef.current) return;
    loadStartedRef.current = true;
    void loadSavedGroups().then((document) => {
      lastProjectionRef.current = JSON.stringify(document);
      setGroups(restoreGroups(document));
      setLoaded(true);
      setPersistenceStatus("healthy");
    }).catch(() => {
      setPersistenceStatus("error");
      showToast(i18n.t("terminal.tabsLoadFailed"), "error");
    });
  }, [setGroups]);

  useEffect(() => {
    if (!loaded || !canSave || persistenceStatus !== "healthy") return;
    const document = projectGroups(groups);
    const projection = JSON.stringify(document);
    if (projection === lastProjectionRef.current) return;
    lastProjectionRef.current = projection;
    const queue = queueRef.current;
    if (queue === null) return;
    queue.enqueue(document);
    if (monitoringRef.current) return;
    monitoringRef.current = true;
    void queue.idle().then(() => {
      monitoringRef.current = false;
    }).catch(() => {
      monitoringRef.current = false;
      setPersistenceStatus("error");
      showToast(i18n.t("terminal.tabsSaveFailed"), "error");
    });
  }, [canSave, groups, loaded, persistenceStatus]);

  return { loaded, persistenceStatus };
}
