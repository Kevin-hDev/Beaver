import { cleanup, render } from "@testing-library/react";
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
});
