import { memo, useEffect, useRef, useState } from "react";
import { ChevronRight } from "@/components/ui/icons";
import { FileIcon } from "@/components/file-preview/file-icon";
import type { FileEntry } from "@/lib/file-tree-filter";

interface FileTreeNodeProps {
  entry: FileEntry;
  depth: number;
  maxDepth: number;
  expanded: boolean;
  active: boolean;
  childEntries?: FileEntry[];
  expandedPaths: Set<string>;
  childrenMap: Map<string, FileEntry[]>;
  activePath: string | null;
  onToggle: (path: string) => void;
  onSelect: (path: string) => void;
}

function FileTreeNodeComponent({
  entry,
  depth,
  maxDepth,
  expanded,
  active,
  childEntries,
  expandedPaths,
  childrenMap,
  activePath,
  onToggle,
  onSelect,
}: FileTreeNodeProps) {
  const childrenRef = useRef<HTMLDivElement>(null);
  const [maxHeight, setMaxHeight] = useState<string>(expanded ? "none" : "0");

  const childCount = childEntries?.length ?? 0;

  useEffect(() => {
    if (!childrenRef.current) return;
    if (expanded) {
      const h = childrenRef.current.scrollHeight;
      setMaxHeight(`${h}px`);
      const timer = setTimeout(() => setMaxHeight("none"), 200);
      return () => clearTimeout(timer);
    }
    const h = childrenRef.current.scrollHeight;
    setMaxHeight(`${h}px`);
    requestAnimationFrame(() => setMaxHeight("0"));
  }, [expanded, childCount]);

  const handleClick = () => {
    if (entry.is_dir) {
      onToggle(entry.path);
    } else {
      onSelect(entry.path);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleClick();
    }
  };

  if (depth >= maxDepth) return null;

  return (
    <div>
      <div
        className={`ft-node ${active ? "ft-node-active" : ""}`}
        style={{ paddingLeft: depth * 16 + 8 }}
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        role="treeitem"
        tabIndex={active ? 0 : -1}
        data-ft-path={entry.path}
        data-ft-dir={entry.is_dir ? "true" : undefined}
        data-nav-active={active ? "true" : undefined}
        title={entry.name}
        aria-selected={active}
        aria-expanded={entry.is_dir ? expanded : undefined}
      >
        <span className={`ft-chevron ${entry.is_dir ? "" : "ft-chevron-placeholder"} ${expanded ? "expanded" : ""}`}>
          {entry.is_dir && <ChevronRight size="var(--icon-sm)" />}
        </span>
        {!entry.is_dir && <FileIcon name={entry.name} size="var(--icon-md)" />}
        <span className="ft-node-name">{entry.name}</span>
      </div>
      {entry.is_dir && childEntries && (
        <div
          ref={childrenRef}
          className={`ft-children ${expanded ? "" : "collapsed"}`}
          style={{ maxHeight: expanded ? maxHeight : "0" }}
          role="group"
        >
          {childEntries.map((child) => (
            <FileTreeNode
              key={child.path}
              entry={child}
              depth={depth + 1}
              maxDepth={maxDepth}
              expanded={expandedPaths.has(child.path)}
              active={child.path === activePath}
              childEntries={childrenMap.get(child.path)}
              expandedPaths={expandedPaths}
              childrenMap={childrenMap}
              activePath={activePath}
              onToggle={onToggle}
              onSelect={onSelect}
            />
          ))}
          {childEntries.length === 0 && expanded && (
            <div className="ft-empty" style={{ paddingLeft: (depth + 1) * 16 + 8 }}>—</div>
          )}
        </div>
      )}
    </div>
  );
}

export const FileTreeNode = memo(FileTreeNodeComponent);
