import type { FileEntry } from "@/lib/file-tree-filter";

export interface VisibleFileTreeEntry {
  entry: FileEntry;
  parentPath: string | null;
}

interface FileTreeKeyboardState {
  rootEntries: FileEntry[];
  childrenMap: Map<string, FileEntry[]>;
  expandedPaths: Set<string>;
}

const MAX_VISIBLE_TREE_DEPTH = 50;

export function visibleFileTreeEntries({
  rootEntries,
  childrenMap,
  expandedPaths,
}: FileTreeKeyboardState): VisibleFileTreeEntry[] {
  const result: VisibleFileTreeEntry[] = [];

  function visit(entries: FileEntry[], parentPath: string | null, depth: number) {
    if (depth >= MAX_VISIBLE_TREE_DEPTH) return;
    for (const entry of entries) {
      result.push({ entry, parentPath });
      if (entry.is_dir && expandedPaths.has(entry.path)) {
        visit(childrenMap.get(entry.path) ?? [], entry.path, depth + 1);
      }
    }
  }

  visit(rootEntries, null, 0);
  return result;
}

export function focusFileTreePath(path: string) {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      const items = document.querySelectorAll<HTMLElement>("[data-ft-path]");
      const item = Array.from(items).find((el) => el.dataset.ftPath === path);
      item?.focus();
    });
  });
}

export function nearestFileTreePath(target: EventTarget | null, fallback: string | null) {
  if (target instanceof HTMLElement) {
    const item = target.closest<HTMLElement>("[data-ft-path]");
    if (item?.dataset.ftPath) return item.dataset.ftPath;
  }
  return fallback;
}
