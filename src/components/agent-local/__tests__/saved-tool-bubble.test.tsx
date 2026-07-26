import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup, fireEvent } from "@testing-library/react";
import { SavedToolBubble } from "../tool-bubble";

afterEach(cleanup);

vi.mock("@phosphor-icons/react", () => ({
  Spinner: () => <span data-testid="spinner" />,
}));
vi.mock("@/components/ui/icons", () => ({
  Copy: () => <span />,
  CaretDown: () => <span />,
  CaretUp: () => <span />,
  Check: () => <span data-testid="check-icon" />,
}));
vi.mock("../tool-icons", () => ({
  ToolIcon: ({ name }: { name: string }) => <span data-testid={`tool-icon-${name}`} />,
}));
vi.mock("../tool-status-icon", () => ({
  ToolStatusIcon: ({ message }: { message?: string }) => (
    <span data-testid="status-icon-error" data-message={message ?? ""} />
  ),
}));
vi.mock("@/components/file-preview/file-icon", () => ({
  FileIcon: ({ name }: { name: string }) => <span data-testid={`file-icon-${name}`} />,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "en" },
    t: (key: string, opts?: Record<string, unknown>) => {
      const text = (value: unknown) => (
        typeof value === "string" || typeof value === "number" ? String(value) : ""
      );
      const count = text(opts?.count);
      if (key === "agentLocal.toolActivity.inProgress") return "in progress";
      if (key === "agentLocal.toolActivity.toggleDetails") return "Show tool details";
      if (key === "agentLocal.toolActivity.groups.command") return "Commands";
      if (key === "agentLocal.toolActivity.groups.modification") return "Changes";
      if (key === "agentLocal.toolActivity.counts.commands") return `${count} command executed`;
      if (key === "agentLocal.toolActivity.counts.writes") return `${count} file written`;
      if (key === "agentLocal.toolActivity.counts.edits") return `${count} file edited`;
      if (key === "agentLocal.toolActivity.actions.read") return "Read";
      if (key === "agentLocal.toolActivity.actions.create") return "Create";
      if (key === "agentLocal.toolActivity.actions.edit") return "Edit";
      if (key === "agentLocal.toolActivity.actions.list") return "List";
      if (key === "agentLocal.toolActivity.actions.search") return "Search";
      if (key === "agentLocal.toolActivity.actions.run") return "Run";
      if (key === "agentLocal.toolActivity.actions.createBranch") return "Create branch";
      if (key === "agentLocal.toolActivity.actions.switchBranch") return "Switch branch";
      if (key === "agentLocal.toolActivity.actions.tool") return "Tool";
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
  ReadSpreadsheetPreview: () => <div data-testid="read-spreadsheet-preview" />,
  WriteSpreadsheetPreview: () => <div data-testid="write-spreadsheet-preview" />,
  DocumentResultPreview: () => <div data-testid="document-preview" />,
  WriteDocumentPreview: () => <div data-testid="write-document-preview" />,
}));
vi.mock("../tool-bubble.css", () => ({}));
vi.mock("@/lib/tool-file-path", () => ({
  isFileTool: (name: string) =>
    ["read_file", "write_file", "edit_file", "read_spreadsheet", "read_document",
      "read_image", "write_spreadsheet", "write_document", "process_image"].includes(name),
}));

beforeEach(() => vi.clearAllMocks());

function openGroup(container: HTMLElement) {
  const toggle = container.querySelector(".tb-group-toggle");
  if (!toggle) throw new Error("group toggle absent");
  fireEvent.click(toggle);
}

function openTool(container: HTMLElement, index = 0) {
  const toggle = container.querySelectorAll(".tb-toggle")[index];
  if (!toggle) throw new Error("tool toggle absent");
  fireEvent.click(toggle);
}

describe("SavedToolBubble", () => {
  it("retourne null si tools est vide", () => {
    const { container } = render(<SavedToolBubble tools={[]} />);
    expect(container.innerHTML).toBe("");
  });

  it("affiche directement un bash sauvegardé unique", () => {
    const { container } = render(
      <SavedToolBubble tools={[{ name: "bash", summary: "npm run build", result: "ok" }]} />,
    );

    expect(container.textContent).toContain("bash");
    expect(container.textContent).toContain("npm run build");
    expect(container.querySelector(".tb-group-toggle")).toBeNull();
    expect(container.textContent).not.toContain("ok");
    openTool(container);
    expect(container.textContent).toContain("ok");
  });

  it("affiche ContentPreview après ouverture du tool", () => {
    const { container, getByTestId, queryByTestId } = render(
      <SavedToolBubble
        tools={[{ name: "write_file", summary: "/tmp/bar.ts", content: "export const x = 1;", result: "ok" }]}
      />,
    );
    expect(queryByTestId("content-preview")).toBeNull();
    openTool(container);
    expect(getByTestId("content-preview")).toBeTruthy();
  });

  it("affiche DiffPreview quand old_text et new_text sont présents", () => {
    const { container, getByTestId, queryByTestId } = render(
      <SavedToolBubble
        tools={[{
          name: "edit_file",
          summary: "/tmp/bar.ts",
          old_text: "const a = 1;",
          new_text: "const a = 2;",
          result: "ok",
        }]}
      />,
    );
    expect(queryByTestId("diff-preview")).toBeNull();
    openTool(container);
    expect(getByTestId("diff-preview")).toBeTruthy();
  });

  it("n'applique pas skipWrite si l'edit_file a un summary différent du write_file", () => {
    const { container, getByTestId, queryByTestId } = render(
      <SavedToolBubble
        tools={[
          { name: "edit_file", summary: "/tmp/autre.ts", old_text: "a", new_text: "b", result: "ok" },
          { name: "write_file", summary: "/tmp/bar.ts", content: "export const x = 1;", result: "ok" },
        ]}
      />,
    );
    expect(queryByTestId("content-preview")).toBeNull();
    openGroup(container);
    openTool(container, 1);
    expect(getByTestId("content-preview")).toBeTruthy();
  });

  it("garde la preview d'un tool sauvegardé unique fermée par défaut", () => {
    const { container } = render(
      <SavedToolBubble
        tools={[{
          name: "edit_file",
          summary: "/tmp/bar.ts",
          old_text: "const a = 1;",
          new_text: "const a = 2;",
          result: "ok",
        }]}
      />,
    );

    expect(container.querySelector(".tb-group-toggle")).toBeNull();
    expect(container.querySelector(".tb-accordion.tb-open")).toBeNull();
  });
});
