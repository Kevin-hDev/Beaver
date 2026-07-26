import { useState, useRef, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { GitBranch, CaretDown } from "@/components/ui/icons";
import { BranchSelectorLockedIndicator } from "./branch-selector-items";
import { BranchSelectorMenu } from "./branch-selector-menu";
import { branchCreateErrorKey, getVisibleBranchOptions } from "./branch-selector-utils";
import { useKeyboard } from "@/hooks/use-keyboard";
import { useClickOutside } from "@/hooks/use-click-outside";
import { showToast } from "@/lib/toast-emitter";
import { validateBranchName } from "@/lib/branch-name";
import { useGitDeletionFlow } from "@/hooks/use-git-deletion-flow";
import { showAppError } from "@/lib/app-error";
import type { BranchSelectorProps } from "./branch-selector-types";
import "./branch-selector.css";
import "./branch-selector-controls.css";

export function BranchSelector({
  git, locked, lockedLabel, onConflict, onWorktreeSelect, onGithubAuthRequired, onBranchReady,
}: BranchSelectorProps) {
  const { t } = useTranslation();
  const deletion = useGitDeletionFlow(git);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [creating, setCreating] = useState(false);
  const [createName, setCreateName] = useState("");
  const [createError, setCreateError] = useState("");
  const [isCreating, setIsCreating] = useState(false);
  const dropRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const createRef = useRef<HTMLInputElement>(null);
  useKeyboard({ onEscape: () => {
    if (creating) {
      setCreating(false);
      setCreateName("");
      setCreateError("");
    } else {
      setOpen(false);
    }
  }});
  useClickOutside(dropRef, () => {
    if (isCreating) return;
    setOpen(false);
    setCreating(false);
  });
  useEffect(() => {
    if (open && !creating && searchRef.current) searchRef.current.focus();
  }, [open, creating]);

  useEffect(() => {
    if (creating && createRef.current) createRef.current.focus();
  }, [creating]);

  const notifyBranchReady = useCallback(async (name: string) => {
    try {
      await onBranchReady?.(name);
    } catch {
      showToast(t("branches.errorInternal"), "error", 3000);
    }
  }, [onBranchReady, t]);

  const handleSelect = useCallback(async (name: string) => {
    if (name === git.currentBranch) {
      setOpen(false);
      return;
    }
    const result = await git.checkout(name);
    if (result.ok) {
      await notifyBranchReady(name);
      setOpen(false);
      setSearch("");
    } else if (result.dirtyCount != null) {
      setOpen(false);
      onConflict(name, result.dirtyCount);
    } else {
      showAppError(result, t, "branches.errorCheckoutFailed");
    }
  }, [git, notifyBranchReady, onConflict, t]);

  const handleCreate = useCallback(async () => {
    if (isCreating) return;
    const name = createName.trim();
    const validation = validateBranchName(name);
    if (!validation.valid) {
      setCreateError(t(branchCreateErrorKey(validation.reason)));
      return;
    }

    setIsCreating(true);
    try {
      const result = await git.create(name);
      if (result.ok) {
        await notifyBranchReady(name);
        showToast(`${t("branches.bubbleCreated")}: ${name}`, "success", 3000);
        setOpen(false);
        setCreating(false);
        setCreateName("");
        setCreateError("");
        setSearch("");
      } else if (result.reason === "github_auth_required") {
        setOpen(false);
        setCreating(false);
        setCreateName("");
        setCreateError("");
        onGithubAuthRequired();
      } else {
        setCreateError(t(branchCreateErrorKey(result.kind)));
      }
    } finally {
      setIsCreating(false);
    }
  }, [createName, git, isCreating, notifyBranchReady, onGithubAuthRequired, t]);

  const handleWorktreeSelect = useCallback((path: string, branch: string) => {
    setOpen(false);
    onWorktreeSelect(path, branch);
  }, [onWorktreeSelect]);
  const cancelCreate = useCallback(() => {
    if (isCreating) return;
    setCreating(false);
    setCreateName("");
    setCreateError("");
  }, [isCreating]);

  if (!git.isGitRepo) return null;

  if (locked) {
    return <BranchSelectorLockedIndicator label={lockedLabel || git.currentBranch || t("branches.detachedHead")} />;
  }

  const { filteredBranches, filteredWorktrees } = getVisibleBranchOptions(
    git.branches,
    git.worktrees,
    search,
  );

  const label = git.currentBranch || t("branches.detachedHead");
  return (
    <div className="bs-row" ref={dropRef}>
      <button className="bs-btn" onClick={() => setOpen(!open)}>
        <GitBranch size="var(--icon-sm)" />
        <span>{label}</span>
        <CaretDown size="var(--icon-2xs)" />
      </button>
      {open && (
        <BranchSelectorMenu
          branches={filteredBranches}
          worktrees={filteredWorktrees}
          search={search}
          searchRef={searchRef}
          creating={creating}
          createRef={createRef}
          createName={createName}
          createError={createError}
          isCreating={isCreating}
          onSearchChange={setSearch}
          onSelectBranch={(name) => void handleSelect(name)}
          onSelectWorktree={handleWorktreeSelect}
          onInspectBranch={deletion.inspectBranch}
          onDeleteBranch={deletion.deleteCleanBranch}
          onInspectWorktree={deletion.inspectWorktree}
          onDeleteWorktree={deletion.deleteCleanWorktree}
          onCreateNameChange={(value) => { setCreateName(value); setCreateError(""); }}
          onCreate={() => void handleCreate()}
          onCancelCreate={cancelCreate}
          onStartCreate={() => setCreating(true)}
        />
      )}
      {deletion.dialog}
    </div>
  );
}
