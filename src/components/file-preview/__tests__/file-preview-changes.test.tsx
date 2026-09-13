import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FilePreviewChanges } from "../file-preview-changes";
import type { FileOperation } from "@/types/file-preview";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: { index?: number; count?: number }) =>
      values ? `${key} ${values.index ?? ""}/${values.count}` : key,
  }),
}));

describe("FilePreviewChanges", () => {
  it("montre chaque changement l'un sous l'autre, le plus récent en haut, avec ses chiffres", () => {
    const { container } = render(
      <FilePreviewChanges operation={file(3, 1, [change("dernier", 2, 0), change("premier", 1, 1)])} />,
    );

    expect(headings(container)).toEqual(["filePreview.changeOf 2/2+2", "filePreview.changeOf 1/2+1-1"]);
    expect(screen.getByText("dernier")).toBeInTheDocument();
    expect(screen.getByText("premier")).toBeInTheDocument();
  });

  it.each(["absent", "binaire", "vide"])("garde l’en-tête et les chiffres avec un diff %s", (kind) => {
    const latest = change("dernier", 2, 0);
    latest.recordedDiff = kind === "absent"
      ? undefined
      : { binary: kind === "binaire", truncated: false, hunks: [] };
    const { container } = render(
      <FilePreviewChanges operation={file(3, 1, [latest, change("premier", 1, 1)])} />,
    );

    expect(headings(container)).toEqual(["filePreview.changeOf 2/2+2", "filePreview.changeOf 1/2+1-1"]);
    expect(screen.getByText("filePreview.diffUnavailable")).toBeInTheDocument();
    expect(screen.getByText("premier")).toBeInTheDocument();
  });

  it("résume les changements au-delà du plafond, pour que la somme redonne le total", () => {
    const { container } = render(
      <FilePreviewChanges operation={{
        ...file(7, 3, [change("a", 2, 0), change("b", 1, 1)]),
        olderChanges: { count: 3, additions: 4, deletions: 2 },
      }} />,
    );

    expect(headings(container)).toEqual([
      "filePreview.changeOf 5/5+2",
      "filePreview.changeOf 4/5+1-1",
      "filePreview.olderChanges /3+4-2",
    ]);
  });
});

function headings(container: HTMLElement): (string | null)[] {
  return [...container.querySelectorAll(".fpc-heading")].map((heading) => heading.textContent);
}

function file(additions: number, deletions: number, changes: FileOperation[]): FileOperation {
  return { ...changes[0], id: "file:m1:/repo/a.ts", additions, deletions, changes };
}

function change(content: string, additions: number, deletions: number): FileOperation {
  return {
    id: `m1-${content}`,
    path: "/repo/a.ts",
    name: "a.ts",
    type: "edit",
    timestamp: "2026-09-11T10:00:00Z",
    additions,
    deletions,
    recordedStatus: "modified",
    recordedDiff: {
      binary: false,
      truncated: false,
      hunks: [{
        old_start: 1, old_lines: 1, new_start: 1, new_lines: 1,
        lines: [{ kind: "added", content, old_line: null, new_line: 1 }],
      }],
    },
  };
}
