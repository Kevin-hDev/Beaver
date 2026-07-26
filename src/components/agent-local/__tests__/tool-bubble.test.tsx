import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup, fireEvent } from "@testing-library/react";
import { ToolBubble } from "../tool-bubble";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  Copy: () => <span />,
  CaretDown: () => <span />,
  CaretUp: () => <span />,
  Check: () => <span data-testid="check-icon" />,
  Spinner: () => <span data-testid="spinner" />,
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
      if (key === "agentLocal.toolActivity.groupError") return "An error occurred in this group";
      if (key === "agentLocal.toolActivity.groups.command") return "Commands";
      if (key === "agentLocal.toolActivity.groups.modification") return "Changes";
      if (key === "agentLocal.toolActivity.groups.web") return "Web";
      if (key === "agentLocal.toolActivity.counts.commands") return `${count} command executed`;
      if (key === "agentLocal.toolActivity.counts.writes") return `${count} file written`;
      if (key === "agentLocal.toolActivity.counts.edits") return `${count} file edited`;
      if (key === "agentLocal.toolActivity.counts.webSearches") return `${count} web search`;
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

function openTool(container: HTMLElement) {
  const toggle = container.querySelector(".tb-toggle");
  if (!toggle) throw new Error("tool toggle absent");
  fireEvent.click(toggle);
}

describe("ToolBubble", () => {
  it("retourne null si tools est vide", () => {
    const { container } = render(<ToolBubble tools={[]} />);
    expect(container.innerHTML).toBe("");
  });

  it("retourne null si seul todo_write est présent", () => {
    const { container } = render(
      <ToolBubble tools={[{ name: "todo_write", args: { todos: [] }, result: "ok" }]} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("retourne null si seuls les tools internes sont présents", () => {
    const { container } = render(
      <ToolBubble tools={[{ name: "ask_user_choice", args: { questions: [] }, result: "ok" }]} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("affiche directement un bash unique", () => {
    const { container } = render(
      <ToolBubble tools={[{ name: "bash", args: { command: "echo hello" } }]} />,
    );

    expect(container.textContent).toContain("bash");
    expect(container.textContent).toContain("echo hello");
    expect(container.querySelector(".tb-group-toggle")).toBeNull();
  });

  it("masque la commande bash complète jusqu'au dépliage du tool", () => {
    const command = `${"a".repeat(110)} && echo done`;
    const { container } = render(
      <ToolBubble tools={[{ name: "bash", args: { command }, result: "ok", isError: false }]} />,
    );

    expect(container.textContent).toContain(`${"a".repeat(96)}...`);
    expect(container.textContent).not.toContain(command);
    openTool(container);
    expect(container.textContent).toContain(command);
    expect(container.textContent).toContain("ok");
  });

  it("affiche le spinner quand un tool est en cours", () => {
    const { getByTestId } = render(
      <ToolBubble tools={[{ name: "bash", args: { command: "sleep 5" } }]} />,
    );
    expect(getByTestId("spinner")).toBeTruthy();
  });

  it("n'affiche pas d'icône de succès quand tous les tools sont terminés", () => {
    const { queryByTestId } = render(
      <ToolBubble
        tools={[{ name: "bash", args: { command: "ls" }, result: "fichier.txt", isError: false }]}
      />,
    );
    expect(queryByTestId("status-icon-error")).toBeNull();
  });

  it("affiche l'icône d'erreur quand un tool échoue", () => {
    const { container } = render(
      <ToolBubble
        tools={[{ name: "bash", args: { command: "exit 1" }, result: "erreur", isError: true }]}
      />,
    );
    expect(container.querySelector('[data-testid="status-icon-error"]')).toBeTruthy();
  });

  it("affiche ContentPreview après ouverture du tool", () => {
    const { container, getByTestId, queryByTestId } = render(
      <ToolBubble
        tools={[{ name: "write_file", args: { path: "/tmp/foo.ts", content: "const x = 1;" }, result: "ok" }]}
      />,
    );
    expect(queryByTestId("content-preview")).toBeNull();
    openTool(container);
    expect(getByTestId("content-preview")).toBeTruthy();
  });

  it("affiche DiffPreview après ouverture du groupe puis du tool", () => {
    const { container, getByTestId, queryByTestId } = render(
      <ToolBubble
        tools={[{
          name: "edit_file",
          args: { path: "/tmp/foo.ts", old_string: "const x = 1;", new_string: "const x = 2;" },
          result: "ok",
        }]}
      />,
    );
    expect(queryByTestId("diff-preview")).toBeNull();
    openTool(container);
    expect(getByTestId("diff-preview")).toBeTruthy();
  });

  it("n'affiche pas ContentPreview pour write_file précédé d'edit_file sur le même path", () => {
    const { container, queryByTestId } = render(
      <ToolBubble
        tools={[
          { name: "edit_file", args: { path: "/tmp/foo.ts", old_string: "a", new_string: "b" }, result: "ok" },
          { name: "write_file", args: { path: "/tmp/foo.ts", content: "const x = 2;" }, result: "ok" },
        ]}
      />,
    );
    openGroup(container);
    expect(queryByTestId("content-preview")).toBeNull();
  });

  it("affiche WebResultsPreview après ouverture du tool", () => {
    const { container, getByTestId, queryByTestId } = render(
      <ToolBubble
        tools={[{ name: "web_search", args: { query: "vitest jsdom" }, result: "{\"results\":[]}" }]}
      />,
    );
    expect(queryByTestId("web-preview")).toBeNull();
    openTool(container);
    expect(getByTestId("web-preview")).toBeTruthy();
  });

  it("garde la preview d'un tool unique fermée par défaut", () => {
    const { container } = render(
      <ToolBubble
        tools={[{ name: "write_file", args: { path: "/tmp/foo.ts", content: "const x = 1;" }, result: "ok" }]}
      />,
    );
    expect(container.querySelector(".tb-group-toggle")).toBeNull();
    expect(container.querySelector(".tb-accordion.tb-open")).toBeNull();
  });
});
