import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { existsSync, readFileSync } from "node:fs";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SubagentAccordion } from "../subagent-accordion";
import type { SubagentInfo } from "@/types/agent";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      if (key === "subagents.backgroundCount") {
        const count = typeof opts?.count === "number" || typeof opts?.count === "string"
          ? opts.count
          : "";
        return `${count} sous-agents actifs`;
      }
      if (key === "subagents.running") return "en cours...";
      if (key === "subagents.stopAll") return "Tout arrêter";
      if (key === "subagents.stop") return "Arrêter";
      if (key === "subagents.open") return "Ouvrir";
      return key;
    },
  }),
}));

vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ label, children }: { label: string; children: ReactNode }) => (
    <span data-tooltip={label}>{children}</span>
  ),
}));

vi.mock("@/components/ui/icons", () => ({
  ChevronDown: (props: Record<string, unknown>) => <span data-testid="chevron" {...props} />,
  Square: (props: Record<string, unknown>) => <span data-testid="square" {...props} />,
}));

function renderPanel(subagents: SubagentInfo[], onCancel = vi.fn(), onOpen = vi.fn()) {
  return render(<SubagentAccordion subagents={subagents} onCancel={onCancel} onOpen={onOpen} />);
}

describe("SubagentAccordion", () => {
  it("ne rend rien sans sous-agent", () => {
    const { container } = renderPanel([]);
    expect(container.innerHTML).toBe("");
  });

  it("donne une ligne par sous-agent, sans « en cours »", () => {
    const { container } = renderPanel([agent("coder", "Audit subagents long"), agent("explorer", "Audit web")]);

    expect(screen.getByText("Claudiator")).toHaveClass("thp-name");
    expect(screen.getByText("Geminitor")).toBeInTheDocument();
    expect(screen.getByText("Audit subagents long")).toHaveClass("thp-muted");
    expect(screen.queryByText("en cours...")).toBeNull();
    expect(container.querySelectorAll(".sa-row")).toHaveLength(2);
    expect(container.querySelector(".sai-claudiator.sai-running")).toBeTruthy();
    expect(container.querySelector(".sai-geminitor.sai-running")).toBeTruthy();
  });

  it("ouvre le sous-agent par toute la ligne, Arrêter reste un bouton voisin", () => {
    const onCancel = vi.fn();
    const onOpen = vi.fn();
    const { container } = renderPanel([agent("coder", "Audit")], onCancel, onOpen);
    const open = container.querySelector(".sa-row-open") as HTMLButtonElement;
    const stop = screen.getByRole("button", { name: "Arrêter" });

    expect(open).toHaveClass("thp-row", "thp-row-clickable");
    expect(open.closest("[data-tooltip]")).toHaveAttribute("data-tooltip", "Ouvrir");
    expect(open.contains(stop)).toBe(false);
    fireEvent.click(open);
    expect(onOpen).toHaveBeenCalledWith("coder-child");
    fireEvent.click(stop);
    expect(onCancel).toHaveBeenCalledWith("coder-child");
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("colle le chronomètre à la fin de la ligne, juste avant Arrêter", () => {
    vi.useFakeTimers();
    vi.setSystemTime(10_000);
    const { container } = renderPanel([{ ...agent("explorer", "Audit"), spawnedAt: 1_000 }]);
    act(() => { vi.advanceTimersByTime(1_000); });

    const timer = container.querySelector(".sa-row-timer");
    expect(timer).toHaveTextContent("10s");
    expect(container.querySelector(".sa-row-open")?.lastElementChild).toBe(timer);
  });

  it("pose un en-tête au visage immobile, qui replie, et « Tout arrêter » avant la flèche", () => {
    const onCancel = vi.fn();
    const { container } = renderPanel([agent("coder", "A"), agent("explorer", "B")], onCancel);
    const toggle = screen.getByRole("button", { name: "2 sous-agents actifs" });
    const stopAll = screen.getByRole("button", { name: "Tout arrêter" });

    expect(container.firstElementChild).toHaveClass("thp-root", "relief", "sa-panel");
    expect(toggle.querySelector(".thp-icon circle[r='9.5']")).toBeTruthy();
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(toggle.contains(stopAll)).toBe(false);
    expect(stopAll.closest(".thp-action")).not.toBeNull();
    fireEvent.click(stopAll);
    expect(onCancel).toHaveBeenCalledTimes(2);
    fireEvent.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "false");
  });

  it("n'a qu'une feuille, sans contour propre ni ligne qui se replie en conversation étroite", () => {
    const css = readFileSync("src/components/agent-local/subagent-accordion.css", "utf8");
    expect(existsSync("src/components/agent-local/subagent-accordion-controls.css")).toBe(false);
    expect(css).not.toMatch(/border\s*:/);
    expect(css).not.toContain("@container");
    expect(css).not.toContain("@keyframes");
  });

  it("donne à Arrêter et « Tout arrêter » au clavier leur apparence de survol, dans ce panneau seulement", () => {
    const css = readFileSync("src/components/agent-local/subagent-accordion.css", "utf8");
    expect(css).toMatch(
      /\.sa-stop:hover:not\(:disabled\),\s*\.sa-stop:focus-visible\s*\{\s*color:\s*var\(--signal-error\);\s*background:\s*var\(--surface-hover\);/s,
    );
    const selectors = css.replace(/\/\*[\s\S]*?\*\//g, "").match(/[^{}]+(?=\{)/g) ?? [];
    const focusSelectors = selectors.flatMap((selector) => selector.split(","))
      .map((selector) => selector.trim()).filter((selector) => selector.includes(":focus-visible"));
    expect(focusSelectors).toEqual([".sa-stop:focus-visible"]);
  });

  it("garde le fond de la ligne sous Arrêter ; le clavier le reçoit de .thp-row-clickable", () => {
    const css = readFileSync("src/components/agent-local/subagent-accordion.css", "utf8");
    expect(css).toMatch(/\.sa-row:hover \.sa-row-open\s*\{\s*background:\s*var\(--surface-hover\);/s);
  });
});

function agent(type: "explorer" | "coder", name: string): SubagentInfo {
  return { sessionId: `${type}-child`, name, type, status: "running", promptPreview: "" };
}
