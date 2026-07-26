import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ToolBubble } from "../tool-bubble";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretUp: () => <span />,
  Copy: () => <span />,
  Spinner: () => <span data-testid="spinner" />,
}));
vi.mock("../tool-icons", () => ({
  ToolIcon: ({ name }: { name: string }) => <span data-testid={`tool-icon-${name}`} />,
}));
vi.mock("../tool-status-icon", () => ({
  ToolStatusIcon: () => <span data-testid="status-icon-error" />,
}));
vi.mock("@/components/file-preview/file-icon", () => ({
  FileIcon: ({ name }: { name: string }) => <span data-testid={`file-icon-${name}`} />,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const rawCount = opts?.count;
      const count = typeof rawCount === "string" || typeof rawCount === "number"
        ? String(rawCount)
        : "";
      if (key === "agentLocal.toolActivity.actions.create") return "Create";
      if (key === "agentLocal.toolActivity.actions.edit") return "Edit";
      if (key === "agentLocal.toolActivity.actions.tool") return "Tool";
      if (key === "agentLocal.toolActivity.groups.modification") return "Changes";
      if (key === "agentLocal.toolActivity.toggleDetails") return "Show tool details";
      if (key === "agentLocal.toolActivity.counts.writes") return `${count} file written`;
      if (key === "agentLocal.toolActivity.counts.edits") return `${count} file edited`;
      return key;
    },
  }),
}));
vi.mock("../tool-previews", () => ({
  ContentPreview: () => <div data-testid="content-preview" />,
  DiffPreview: () => <div data-testid="diff-preview" />,
  WebResultsPreview: () => <div data-testid="web-preview" />,
}));
vi.mock("../tool-office-previews", () => ({
  DocumentResultPreview: () => <div data-testid="document-preview" />,
  ReadSpreadsheetPreview: () => <div data-testid="read-spreadsheet-preview" />,
  WriteDocumentPreview: () => <div data-testid="write-document-preview" />,
  WriteSpreadsheetPreview: () => <div data-testid="write-spreadsheet-preview" />,
}));
vi.mock("../tool-bubble.css", () => ({}));
vi.mock("@/lib/tool-file-path", () => ({
  isFileTool: (name: string) => [
    "write_file",
    "edit_file",
    "write_spreadsheet",
    "write_document",
  ].includes(name),
}));

beforeEach(() => vi.clearAllMocks());

function fileButton(container: HTMLElement, fileName: string) {
  const label = Array.from(container.querySelectorAll(".tb-item-name-text"))
    .find((node) => node.textContent === fileName);
  const button = label?.closest('[role="button"]');
  if (!(button instanceof HTMLElement)) throw new Error(`preview button absent for ${fileName}`);
  return button;
}

describe("ToolBubble file preview links", () => {
  it.each([
    {
      name: "write_file",
      path: "/tmp/alpha.ts",
      fileName: "alpha.ts",
      args: { path: "/tmp/alpha.ts", content: "const alpha = 1;" },
    },
    {
      name: "edit_file",
      path: "/tmp/beta.ts",
      fileName: "beta.ts",
      args: { path: "/tmp/beta.ts", old_string: "a", new_string: "b" },
    },
    {
      name: "write_spreadsheet",
      path: "/tmp/gamma.xlsx",
      fileName: "gamma.xlsx",
      args: { path: "/tmp/gamma.xlsx", operations: [] },
    },
    {
      name: "write_document",
      path: "/tmp/delta.docx",
      fileName: "delta.docx",
      args: { path: "/tmp/delta.docx", content: [] },
    },
  ])("ouvre la preview depuis le nom du fichier pour $name", ({ name, path, fileName, args }) => {
    const onFilePreview = vi.fn();
    const { container } = render(
      <ToolBubble
        tools={[{ name, args, result: "ok" }]}
        onFilePreview={onFilePreview}
      />,
    );

    fireEvent.click(fileButton(container, fileName));

    expect(onFilePreview).toHaveBeenCalledTimes(1);
    expect(onFilePreview).toHaveBeenCalledWith(path);
  });

  it("ouvre aussi la preview au clavier", () => {
    const onFilePreview = vi.fn();
    const { container } = render(
      <ToolBubble
        tools={[{
          name: "write_file",
          args: { path: "/tmp/keyboard.ts", content: "const x = 1;" },
          result: "ok",
        }]}
        onFilePreview={onFilePreview}
      />,
    );

    fireEvent.keyDown(fileButton(container, "keyboard.ts"), { key: "Enter" });

    expect(onFilePreview).toHaveBeenCalledWith("/tmp/keyboard.ts");
  });
});
