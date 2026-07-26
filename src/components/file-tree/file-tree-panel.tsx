import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { filterTree } from "@/lib/file-tree-filter";
import { FileTreeFilter } from "./file-tree-filter";
import { focusFileTreePath, nearestFileTreePath, visibleFileTreeEntries } from "./file-tree-keyboard";
import { FileTreeNode } from "./file-tree-node";
import type { useFileTree } from "@/hooks/use-file-tree";
import "./file-tree-panel.css";

const MAX_RENDER_DEPTH = 50;

interface FileTreePanelProps {
  tree: ReturnType<typeof useFileTree>;
  displayWidth?: number;
  onFileSelect: (path: string) => void;
  activePath: string | null;
}

export function FileTreePanel({ tree, displayWidth, onFileSelect, activePath }: FileTreePanelProps) {
  const { t } = useTranslation();
  const [debouncedFilter, setDebouncedFilter] = useState("");
  const timerRef = useRef(0);

  const handleFilterChange = (value: string) => {
    tree.setFilter(value);
    clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => setDebouncedFilter(value), 150);
  };

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- clearing debounce immediately keeps filter reset deterministic
    if (tree.filter === "") setDebouncedFilter("");
  }, [tree.filter]);

  useEffect(() => {
    return () => clearTimeout(timerRef.current);
  }, []);

  const filtered = useMemo(
    () => filterTree(tree.rootEntries, tree.childrenMap, debouncedFilter),
    [tree.rootEntries, tree.childrenMap, debouncedFilter],
  );

  const mergedExpanded = useMemo(() => {
    const merged = new Set(tree.expandedPaths);
    for (const p of filtered.expandedPaths) merged.add(p);
    return merged;
  }, [tree.expandedPaths, filtered.expandedPaths]);

  const visibleEntries = useMemo(
    () => visibleFileTreeEntries({ rootEntries: filtered.entries, childrenMap: tree.childrenMap, expandedPaths: mergedExpanded }),
    [filtered.entries, tree.childrenMap, mergedExpanded],
  );
  const { toggleExpand } = tree;
  const handleToggle = useCallback((path: string) => {
    void toggleExpand(path);
  }, [toggleExpand]);
  const handleSelect = useCallback((path: string) => {
    onFileSelect(path);
  }, [onFileSelect]);

  const handleTreeKeyDown = (event: React.KeyboardEvent) => {
    if (!["ArrowDown", "ArrowUp", "ArrowRight", "ArrowLeft"].includes(event.key)) return;
    const currentPath = nearestFileTreePath(event.target, activePath ?? visibleEntries[0]?.entry.path ?? null);
    if (!currentPath) return;
    const index = visibleEntries.findIndex((item) => item.entry.path === currentPath);
    const item = visibleEntries[index];
    if (!item) return;

    event.preventDefault();
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      const next = visibleEntries[index + (event.key === "ArrowDown" ? 1 : -1)];
      if (!next) return;
      if (!next.entry.is_dir) onFileSelect(next.entry.path);
      focusFileTreePath(next.entry.path);
      return;
    }

    if (event.key === "ArrowRight" && item.entry.is_dir) {
      if (!mergedExpanded.has(item.entry.path)) handleToggle(item.entry.path);
      else {
        const firstChild = tree.childrenMap.get(item.entry.path)?.[0];
        if (firstChild) focusFileTreePath(firstChild.path);
      }
      return;
    }

    if (event.key === "ArrowLeft") {
      if (item.entry.is_dir && mergedExpanded.has(item.entry.path)) handleToggle(item.entry.path);
      else if (item.parentPath) focusFileTreePath(item.parentPath);
    }
  };

  return (
    <aside
      className={`ft-panel ${tree.open ? "open" : ""} ${tree.resizing ? "resizing" : ""}`}
      data-nav-zone="fileTree"
      style={{ "--ft-width": `${displayWidth ?? tree.width}px` } as React.CSSProperties}
      aria-hidden={!tree.open}
    >
      <div className="ft-resize" onPointerDown={tree.startResize} />
      <div className="ft-head">
        <FileTreeFilter value={tree.filter} onChange={handleFilterChange} />
      </div>
      <div className="ft-body" role="tree" tabIndex={0} onKeyDown={handleTreeKeyDown}>
        {tree.loadError ? (
          <div className="ft-empty">{t("fileTree.loadError")}</div>
        ) : filtered.entries.length === 0 && debouncedFilter ? (
          <div className="ft-empty">{t("fileTree.noResults")}</div>
        ) : (
          filtered.entries.map((entry) => (
            <FileTreeNode
              key={entry.path}
              entry={entry}
              depth={0}
              maxDepth={MAX_RENDER_DEPTH}
              expanded={mergedExpanded.has(entry.path)}
              active={entry.path === activePath}
              childEntries={tree.childrenMap.get(entry.path)}
              expandedPaths={mergedExpanded}
              childrenMap={tree.childrenMap}
              activePath={activePath}
              onToggle={handleToggle}
              onSelect={handleSelect}
            />
          ))
        )}
      </div>
    </aside>
  );
}
