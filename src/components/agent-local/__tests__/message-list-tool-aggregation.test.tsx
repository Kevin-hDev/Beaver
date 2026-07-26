import { describe, it, expect, vi, afterEach } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { MessageList, SegmentedAssistantMessage } from "../message-list";
import { assistantWithSegmentTools } from "../test-utils/message-list-tool-aggregation-fixtures";

afterEach(cleanup);

vi.mock("@phosphor-icons/react", () => ({
  Spinner: () => <span data-testid="spinner" />,
}));
vi.mock("@/components/ui/icons", () => ({
  Copy: () => <span />, CaretDown: () => <span />, CaretUp: () => <span />,
  CaretRight: () => <span />,
  Check: () => <span data-testid="check-icon" />, ClipboardText: () => <span />,
  Brain: () => <span data-testid="brain-icon" />,
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
vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));
vi.mock("@/hooks/use-hover-class", () => ({ useHoverClass: () => ({ current: null }) }));
vi.mock("../working-stats", () => ({
  LoadingIndicator: () => <div data-testid="loading-indicator" />,
}));
vi.mock("../compression-indicator", () => ({
  CompressionIndicator: () => <div data-testid="compression-indicator" />,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "en" },
    t: (key: string, opts?: Record<string, unknown>) => {
      const text = (value: unknown) => (
        typeof value === "string" || typeof value === "number" ? String(value) : ""
      );
      const count = text(opts?.count);
      if (key === "agentLocal.toolActivity.toggleDetails") return "Show tool details";
      if (key === "agentLocal.toolActivity.groupError") return "An error occurred in this group";
      if (key === "agentLocal.toolActivity.groups.command") return "Commands";
      if (key === "agentLocal.toolActivity.groups.modification") return "Changes";
      if (key === "agentLocal.toolActivity.groups.exploration") return "Exploration";
      if (key === "agentLocal.toolActivity.counts.files") return `${count} file read`;
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
vi.mock("@/lib/tool-file-path", () => ({
  isFileTool: (name: string) => ["write_file", "edit_file"].includes(name),
  inferSavedToolPaths: <T,>(tools: T[]) => tools,
}));
vi.mock("../chat.css", () => ({}));
vi.mock("../messages.css", () => ({}));
vi.mock("../tool-bubble.css", () => ({}));

describe("MessageList tool aggregation", () => {
  it("garde un flux compact par segment sauvegardé pour préserver l'ordre", () => {
    const { container } = render(
      <SegmentedAssistantMessage
        msg={assistantWithSegmentTools()}
        tps={0}
        totalElapsedMs={0}
      />,
    );

    const streams = container.querySelectorAll(".tb-stream");
    expect(streams).toHaveLength(2);
    expect(streams[0].textContent).toContain("bash");
    expect(streams[0].textContent).toContain("npm test");
    expect(streams[0].textContent).not.toContain("Commands");
    expect(streams[0].textContent).toContain("Changes");
    expect(streams[1].querySelector(".tb-group-toggle")).toBeNull();
    expect(streams[1].textContent).toContain("bash");
    expect(streams[1].textContent).toContain("npm run build");
    expect(streams[0].textContent).not.toContain("npm run build");

    fireEvent.click(streams[0].querySelectorAll(".tb-group-toggle")[0]);
    expect(streams[0].textContent).not.toContain("npm run build");
  });

  it("garde les tools du stream sous le segment qui les a produits", () => {
    const { container } = render(
      <MessageList
        messages={[]}
        completedSegments={[
          {
            thinking: "first reflection",
            content: "",
            tools: [
              { name: "bash", args: { command: "npm test" }, result: "ok" },
              { name: "write_file", args: { path: "a.ts", content: "x" }, result: "ok" },
            ],
          },
          {
            thinking: "",
            content: "",
            tools: [{ name: "read_file", args: { path: "a.ts" }, result: "contents" }],
          },
          { thinking: "second reflection", content: "", tools: [{ name: "bash", args: { command: "npm run build" }, result: "ok" }] },
        ]}
        currentContent=""
        currentThinking=""
        currentTools={[{ name: "edit_file", args: { path: "b.ts", old_string: "a", new_string: "b" }, result: "ok" }]}
        isStreaming
        isCompressing={false}
        tps={0}
        totalElapsedMs={0}
        segmentStartedAt={1}
        liveTokenCount={0}
      />,
    );

    const streams = container.querySelectorAll(".tb-stream");
    expect(streams).toHaveLength(2);
    expect(streams[0].querySelector(".tb-group-toggle")).toBeNull();
    expect(streams[0].textContent).toContain("npm test");
    expect(streams[0].textContent).toContain("a.ts");
    expect(streams[0].textContent).not.toContain("npm run build");
    expect(streams[0].textContent).not.toContain("b.ts");
    expect(streams[1].querySelector(".tb-group-toggle")).toBeNull();
    expect(streams[1].textContent).toContain("npm run build");
    expect(streams[1].textContent).toContain("b.ts");
  });

  it("affiche la preview du plan après la timeline du stream", () => {
    const { container } = render(
      <MessageList
        messages={[]}
        completedSegments={[
          {
            thinking: "",
            content: "",
            tools: [{ name: "grep", args: { pattern: "Plan" }, result: "ok" }],
          },
        ]}
        currentContent=""
        currentThinking="thinking before plan"
        currentTools={[]}
        isStreaming
        isCompressing={false}
        tps={0}
        totalElapsedMs={0}
        segmentStartedAt={1}
        liveTokenCount={0}
        planPreview={{
          id: "plan-1",
          title: "Plan validé",
          content: "## Étapes\n\n- Implémenter",
          status: "awaiting_approval",
        }}
      />,
    );

    const text = container.textContent ?? "", planIndex = text.indexOf("Plan validé");
    expect(text.indexOf("Exploration")).toBeLessThan(planIndex);
    expect(text.indexOf("thinking before plan")).toBeLessThan(planIndex);
  });
});
