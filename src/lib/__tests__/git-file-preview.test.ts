import { describe, expect, it } from "vitest";
import {
  commitFileOperation,
  uncommittedChangeSummary,
  uncommittedFileOperations,
} from "../git-file-preview";

describe("git-file-preview", () => {
  it("lit un fichier supprimé depuis le parent du commit", () => {
    const operation = commitFileOperation(
      { id: "a".repeat(40), short_id: "aaaaaaaa", message: "delete", timestamp: 1 },
      { path: "src/old.ts", status: "deleted", additions: 0, deletions: 4 },
      "main",
    );

    expect(operation.source).toEqual(expect.objectContaining({
      filePath: "src/old.ts",
      expectedBranch: "main",
      useParent: true,
    }));
    expect(operation.gitDiff).toEqual(expect.objectContaining({
      mode: "commit",
      status: "deleted",
      commitId: "a".repeat(40),
      filePath: "src/old.ts",
    }));
  });

  it("borne et mappe les changements non commit", () => {
    const files = Array.from({ length: 205 }, (_, index) => ({
      path: `file-${index}.txt`,
      status: "modified",
      additions: index,
      deletions: 0,
    }));
    const operations = uncommittedFileOperations({
      head_commit: "b".repeat(40),
      files,
      total_files: files.length,
      truncated: true,
    }, "feature");

    expect(operations).toHaveLength(200);
    expect(operations[0]).toEqual(expect.objectContaining({
      id: "git-uncommitted:feature:file-0.txt",
      path: "file-0.txt",
    }));
    expect(operations[0]?.gitDiff).toEqual(expect.objectContaining({
      mode: "working",
      status: "modified",
      commitId: "b".repeat(40),
      expectedBranch: "feature",
    }));
  });

  it("conserve l'ancien chemin d'un fichier renommé", () => {
    const [operation] = uncommittedFileOperations({
      head_commit: "c".repeat(40),
      total_files: 1,
      truncated: false,
      files: [{
        path: "src/new.ts",
        previous_path: "src/old.ts",
        status: "renamed",
        additions: 1,
        deletions: 1,
      }],
    }, "feature");

    expect(operation.gitDiff?.previousPath).toBe("src/old.ts");
    expect(operation.gitDiff?.status).toBe("renamed");
  });

  it("présente un nouveau fichier comme créé", () => {
    const [operation] = uncommittedFileOperations({
      head_commit: "d".repeat(40),
      total_files: 1,
      truncated: false,
      files: [{ path: "new.txt", status: "new", additions: 2, deletions: 0 }],
    }, "main");

    expect(operation.gitDiff?.status).toBe("added");
  });

  it("additionne le diff Git net du worktree", () => {
    expect(uncommittedChangeSummary({
      head_commit: "e".repeat(40),
      total_files: 2,
      truncated: false,
      files: [
        { path: "a.ts", status: "modified", additions: 10, deletions: 2 },
        { path: "b.ts", status: "new", additions: 4, deletions: 4 },
      ],
    })).toEqual({ additions: 14, deletions: 6 });
  });
});
