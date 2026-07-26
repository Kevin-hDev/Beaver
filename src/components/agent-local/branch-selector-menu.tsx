import type { RefObject } from "react";
import { useTranslation } from "react-i18next";
import { BranchSelectorCreateForm } from "./branch-selector-create-form";
import {
  BranchSelectorBranchItem,
  BranchSelectorCreateItem,
  BranchSelectorWorktreeItem,
} from "./branch-selector-items";
import { GitDeleteButton } from "./git-delete-button";
import type { BranchInfo, WorktreeInfo } from "@/hooks/use-git-branch";

interface BranchSelectorMenuProps {
  branches: BranchInfo[];
  worktrees: WorktreeInfo[];
  search: string;
  searchRef: RefObject<HTMLInputElement | null>;
  creating: boolean;
  createRef: RefObject<HTMLInputElement | null>;
  createName: string;
  createError: string;
  isCreating: boolean;
  onSearchChange: (value: string) => void;
  onSelectBranch: (name: string) => void;
  onSelectWorktree: (path: string, branch: string) => void;
  onInspectBranch: (name: string) => Promise<boolean>;
  onDeleteBranch: (name: string) => Promise<void>;
  onInspectWorktree: (path: string) => Promise<boolean>;
  onDeleteWorktree: (path: string) => Promise<void>;
  onCreateNameChange: (value: string) => void;
  onCreate: () => void;
  onCancelCreate: () => void;
  onStartCreate: () => void;
}

export function BranchSelectorMenu({
  branches,
  worktrees,
  search,
  searchRef,
  creating,
  createRef,
  createName,
  createError,
  isCreating,
  onSearchChange,
  onSelectBranch,
  onSelectWorktree,
  onInspectBranch,
  onDeleteBranch,
  onInspectWorktree,
  onDeleteWorktree,
  onCreateNameChange,
  onCreate,
  onCancelCreate,
  onStartCreate,
}: BranchSelectorMenuProps) {
  const { t } = useTranslation();

  return (
    <div className="bs-dropdown">
      <input
        ref={searchRef}
        className="bs-dropdown-search"
        placeholder={t("branches.search")}
        value={search}
        onChange={(event) => onSearchChange(event.target.value)}
      />
      <div className="bs-section-label">{t("branches.title")}</div>
      {branches.length === 0 && <div className="bs-empty">{t("branches.noMatch")}</div>}
      {branches.map((branch) => (
        <BranchSelectorBranchItem
          key={branch.name}
          branch={branch}
          dirtyLabel={branch.is_current && branch.dirty_count > 0
            ? t("branches.uncommitted", { count: branch.dirty_count })
            : undefined}
          onSelect={onSelectBranch}
          deleteControl={(
            <GitDeleteButton
              label={t("branches.deleteBranch", { name: branch.name })}
              confirmLabel={t("branches.confirmDelete")}
              onInspect={() => onInspectBranch(branch.name)}
              onConfirm={() => onDeleteBranch(branch.name)}
            />
          )}
        />
      ))}
      {worktrees.length > 0 && (
        <>
          <div className="bs-sep" />
          <div className="bs-section-label">{t("branches.worktrees")}</div>
          {worktrees.map((worktree) => (
            <BranchSelectorWorktreeItem
              key={worktree.path}
              worktree={worktree}
              onSelect={onSelectWorktree}
              deleteControl={(
                <GitDeleteButton
                  label={t("branches.deleteWorktree", { name: worktree.branch || worktree.path })}
                  confirmLabel={t("branches.confirmDelete")}
                  onInspect={() => onInspectWorktree(worktree.path)}
                  onConfirm={() => onDeleteWorktree(worktree.path)}
                />
              )}
            />
          ))}
        </>
      )}
      <div className="bs-sep" />
      {creating ? (
        <BranchSelectorCreateForm
          inputRef={createRef}
          value={createName}
          error={createError}
          isCreating={isCreating}
          placeholder={t("branches.createPlaceholder")}
          onValueChange={onCreateNameChange}
          onSubmit={onCreate}
          onCancel={onCancelCreate}
        />
      ) : (
        <BranchSelectorCreateItem label={t("branches.createNew")} onStart={onStartCreate} />
      )}
    </div>
  );
}
