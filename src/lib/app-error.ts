import { showToast } from "@/lib/toast-emitter";

export type AppErrorKind = typeof APP_ERROR_KINDS[number];

export interface AppError {
  kind: AppErrorKind;
  dirtyCount?: number;
  count?: number;
}

type ErrorTranslator = (key: string, options?: { count: number }) => string;

export type GitCreateBranchErrorKind = Extract<AppErrorKind,
  | "invalid_name"
  | "name_too_long"
  | "already_exists"
  | "unborn_head"
  | "github_auth_required"
  | "internal_error"
>;

export type GitCreateBranchResult =
  | { ok: true }
  | { ok: false; reason?: "github_auth_required"; kind?: GitCreateBranchErrorKind };

export function isGitCreateBranchErrorKind(
  kind: string | undefined,
): kind is GitCreateBranchErrorKind {
  return kind === "invalid_name"
    || kind === "name_too_long"
    || kind === "already_exists"
    || kind === "unborn_head"
    || kind === "github_auth_required"
    || kind === "internal_error";
}

const APP_ERROR_KINDS = [
  "invalid_name", "name_too_long", "already_exists", "unborn_head",
  "github_auth_required", "repository_unavailable", "branch_unavailable",
  "dirty_worktree", "no_fallback_branch", "protected_branch", "branch_active",
  "identity_missing", "invalid_commit_description", "checkout_failed",
  "commit_failed", "merge_failed", "unmerged_commits", "delete_failed",
  "worktree_unavailable", "clone_unavailable", "no_remote",
  "authentication_required", "permission_denied", "remote_changed",
  "network_unavailable", "context_changed", "nothing_to_merge",
  "merge_conflict", "internal_error",
] as const;

const KNOWN_KINDS = new Set<string>(APP_ERROR_KINDS);

const ERROR_KEYS: Record<AppErrorKind, string> = {
  invalid_name: "branches.errorInvalidName",
  name_too_long: "branches.errorNameTooLong",
  already_exists: "branches.errorAlreadyExists",
  unborn_head: "branches.errorUnbornHead",
  github_auth_required: "branches.errorGithubAuth",
  repository_unavailable: "branches.errorRepositoryUnavailable",
  branch_unavailable: "branches.errorBranchUnavailable",
  dirty_worktree: "branches.errorDirtyWorktree",
  no_fallback_branch: "branches.deleteNoFallback",
  protected_branch: "branches.errorProtectedBranch",
  branch_active: "branches.errorBranchActive",
  identity_missing: "branches.errorIdentityMissing",
  invalid_commit_description: "branches.errorInvalidCommitDescription",
  checkout_failed: "branches.errorCheckoutFailed",
  commit_failed: "branches.errorCommitFailed",
  merge_failed: "agentLocal.sessionSummary.git.mergeError",
  unmerged_commits: "branches.errorUnmergedCommits",
  delete_failed: "branches.deleteError",
  worktree_unavailable: "branches.errorWorktreeUnavailable",
  clone_unavailable: "agentLocal.clone.errorUnavailable",
  no_remote: "agentLocal.sessionSummary.git.noRemote",
  authentication_required: "agentLocal.sessionSummary.git.authenticationError",
  permission_denied: "agentLocal.sessionSummary.git.permissionDenied",
  remote_changed: "agentLocal.sessionSummary.git.remoteChanged",
  network_unavailable: "agentLocal.sessionSummary.git.networkError",
  context_changed: "agentLocal.sessionSummary.git.contextChanged",
  nothing_to_merge: "agentLocal.sessionSummary.git.mergeAlready",
  merge_conflict: "agentLocal.sessionSummary.git.mergeConflict",
  internal_error: "errors.operationFailed",
};

export function parseAppError(error: unknown): AppError | null {
  if (typeof error === "string") {
    if (error.includes("GITHUB_AUTH_REQUIRED")) {
      return { kind: "github_auth_required" };
    }
    try {
      return parseAppError(JSON.parse(error));
    } catch {
      return null;
    }
  }
  if (!error || typeof error !== "object") return null;
  const value = error as Record<string, unknown>;
  if (typeof value.kind !== "string" || !KNOWN_KINDS.has(value.kind)) return null;
  return {
    kind: value.kind as AppErrorKind,
    dirtyCount: safeCount(value.dirty_count ?? value.dirtyCount),
    count: safeCount(value.count),
  };
}

export function appErrorKey(error: unknown, fallback = "errors.operationFailed"): string {
  const parsed = typeof error === "string" && KNOWN_KINDS.has(error)
    ? { kind: error as AppErrorKind }
    : parseAppError(error);
  return parsed ? ERROR_KEYS[parsed.kind] : fallback;
}

export function appErrorMessage(
  error: unknown,
  translate: ErrorTranslator,
  fallback = "errors.operationFailed",
): string {
  const parsed = parseAppError(error);
  const key = parsed?.kind === "dirty_worktree" && parsed.dirtyCount == null
    ? "branches.commitRequired"
    : parsed ? ERROR_KEYS[parsed.kind] : fallback;
  return translate(key, {
    count: parsed?.count ?? parsed?.dirtyCount ?? 0,
  });
}

export function showAppError(
  error: unknown,
  translate: ErrorTranslator,
  fallback?: string,
  duration = 3000,
) {
  showToast(appErrorMessage(error, translate, fallback), "error", duration);
}

function safeCount(value: unknown): number | undefined {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0
    ? value
    : undefined;
}
