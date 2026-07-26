import { describe, expect, it, vi, afterEach } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { StreamEndArtifacts } from "../stream-end-artifacts";
import type { AgentMessage } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";

afterEach(cleanup);

vi.mock("@/components/file-preview/file-icon", () => ({
  FileIcon: ({ name }: { name: string }) => <span data-testid={`file-icon-${name}`} />,
}));

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span data-testid="caret-down" />,
  CaretRight: () => <span data-testid="caret-right" />,
}));

vi.mock("../assistant-message", () => ({
  AssistantMessage: ({ content }: { content: string }) => (
    <div data-testid="assistant-message">{content}</div>
  ),
}));

vi.mock("../message-tool-timeline", () => ({
  SavedToolTimeline: () => <div data-testid="saved-timeline" />,
  StreamToolTimeline: () => <div data-testid="stream-timeline" />,
}));

vi.mock("../working-stats", () => ({
  LoadingIndicator: () => <div data-testid="loading-indicator" />,
}));

vi.mock("../compression-indicator", () => ({
  CompressionIndicator: () => <div data-testid="compression-indicator" />,
}));

vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const count = typeof opts?.count === "number" || typeof opts?.count === "string" ? String(opts.count) : "";
      const name = typeof opts?.name === "string" ? opts.name : "";
      if (key === "agentLocal.fileChanges.changed") return `${count} files changed`;
      if (key === "agentLocal.fileChanges.toggle") return "Show changed files";
      if (key === "agentLocal.fileChanges.review") return "Review";
      if (key === "agentLocal.fileChanges.reviewFile") return `Review ${name}`;
      return key;
    },
  }),
}));

describe("FileChangeBubble", () => {
  it("affiche les fichiers modifiés sous une réponse assistant", () => {
    render(
      <StreamEndArtifacts
        messages={[assistant([
          { name: "write_file", summary: "/repo/src/a.ts", content: "one\ntwo" },
          { name: "edit_file", summary: "/repo/src/b.ts", old_text: "old", new_text: "new" },
        ])]}
        projectPath="/repo"
        knownSubagents={[]}
      />,
    );

    expect(screen.getByText("2 files changed")).toBeTruthy();
    expect(screen.queryByText("a.ts")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Show changed files" }));

    expect(screen.getByText("a.ts")).toBeTruthy();
    expect(screen.getByText("b.ts")).toBeTruthy();
    expect(screen.getAllByText("Review")).toHaveLength(2);
  });

  it("n'affiche rien si la réponse ne modifie aucun fichier", () => {
    const { container } = render(
      <StreamEndArtifacts
        messages={[assistant([{ name: "read_file", summary: "/repo/src/a.ts", result: "ok" }])]}
        projectPath="/repo"
        knownSubagents={[]}
      />,
    );

    expect(container.querySelector(".fcb-root")).toBeNull();
  });

  it("affiche directement le fichier s'il est seul", () => {
    render(
      <StreamEndArtifacts
        messages={[assistant([{ name: "write_file", summary: "/repo/src/a.ts", content: "one" }])]}
        projectPath="/repo"
        knownSubagents={[]}
      />,
    );

    expect(screen.getByText("a.ts")).toBeTruthy();
    expect(screen.getByText("+1")).toBeTruthy();
    expect(document.querySelector(".fcb-root")).toHaveClass("chat-column-surface");
    expect(screen.getByRole("button", { name: "Review a.ts" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Show changed files" })).toBeNull();
  });

  it("ouvre le review avec l'opération du fichier concerné", () => {
    const onReview = vi.fn<(operation: FileOperation) => void>();
    render(
      <StreamEndArtifacts
        messages={[assistant([{ name: "edit_file", summary: "/repo/src/b.ts", old_text: "old", new_text: "new" }])]}
        projectPath="/repo"
        onFileReview={onReview}
        knownSubagents={[]}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Review b.ts" }));

    expect(onReview).toHaveBeenCalledWith(expect.objectContaining({
      path: "/repo/src/b.ts",
      type: "edit",
      oldText: "old",
      newText: "new",
    }));
  });
});

function assistant(tool_activities: AgentMessage["tool_activities"]): AgentMessage {
  return {
    id: "assistant-1",
    role: "assistant",
    content: "Done",
    files: [],
    timestamp: "2026-07-04T10:00:00Z",
    tool_activities,
  };
}
