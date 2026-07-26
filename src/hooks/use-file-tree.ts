import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { FileEntry } from "@/lib/file-tree-filter";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { clampFileTreeStoredWidth } from "./file-tree-layout";
import { useFileTreeResize } from "./use-file-tree-resize";

const MAX_EXPANDED = 500;
const MAX_CACHED_DIRS = 600;

function treeStorageKey(sessionId: string | null): string {
  return `clgo-file-tree-width:${sessionId ?? "none"}`;
}

function readStoredWidth(sessionId: string | null): number {
  try {
    const raw = localStorage.getItem(treeStorageKey(sessionId));
    const parsed = Number(raw);
    return clampFileTreeStoredWidth(parsed);
  } catch { /* ignore */ }
  return clampFileTreeStoredWidth(undefined);
}

export function useFileTree(sessionId: string | null, projectPath: string | undefined) {
  const [open, setOpen] = useState(false);
  const [width, setWidth] = useState(() => readStoredWidth(sessionId));
  const { resizing, startResize } = useFileTreeResize(width, setWidth);

  const [rootEntries, setRootEntries] = useState<FileEntry[]>([]);
  const [childrenMap, setChildrenMap] = useState<Map<string, FileEntry[]>>(new Map());
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(new Set());
  const [filter, setFilter] = useState("");
  const [loadError, setLoadError] = useState<string | null>(null);

  const expandedRef = useRef(expandedPaths);
  useEffect(() => {
    expandedRef.current = expandedPaths;
  }, [expandedPaths]);

  const hasProject = !!projectPath;

  const loadDirectory = useCallback(async (dirPath: string): Promise<FileEntry[]> => {
    const entries = await invoke<FileEntry[]>("list_directory", {
      path: dirPath,
      showHidden: false,
      projectRoot: projectPath ?? null,
    });
    return entries;
  }, [projectPath]);

  useEffect(() => {
    if (!projectPath || !open) return;
    let alive = true;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- clears previous async load error before reloading the tree
    setLoadError(null);
    loadDirectory(projectPath)
      .then((entries) => { if (alive) setRootEntries(entries); })
      .catch(() => { if (alive) setLoadError("error"); });
    return () => { alive = false; };
  }, [projectPath, open, loadDirectory]);

  useEffect(() => {
    if (!projectPath || !open) return;
    invoke("watch_project_directory", { path: projectPath }).catch(() => {});
    return () => {
      invoke("unwatch_project_directory").catch(() => {});
    };
  }, [projectPath, open]);

  useEffect(() => {
    if (!open || !projectPath) return;
    let alive = true;
    const unlisten = listen<{ path: string }>("file-tree-changed", (event) => {
      if (!alive) return;
      const changedDir = event.payload.path;
      if (changedDir === projectPath) {
        loadDirectory(changedDir)
          .then((entries) => { if (alive) setRootEntries(entries); })
          .catch(() => {});
      } else if (expandedRef.current.has(changedDir)) {
        loadDirectory(changedDir).then((entries) => {
          if (!alive) return;
          setChildrenMap((prev) => {
            const next = new Map(prev);
            next.set(changedDir, entries);
            return next;
          });
        }).catch(() => {});
      }
    });
    return () => { alive = false; cleanupTauriListener(unlisten); };
  }, [open, projectPath, loadDirectory]);

  const toggleExpand = useCallback(async (dirPath: string) => {
    setExpandedPaths((prev) => {
      const next = new Set(prev);
      if (next.has(dirPath)) {
        next.delete(dirPath);
        return next;
      }
      if (next.size >= MAX_EXPANDED) return prev;
      next.add(dirPath);
      return next;
    });

    if (!childrenMap.has(dirPath)) {
      const entries = await loadDirectory(dirPath).catch(() => [] as FileEntry[]);
      setChildrenMap((prev) => {
        const next = new Map(prev);
        next.set(dirPath, entries);
        if (next.size > MAX_CACHED_DIRS) {
          const current = expandedRef.current;
          for (const key of next.keys()) {
            if (next.size <= MAX_CACHED_DIRS) break;
            if (!current.has(key)) next.delete(key);
          }
        }
        return next;
      });
    }
  }, [childrenMap, loadDirectory]);

  const toggleOpen = useCallback(() => {
    setOpen((v) => !v);
  }, []);

  const closeTree = useCallback(() => {
    setOpen(false);
  }, []);

  useEffect(() => {
    localStorage.setItem(treeStorageKey(sessionId), String(width));
  }, [sessionId, width]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- session switch must reset the project tree state
    setOpen(false);
    setRootEntries([]);
    setChildrenMap(new Map());
    setExpandedPaths(new Set());
    setFilter("");
    setLoadError(null);
  }, [sessionId]);

  return {
    open,
    width,
    resizing,
    hasProject,
    rootEntries,
    childrenMap,
    expandedPaths,
    filter,
    loadError,
    setFilter,
    setOpen,
    toggleOpen,
    closeTree,
    toggleExpand,
    startResize,
  };
}
