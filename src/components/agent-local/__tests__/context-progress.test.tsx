import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { ContextProgress } from "../context-progress";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";
import type { ResolvedContextUsage } from "@/hooks/agent-token-estimate";

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
  vi.useRealTimers();
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

function contextSummary(used: number, max: number): ResolvedContextUsage {
  return {
    used, max, output: null, status: "reconstructed", secondaryStatus: null,
    source: "reconstructed", coverage: "complete", breakdown: null,
  };
}

describe("ContextProgress", () => {
  it("affiche un seul total égal à la somme des catégories", () => {
    const view = render(
      <ContextProgress
        breakdown={{ ...breakdown, used: 8_297 }}
        summary={{
          used: 6_371, max: 258_400, output: 13, status: "measured",
          secondaryStatus: null, source: "provider", coverage: "complete", breakdown: null,
        }}
      />,
    );

    fireEvent.mouseEnter(view.getByLabelText("Context window"));

    expect(view.getByText("8.3K / 258.4K (3.2%)")).toBeTruthy();
    expect(view.getByText("Messages")).toBeTruthy();
    expect(view.queryByText("Last measured input")).toBeNull();
    expect(view.queryByText("Provider measurement")).toBeNull();
    expect(view.queryByText("Estimated breakdown · 8.3K")).toBeNull();
    expect(view.queryByText("Output")).toBeNull();
    expect(view.queryByText("13")).toBeNull();
  });

  it("affiche le panneau détaillé avec les 7 catégories", () => {
    const { getByText, getByLabelText } = render(
      <ContextProgress summary={contextSummary(100, 1000)} breakdown={breakdown} />,
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

  it("affiche le total sans inventer de pourcentage si le maximum est inconnu", () => {
    const { getByLabelText, getByText } = render(
      <ContextProgress breakdown={breakdown} summary={{
        used: 100, max: null, output: null, status: "measured",
        secondaryStatus: null, source: "provider", coverage: "complete", breakdown: null,
      }} />,
    );
    fireEvent.mouseEnter(getByLabelText("Context window"));
    expect(getByText("100")).toBeTruthy();
    expect(document.querySelector(".context-ring-bar")).toBeNull();
  });

  it("utilise les catégories quand la mesure fournisseur est indisponible", () => {
    const { getByLabelText, getByText } = render(
      <ContextProgress breakdown={breakdown} summary={{
        used: null, max: null, output: null, status: "unavailable",
        secondaryStatus: null, source: null, coverage: null, breakdown: null,
      }} />,
    );

    fireEvent.mouseEnter(getByLabelText("Context window"));
    expect(getByText("100")).toBeTruthy();
  });

  it("actualise aussi le panneau détaillé pendant le stream", () => {
    const { getByText, rerender } = render(
      <ContextProgress summary={contextSummary(100, 1000)} breakdown={breakdown} />,
    );
    fireEvent.mouseEnter(document.querySelector(".context-ring") as HTMLElement);
    const liveBreakdown: ContextUsageBreakdown = {
      used: 140,
      items: breakdown.items.map((item) => item.key === "messages"
        ? { ...item, tokens: 90, percentage: 64.3 }
        : { ...item, percentage: (item.tokens / 140) * 100 }),
    };

    rerender(<ContextProgress summary={contextSummary(140, 1000)} breakdown={liveBreakdown} />);

    expect(getByText("140 / 1K (14.0%)")).toBeTruthy();
    expect(getByText("90")).toBeTruthy();
  });

  it("affiche 1M et place le focus dans le panneau activé au clavier", async () => {
    const { getByLabelText, getByRole, getByText } = render(
      <ContextProgress summary={contextSummary(400_000, 1_000_000)} />,
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
        <ContextProgress summary={contextSummary(400, 1_000)} />
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

  it("conserve le panneau ouvert si son délai de fermeture expire inactive", () => {
    vi.useFakeTimers();
    let active = true;
    const view = render(
      <AppSurfaceActivityProvider active={active}>
        <ContextProgress summary={contextSummary(100, 1000)} breakdown={breakdown} />
      </AppSurfaceActivityProvider>,
    );
    const ring = view.getByLabelText("Context window");
    void act(() => fireEvent.mouseEnter(ring));
    expect(view.getByRole("dialog", { name: "Context window" })).toBeInTheDocument();
    void act(() => fireEvent.mouseLeave(ring));

    active = false;
    void act(() => view.rerender(
      <AppSurfaceActivityProvider active={active}>
        <ContextProgress summary={contextSummary(100, 1000)} breakdown={breakdown} />
      </AppSurfaceActivityProvider>,
    ));
    void act(() => vi.advanceTimersByTime(150));
    active = true;
    void act(() => view.rerender(
      <AppSurfaceActivityProvider active={active}>
        <ContextProgress summary={contextSummary(100, 1000)} breakdown={breakdown} />
      </AppSurfaceActivityProvider>,
    ));
    expect(view.getByRole("dialog", { name: "Context window" })).toBeInTheDocument();
  });
});
