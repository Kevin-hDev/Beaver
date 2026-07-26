import { beforeEach, describe, expect, it, vi } from "vitest";
import { appErrorKey, appErrorMessage, parseAppError, showAppError } from "./app-error";
import { showToast } from "./toast-emitter";

vi.mock("./toast-emitter", () => ({ showToast: vi.fn() }));

const translate = (key: string, values?: { count?: number }) => `${key}:${values?.count ?? ""}`;

describe("app-error", () => {
  beforeEach(() => vi.clearAllMocks());

  it("lit une erreur Tauri structurée", () => {
    expect(parseAppError({ kind: "dirty_worktree", dirty_count: 4 })).toEqual({
      kind: "dirty_worktree",
      dirtyCount: 4,
      count: undefined,
    });
  });

  it("lit aussi une erreur sérialisée en JSON", () => {
    expect(parseAppError('{"kind":"protected_branch"}')?.kind).toBe("protected_branch");
  });

  it("refuse une erreur inconnue et masque son texte", () => {
    expect(parseAppError("secret path /Users/name")).toBeNull();
    expect(appErrorMessage("secret path /Users/name", translate)).toBe("errors.operationFailed:0");
  });

  it("retourne la traduction centralisée d'un code connu", () => {
    expect(appErrorKey("branch_unavailable")).toBe("branches.errorBranchUnavailable");
  });

  it("n'affiche pas un faux compteur quand le backend n'en fournit pas", () => {
    expect(appErrorMessage({ kind: "dirty_worktree" }, translate))
      .toBe("branches.commitRequired:0");
  });

  it("affiche le toast commun sans exposer le message backend", () => {
    showAppError({ kind: "clone_unavailable", message: "/private/session.json" }, translate);
    expect(showToast).toHaveBeenCalledWith(
      "agentLocal.clone.errorUnavailable:0",
      "error",
      3000,
    );
  });
});
