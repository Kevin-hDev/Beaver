import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GitDiffPreview, RecordedDiffPreview } from "../git-diff-preview";
import type { GitDiffPreview as GitDiffData } from "@/types/file-preview";

const readGitDiffPreview = vi.fn<() => Promise<GitDiffData>>();

vi.mock("@/services/file-preview", () => ({
  readGitDiffPreview: () => readGitDiffPreview(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("GitDiffPreview", () => {
  it("affiche directement un diff historique sans relire Git", () => {
    render(
      <RecordedDiffPreview
        path="src/example.txt"
        status="deleted"
        data={{
          binary: false,
          truncated: false,
          hunks: [{
            old_start: 1,
            old_lines: 1,
            new_start: 0,
            new_lines: 0,
            lines: [{ kind: "deleted", content: "historical", old_line: 1, new_line: null }],
          }],
        }}
      />,
    );

    expect(screen.getByText("historical")).toBeInTheDocument();
    expect(screen.getByText("filePreview.gitStatus.deleted")).toBeInTheDocument();
    expect(readGitDiffPreview).not.toHaveBeenCalled();
  });

  it("affiche un fichier créé comme un fichier normal", () => {
    const { container } = render(
      <RecordedDiffPreview
        path="plan.md"
        status="added"
        data={{
          binary: false,
          truncated: false,
          hunks: [{
            old_start: 0,
            old_lines: 0,
            new_start: 1,
            new_lines: 2,
            lines: [
              { kind: "added", content: "# Plan", old_line: null, new_line: 1 },
              { kind: "added", content: "texte", old_line: null, new_line: 2 },
            ],
          }],
        }}
      />,
    );

    expect(screen.getByText("filePreview.gitStatus.added")).toBeInTheDocument();
    expect(container.querySelector(".tp-line-ok")).toBeNull();
    expect(container.querySelectorAll(".tp-line-context")).toHaveLength(2);
    expect(
      Array.from(container.querySelectorAll(".tp-prefix"), (element) => element.textContent),
    ).toEqual([" ", " "]);
  });

  it("colorie la ligne ajoutée sans le commentaire ouvert par la ligne retirée", () => {
    const { container } = render(
      <RecordedDiffPreview
        path="src/example.ts"
        status="modified"
        data={{
          binary: false,
          truncated: false,
          hunks: [{
            old_start: 1,
            old_lines: 2,
            new_start: 1,
            new_lines: 2,
            lines: [
              { kind: "context", content: "const a = 1;", old_line: 1, new_line: 1 },
              { kind: "deleted", content: "/* ancien", old_line: 2, new_line: null },
              { kind: "added", content: "const neuf = 2;", old_line: null, new_line: 2 },
            ],
          }],
        }}
      />,
    );

    const added = container.querySelector(".tp-line-ok .tp-code")?.innerHTML ?? "";
    expect(added).not.toContain("hljs-comment");
    expect(added).toContain("hljs-keyword");
    expect(container.querySelector(".tp-line-error .tp-code")?.innerHTML).toContain("hljs-comment");
  });

  it("affiche les anciens et nouveaux numéros dans une seule colonne", async () => {
    readGitDiffPreview.mockResolvedValue({
      binary: false,
      truncated: true,
      hunks: [{
        old_start: 4,
        old_lines: 2,
        new_start: 4,
        new_lines: 2,
        lines: [
          { kind: "context", content: "same", old_line: 4, new_line: 4 },
          { kind: "deleted", content: "old", old_line: 5, new_line: null },
          { kind: "added", content: "new", old_line: null, new_line: 5 },
        ],
      }],
    });

    const { container } = render(
      <GitDiffPreview
        path="src/example.txt"
        baseDir="/repo"
        source={{
          kind: "git-diff",
          mode: "working",
          status: "modified",
          commitId: "a".repeat(40),
          filePath: "src/example.txt",
          expectedBranch: "main",
        }}
      />,
    );

    await waitFor(() => expect(screen.getByText("old")).toBeInTheDocument());
    expect(screen.getByText("new")).toBeInTheDocument();
    expect(screen.getByText("filePreview.gitStatus.modified")).toBeInTheDocument();
    expect(
      Array.from(container.querySelectorAll(".gdp-line-number"), (element) => element.textContent),
    ).toEqual(["4", "5", "5"]);
    expect(container.querySelector(".gdp-hunk-header")).toBeNull();
    expect(container).not.toHaveTextContent("@@");
    expect(screen.getByText("filePreview.diffTruncated")).toBeInTheDocument();
    expect(container.querySelector(".tp-line-ok")).not.toBeNull();
    expect(container.querySelector(".tp-line-error")).not.toBeNull();
  });

  it("affiche un renommage sans changement de contenu", async () => {
    readGitDiffPreview.mockResolvedValue({ binary: false, truncated: false, hunks: [] });

    render(
      <GitDiffPreview
        path="src/new.txt"
        baseDir="/repo"
        source={{
          kind: "git-diff",
          mode: "commit",
          status: "renamed",
          commitId: "b".repeat(40),
          filePath: "src/new.txt",
          previousPath: "src/old.txt",
          expectedBranch: "main",
        }}
      />,
    );

    await waitFor(() => expect(screen.getByText("src/old.txt")).toBeInTheDocument());
    expect(screen.getByText("src/new.txt")).toBeInTheDocument();
    expect(screen.getByText("filePreview.gitStatus.renamed")).toBeInTheDocument();
    expect(screen.queryByText("filePreview.diffUnavailable")).not.toBeInTheDocument();
  });

  it("sépare simplement les modifications éloignées", async () => {
    readGitDiffPreview.mockResolvedValue({
      binary: false,
      truncated: false,
      hunks: [
        {
          old_start: 1,
          old_lines: 1,
          new_start: 1,
          new_lines: 1,
          lines: [{ kind: "added", content: "first", old_line: null, new_line: 1 }],
        },
        {
          old_start: 20,
          old_lines: 1,
          new_start: 20,
          new_lines: 1,
          lines: [{ kind: "added", content: "second", old_line: null, new_line: 20 }],
        },
      ],
    });

    const { container } = render(
      <GitDiffPreview
        path="src/example.txt"
        source={{
          kind: "git-diff",
          mode: "commit",
          status: "modified",
          commitId: "c".repeat(40),
          filePath: "src/example.txt",
          expectedBranch: "main",
        }}
      />,
    );

    await waitFor(() => expect(screen.getByText("second")).toBeInTheDocument());
    expect(container.querySelectorAll(".gdp-hunk-separator")).toHaveLength(1);
    expect(container).not.toHaveTextContent("@@");
  });
});
