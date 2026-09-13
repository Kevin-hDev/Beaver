import type { ReactNode } from "react";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SubagentBubble } from "../subagent-bubble";
import type { SubagentInfo } from "@/types/agent";

afterEach(() => cleanup());

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const count = typeof opts?.count === "number" || typeof opts?.count === "string" ? opts.count : 0;
      if (key === "subagents.bubbleLabel") return `${count} agents créés`;
      return key;
    },
  }),
}));

vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ label, children }: { label: string; children: ReactNode }) => (
    <span data-tooltip={label}>{children}</span>
  ),
}));

describe("SubagentBubble", () => {
  it("affiche les identités produit et les couleurs dédiées dans le panneau commun", () => {
    const { container } = render(
      <SubagentBubble
        subagents={[
          agent("coder", "Audit code", "claudiator"),
          agent("explorer", "Audit web", "geminitor"),
        ]}
        onOpen={vi.fn()}
      />,
    );

    const toggle = screen.getByRole("button", { name: /2 agents créés/ });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(container.querySelector(".sb-root")).toHaveClass("thp-root", "relief", "chat-column-surface");
    fireEvent.click(toggle);
    expect(screen.getByText("Claudiator")).toBeTruthy();
    expect(screen.getByText("Geminitor")).toBeTruthy();
    expect(screen.getByText("Audit code")).toBeTruthy();
    expect(container.querySelector(".sai-claudiator")).toBeTruthy();
    expect(container.querySelector(".sai-geminitor")).toBeTruthy();
    expect(container.querySelector(".sai-running")).toBeNull();
  });

  it("ouvre la conversation du sous-agent quand on clique sa ligne, sous l'infobulle « Ouvrir »", () => {
    const onOpen = vi.fn<(sessionId: string) => void>();
    render(<SubagentBubble subagents={[agent("coder", "Audit code", "claudiator")]} onOpen={onOpen} />);

    fireEvent.click(screen.getByRole("button", { name: /1 agents créés/ }));
    const row = screen.getByRole("button", { name: /Claudiator/ });
    fireEvent.click(row);

    expect(onOpen).toHaveBeenCalledWith("coder-child");
    expect(row.closest("[data-tooltip]")?.getAttribute("data-tooltip")).toBe("subagents.open");
  });
});

function agent(type: "explorer" | "coder", description: string, colorKey: string): SubagentInfo {
  return {
    sessionId: `${type}-child`,
    name: type,
    type,
    status: "completed",
    promptPreview: "",
    description,
    colorKey,
  };
}
