import { render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FilePreviewPlan } from "../file-preview-plan";

const readFilePreview = vi.fn<(path: string, baseDir?: string) => Promise<string>>();

vi.mock("@/services/file-preview", () => ({
  readFilePreview: (path: string, baseDir?: string) => readFilePreview(path, baseDir),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("@/components/agent-local/chat-markdown", () => ({
  ChatMarkdown: ({ content }: { content: string }) => (
    <div data-testid="chat-markdown">{content}</div>
  ),
}));

describe("FilePreviewPlan", () => {
  it("lit le plan et le rend avec le markdown du chat", async () => {
    readFilePreview.mockResolvedValue("# Plan\n\n- Task");

    const { getByTestId } = render(
      <FilePreviewPlan
        operation={{
          id: "plan:p1",
          path: "/tmp/p1.md",
          name: "Plan",
          type: "read",
          kind: "plan",
          timestamp: "",
          additions: 0,
          deletions: 0,
        }}
      />,
    );

    await waitFor(() => expect(getByTestId("chat-markdown").textContent).toContain("# Plan"));
    expect(readFilePreview).toHaveBeenCalledWith("/tmp/p1.md", undefined);
  });
});
