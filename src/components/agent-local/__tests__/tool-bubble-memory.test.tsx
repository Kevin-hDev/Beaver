import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
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
      const count = typeof opts?.count === "number" ? opts.count : 0;
      if (key === "agentLocal.toolActivity.groups.memory") return "MEMORY";
      if (key === "agentLocal.toolActivity.counts.files") return `${count} file`;
      if (key === "agentLocal.toolActivity.inProgress") return "in progress";
      if (key === "agentLocal.toolActivity.completed") return "completed";
      if (key === "agentLocal.toolActivity.toggleDetails") return "Toggle memory";
      if (key === "agentLocal.toolActivity.actions.read") return "Read";
      return key;
    },
  }),
}));
vi.mock("../tool-previews", () => ({
  ContentPreview: () => <div data-testid="content-preview" />,
  DiffPreview: () => <div data-testid="diff-preview" />,
  WebResultsPreview: () => <div />,
}));
vi.mock("../tool-office-previews", () => ({
  DocumentResultPreview: () => <div />,
  ReadSpreadsheetPreview: () => <div />,
  WriteDocumentPreview: () => <div />,
  WriteSpreadsheetPreview: () => <div />,
}));
vi.mock("../tool-bubble.css", () => ({}));

const displayPath = "/memory/global/topics/preference.md";
const resolvedPath = "/Users/test/.local/share/cl-go-dash/memory/global/topics/preference.md";

function memoryTool(result?: string): ToolActivity {
  return {
    name: "read_file",
    args: { path: displayPath },
    domain: "memory",
    resolvedPath,
    result,
    isError: result === undefined ? undefined : false,
  };
}

describe("ToolBubble MEMORY", () => {
  it("garde la même bulle ouverte pendant le passage de en cours à terminé", () => {
    const active = memoryTool();
    const view = render(<ToolBubble tools={[active]} activeTools={[active]} />);
    const toggle = view.getByRole("button", { name: "Toggle memory" });

    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(view.getByText("in progress")).toBeTruthy();
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");

    view.rerender(<ToolBubble tools={[memoryTool("# Préférence")]} />);

    expect(view.getByRole("button", { name: "Toggle memory" })).toBe(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(view.getByText("completed")).toBeTruthy();
    expect(view.getByText("preference.md")).toBeTruthy();
  });

  it("ouvre le chemin résolu sans replier la bulle", () => {
    const onFilePreview = vi.fn();
    const view = render(
      <ToolBubble tools={[memoryTool("# Préférence")]} onFilePreview={onFilePreview} />,
    );
    const toggle = view.getByRole("button", { name: "Toggle memory" });
    fireEvent.click(toggle);
    fireEvent.click(view.getByText("preference.md"));

    expect(onFilePreview).toHaveBeenCalledWith(resolvedPath);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
  });
});
