import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { SavedToolBubble, ToolBubble } from "../tool-bubble";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretUp: () => <span />,
  Check: () => <span data-testid="check-icon" />,
  Copy: () => <span />,
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
      const rawCount = opts?.count;
      const count = typeof rawCount === "string" || typeof rawCount === "number" ? String(rawCount) : "";
      if (key === "agentLocal.toolActivity.toggleDetails") return "Show tool details";
      if (key === "agentLocal.toolActivity.groupError") return "An error occurred in this group";
      if (key === "agentLocal.toolActivity.groups.exploration") return "Exploration";
      if (key === "agentLocal.toolActivity.counts.files") return `${count} file read`;
      if (key === "agentLocal.toolActivity.actions.read") return "Read";
      return key;
    },
  }),
}));

vi.mock("@/lib/tool-file-path", () => ({
  isFileTool: (name: string) => name === "read_file",
}));

function openTool(container: HTMLElement) {
  const toggle = container.querySelector(".tb-toggle");
  if (!toggle) throw new Error("tool toggle absent");
  fireEvent.click(toggle);
}

describe("read_file result bubble", () => {
  it("affiche le résultat live dans une bulle de code colorée", () => {
    const content = "export const value = 1;";
    const { container } = render(
      <ToolBubble tools={[{ name: "read_file", args: { path: "/tmp/a.ts" }, result: content }]} />,
    );

    openTool(container);

    expect(container.querySelector(".tb-result-md")).toHaveClass("chat-column-surface");
    expect(container.querySelector(".tb-result-code")?.textContent).toBe(content);
    expect(container.querySelector(".hljs-keyword")).not.toBeNull();
    expect(container.querySelector(".tb-result-preview")).toBeNull();
  });

  it("affiche le résultat sauvegardé dans une bulle de code colorée", () => {
    const content = "export const value = 1;";
    const { container } = render(
      <SavedToolBubble tools={[{ name: "read_file", summary: "/tmp/a.ts", result: content }]} />,
    );

    openTool(container);

    expect(container.querySelector(".tb-result-md")).toHaveClass("chat-column-surface");
    expect(container.querySelector(".tb-result-code")?.textContent).toBe(content);
    expect(container.querySelector(".hljs-keyword")).not.toBeNull();
    expect(container.querySelector(".tb-result-preview")).toBeNull();
  });
});
