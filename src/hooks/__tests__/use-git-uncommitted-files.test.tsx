import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useGitUncommittedFiles } from "../use-git-uncommitted-files";

describe("useGitUncommittedFiles", () => {
  it("suit le snapshot Git partagé du worktree", () => {
    const { result, rerender } = renderHook(
      ({ path }) => useGitUncommittedFiles({
        isGitRepo: true,
        currentBranch: "main",
        dirtyCount: 1,
        uncommittedSnapshot: snapshot(path),
      }),
      { initialProps: { path: "first.ts" } },
    );

    expect(result.current[0]?.path).toBe("first.ts");
    rerender({ path: "edited.ts" });
    expect(result.current[0]?.path).toBe("edited.ts");
  });

  it("masque un ancien snapshot dès que le worktree est propre", () => {
    const { result } = renderHook(() => useGitUncommittedFiles({
      isGitRepo: true,
      currentBranch: "main",
      dirtyCount: 0,
      uncommittedSnapshot: snapshot("stale.ts"),
    }));

    expect(result.current).toEqual([]);
  });
});

function snapshot(path: string) {
  return {
    head_commit: "a".repeat(40),
    total_files: 1,
    truncated: false,
    files: [{ path, status: "modified", additions: 1, deletions: 0 }],
  };
}
