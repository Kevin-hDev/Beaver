import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ToolBubble } from "../tool-bubble";
import type { ToolActivity } from "@/hooks/agent-chat-utils";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretUp: () => <span />,
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
      if (key === "agentLocal.toolActivity.groups.command") return "Commands";
      if (key === "agentLocal.toolActivity.counts.commands") {
        const count = typeof opts?.count === "number" ? opts.count : 0;
        return `${count} commands`;
      }
      if (key === "agentLocal.toolActivity.inProgress") return "in progress";
      if (key === "agentLocal.toolActivity.toggleDetails") return "Show tool details";
      if (key === "agentLocal.toolActivity.actions.run") return "Run";
      return key;
    },
  }),
}));
vi.mock("../tool-previews", () => ({
  ContentPreview: () => <div />,
  DiffPreview: () => <div />,
  WebResultsPreview: () => <div />,
}));
vi.mock("../tool-office-previews", () => ({
  ReadSpreadsheetPreview: () => <div />,
  WriteSpreadsheetPreview: () => <div />,
  DocumentResultPreview: () => <div />,
  WriteDocumentPreview: () => <div />,
}));
vi.mock("@/lib/tool-file-path", () => ({ isFileTool: () => false }));
vi.mock("../tool-bubble.css", () => ({}));
vi.mock("../tool-bubble-arrows.css", () => ({}));
vi.mock("../tool-bubble-detail.css", () => ({}));
vi.mock("../tool-bubble-status.css", () => ({}));
vi.mock("../stream-active.css", () => ({}));

describe("ToolBubble active stream item", () => {
  it("anime uniquement la ligne du tool actif", () => {
    const activeTool: ToolActivity = { name: "bash", args: { command: "sleep 1" } };
    const doneTool: ToolActivity = { name: "grep", args: { pattern: "x" }, result: "ok", isError: false };
    const { container } = render(<ToolBubble tools={[activeTool, doneTool]} activeTools={[activeTool]} />);

    expect(container.querySelector(".tb-row.stream-active")).toBeTruthy();
    expect(container.querySelector(".tb-tool-verb.stream-active-label")).toBeTruthy();
  });

  it("anime plusieurs tools actifs en même temps dans un groupe ouvert", () => {
    const firstTool: ToolActivity = { name: "bash", args: { command: "sleep 10" } };
    const secondTool: ToolActivity = { name: "bash", args: { command: "sleep 20" } };
    const doneTool: ToolActivity = { name: "bash", args: { command: "pwd" }, result: "ok", isError: false };
    const { container, getByRole } = render(
      <ToolBubble tools={[firstTool, secondTool, doneTool]} activeTools={[firstTool, secondTool]} />,
    );

    expect(container.querySelector(".tb-group-toggle.stream-active")).toBeTruthy();

    fireEvent.click(getByRole("button", { name: "Show tool details" }));

    expect(container.querySelectorAll(".tb-row.stream-active")).toHaveLength(2);
    expect(container.querySelectorAll(".tb-tool-verb.stream-active-label")).toHaveLength(2);
  });

  it("n'anime aucun tool quand l'outil actif est terminé", () => {
    const doneTool: ToolActivity = { name: "bash", args: { command: "pwd" }, result: "ok", isError: false };
    const { container } = render(<ToolBubble tools={[doneTool]} />);

    expect(container.querySelector(".stream-active-label")).toBeNull();
    expect(container.querySelector(".tb-row.stream-active")).toBeNull();
  });

  it("anime le groupe fermé puis la ligne exacte quand le groupe est ouvert", () => {
    const firstTool: ToolActivity = { name: "bash", args: { command: "pwd" }, result: "ok", isError: false };
    const activeTool: ToolActivity = { name: "bash", args: { command: "sleep 1" } };
    const { container, getByRole } = render(<ToolBubble tools={[firstTool, activeTool]} activeTools={[activeTool]} />);

    expect(container.querySelector(".tb-group-toggle.stream-active")).toBeTruthy();
    expect(container.querySelector(".tb-group-title.stream-active-label")).toBeTruthy();

    fireEvent.click(getByRole("button", { name: "Show tool details" }));

    expect(container.querySelector(".tb-group-toggle.stream-active")).toBeNull();
    expect(container.querySelectorAll(".tb-row.stream-active")).toHaveLength(1);
  });
});
