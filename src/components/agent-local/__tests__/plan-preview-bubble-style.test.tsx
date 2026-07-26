import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PlanPreviewBubble } from "../plan-preview-bubble";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  ClipboardText: () => <span data-testid="clipboard-icon" />,
}));

vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));

vi.mock("../chat-markdown", () => ({
  ChatMarkdown: ({ content }: { content: string }) => <div>{content}</div>,
}));

describe("PlanPreviewBubble CSS", () => {
  it("rend la preview avec son wrapper de bubble dédié", () => {
    const { container, getByText } = render(
      <PlanPreviewBubble
        plan={{
          id: "plan-1",
          title: "Plan dashboard",
          content: "Objectif\n\nConstruire le dashboard",
          status: "awaiting_approval",
        }}
      />,
    );

    const root = container.querySelector(".ppb-root");
    expect(root).not.toBeNull();
    expect(root).toHaveClass("chat-bubble", "chat-column-surface");
    expect(getByText("Plan dashboard")).toBeInTheDocument();
    expect(root?.textContent).toContain("Objectif");
  });
});
