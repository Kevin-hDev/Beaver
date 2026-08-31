import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ContextProgress } from "../context-progress";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const labels: Record<string, string> = {
        "agentLocal.contextUsage.title": "Context window",
        "agentLocal.contextUsage.categories.messages": "Messages",
        "agentLocal.contextUsage.categories.systemTools": "System tools",
        "agentLocal.contextUsage.categories.mcpConnectors": "MCP / connectors",
        "agentLocal.contextUsage.categories.skills": "Skills",
        "agentLocal.contextUsage.categories.memory": "Memory",
        "agentLocal.contextUsage.categories.metaContext": "Meta context",
        "agentLocal.contextUsage.categories.systemPrompt": "System prompt",
      };
      return labels[key] ?? key;
    },
  }),
}));

vi.mock("../context-progress.css", () => ({}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const breakdown: ContextUsageBreakdown = {
  used: 100,
  items: [
    { key: "messages", tokens: 50, percentage: 50 },
    { key: "systemTools", tokens: 20, percentage: 20 },
    { key: "mcpConnectors", tokens: 10, percentage: 10 },
    { key: "skills", tokens: 8, percentage: 8 },
    { key: "memory", tokens: 0, percentage: 0 },
    { key: "metaContext", tokens: 7, percentage: 7 },
    { key: "systemPrompt", tokens: 5, percentage: 5 },
  ],
};

describe("ContextProgress", () => {
  it("affiche le panneau détaillé avec les 7 catégories", () => {
    const { getByText, getByLabelText } = render(
      <ContextProgress used={100} max={1000} breakdown={breakdown} />,
    );

    expect(getByLabelText("Context window")).toBeTruthy();
    fireEvent.mouseEnter(getByLabelText("Context window"));
    expect(getByText("Messages")).toBeTruthy();
    expect(getByText("System tools")).toBeTruthy();
    expect(getByText("MCP / connectors")).toBeTruthy();
    expect(getByText("Skills")).toBeTruthy();
    expect(getByText("Memory")).toBeTruthy();
    expect(getByText("Meta context")).toBeTruthy();
    expect(getByText("System prompt")).toBeTruthy();
  });

  it("ne rend rien si le maximum est inconnu", () => {
    const { container } = render(<ContextProgress used={100} max={0} breakdown={breakdown} />);

    expect(container.firstChild).toBeNull();
  });

  it("actualise aussi le panneau détaillé pendant le stream", () => {
    const { getByText, rerender } = render(
      <ContextProgress used={100} max={1000} breakdown={breakdown} />,
    );
    fireEvent.mouseEnter(document.querySelector(".context-ring") as HTMLElement);
    const liveBreakdown: ContextUsageBreakdown = {
      used: 140,
      items: breakdown.items.map((item) => item.key === "messages"
        ? { ...item, tokens: 90, percentage: 64.3 }
        : { ...item, percentage: (item.tokens / 140) * 100 }),
    };

    rerender(<ContextProgress used={140} max={1000} breakdown={liveBreakdown} />);

    expect(getByText("140 / 1K (14.0%)")).toBeTruthy();
    expect(getByText("90")).toBeTruthy();
  });

  it("affiche 1M et place le focus dans le panneau activé au clavier", async () => {
    const { getByLabelText, getByRole, getByText } = render(
      <ContextProgress used={400_000} max={1_000_000} />,
    );
    const trigger = getByLabelText("Context window");

    fireEvent.keyDown(trigger, { key: "Enter" });

    expect(getByText(/400K \/ 1M/)).toBeTruthy();
    await waitFor(() => expect(getByRole("dialog", { name: "Context window" })).toHaveFocus());
  });

  it("laisse Tab suivre l'ordre normal sans piéger le focus dans le panneau", async () => {
    const user = userEvent.setup();
    const { getByLabelText, getByRole } = render(
      <>
        <ContextProgress used={400} max={1_000} />
        <button type="button">Après l’anneau</button>
      </>,
    );
    const trigger = getByLabelText("Context window");
    await user.tab();
    expect(trigger).toHaveFocus();
    expect(getByRole("dialog", { name: "Context window" })).toBeInTheDocument();

    await user.tab();

    expect(getByRole("button", { name: "Après l’anneau" })).toHaveFocus();
  });
});
