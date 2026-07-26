import { type ReactNode } from "react";
import { GitBranch, Check, Plus } from "@/components/ui/icons";

interface BranchItem {
  name: string;
  is_current: boolean;
  dirty_count: number;
}

interface WorktreeItem {
  path: string;
  branch: string;
}

interface BranchSelectorBranchItemProps {
  branch: BranchItem;
  dirtyLabel?: string;
  onSelect: (name: string) => void;
  deleteControl: ReactNode;
}

interface BranchSelectorWorktreeItemProps {
  worktree: WorktreeItem;
  onSelect: (path: string, branch: string) => void;
  deleteControl: ReactNode;
}

interface BranchSelectorCreateItemProps {
  label: string;
  onStart: () => void;
}

interface BranchSelectorLockedIndicatorProps {
  label: string;
}

export function BranchSelectorBranchItem({
  branch,
  dirtyLabel,
  onSelect,
  deleteControl,
}: BranchSelectorBranchItemProps) {
  return (
    <div className={`bs-item ${branch.is_current ? "bs-selected" : ""}`}>
      <button className="bs-item-select" type="button" onClick={() => onSelect(branch.name)}>
        <GitBranch size="var(--icon-sm)" />
        <span className="bs-item-info">
          <span className="bs-item-name">{branch.name}</span>
          {dirtyLabel && <span className="bs-item-detail">{dirtyLabel}</span>}
        </span>
        {branch.is_current && <Check size="var(--icon-sm)" />}
      </button>
      {deleteControl}
    </div>
  );
}

export function BranchSelectorWorktreeItem({
  worktree,
  onSelect,
  deleteControl,
}: BranchSelectorWorktreeItemProps) {
  const name = worktree.branch || worktree.path;

  return (
    <div className="bs-item">
      <button className="bs-item-select" type="button" onClick={() => onSelect(worktree.path, worktree.branch)}>
        <GitBranch size="var(--icon-sm)" />
        <span className="bs-item-info">
          <span className="bs-item-name">{name}</span>
          <span className="bs-item-detail">{worktree.path}</span>
        </span>
      </button>
      {deleteControl}
    </div>
  );
}

export function BranchSelectorCreateItem({ label, onStart }: BranchSelectorCreateItemProps) {
  return (
    <div
      className="bs-item"
      role="button"
      tabIndex={0}
      onClick={onStart}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") onStart();
      }}
    >
      <Plus size="var(--icon-sm)" />
      <span>{label}</span>
    </div>
  );
}

export function BranchSelectorLockedIndicator({ label }: BranchSelectorLockedIndicatorProps) {
  return (
    <div className="bs-row">
      <div className="bs-indicator">
        <GitBranch size="var(--icon-sm)" />
        <span>{label}</span>
      </div>
    </div>
  );
}
