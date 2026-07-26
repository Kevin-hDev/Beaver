import { describe, it, expect, vi, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/react";
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
      const count = typeof opts?.count === "number" ? String(opts.count) : "";
      if (key === "agentLocal.toolActivity.toggleDetails") return "Show tool details";
      if (key === "agentLocal.toolActivity.groupError") return "An error occurred in this group";
      if (key === "agentLocal.toolActivity.groups.web") return "Web";
      if (key === "agentLocal.toolActivity.counts.webSearches") return `${count} web search`;
      if (key === "agentLocal.toolActivity.counts.webFetches") return `${count} web fetch`;
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
vi.mock("@/lib/tool-file-path", () => ({ isFileTool: () => false }));

describe("ToolBubble error details", () => {
  it("masque le détail brut des erreurs web_fetch", () => {
    const { container, queryByTestId } = render(
      <ToolBubble
        tools={[{
          name: "web_fetch",
          args: { url: "https://example.com/private" },
          result: "HTTP 403 secret_key=abc123456",
          isError: true,
        }]}
      />,
    );

    expect(container.querySelector('[data-testid="status-icon-error"]')).toBeTruthy();
    expect(container.textContent).not.toContain("HTTP 403");
    expect(container.textContent).not.toContain("secret_key");
    expect(container.querySelector(".tb-toggle")).toBeNull();
    expect(container.querySelector(".tb-tool-arrow-spacer")).toBeNull();
    expect(queryByTestId("web-preview")).toBeNull();
  });

  it("nettoie le détail affichable des erreurs web_search", () => {
    const { container } = render(
      <ToolBubble
        tools={[{
          name: "web_search",
          args: { query: "news" },
          result: "SearXNG: secret_key=abc123456 /Users/me/file",
          isError: true,
        }]}
      />,
    );

    expect(container.innerHTML).not.toContain("abc123456");
    expect(container.innerHTML).not.toContain("/Users/me/file");
  });
});
