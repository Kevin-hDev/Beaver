import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ThreadPanel, type ThreadPanelProps } from "../thread-panel";

afterEach(cleanup);

function renderPanel(props: Partial<ThreadPanelProps> = {}) {
  return render(
    <ThreadPanel
      icon={<svg data-testid="icon" />}
      title="3 tâches"
      open={false}
      onToggle={vi.fn()}
      {...props}
    >
      <div className="thp-row">ligne</div>
    </ThreadPanel>,
  );
}

describe("ThreadPanel", () => {
  it("porte le contour commun et la classe de celui qui le pose", () => {
    const { container } = renderPanel({ className: "tdp-panel" });

    expect(container.firstElementChild).toHaveClass("thp-root", "relief", "tdp-panel");
  });

  it("replie par tout l'en-tête, nommé par son texte visible, et annonce son état", () => {
    const onToggle = vi.fn();
    const { container } = renderPanel({ onToggle });
    const toggle = screen.getByRole("button", { name: "3 tâches" });

    expect(toggle).not.toHaveAttribute("aria-label");
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(toggle).toContainElement(screen.getByTestId("icon"));
    expect(container.querySelector(".cps-region")).toHaveAttribute("data-open", "false");
    fireEvent.click(toggle);
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it("ouvert, fait pivoter la flèche et montre les lignes sous l'en-tête", () => {
    const { container } = renderPanel({ open: true });

    expect(screen.getByRole("button", { name: "3 tâches" })).toHaveAttribute("aria-expanded", "true");
    expect(container.querySelector(".thp-chevron")).toHaveClass("thp-chevron-open");
    expect(container.querySelector(".cps-region[data-open='true'] .thp-list")).toHaveTextContent("ligne");
  });

  it("met le contenu d'en-tête dans le bouton et l'action à côté, jamais dedans", () => {
    const { container } = renderPanel({
      headerExtra: <span>33%</span>,
      headerAction: <button type="button">Tout arrêter</button>,
    });
    const toggle = screen.getByRole("button", { name: /^3 tâches.*33%$/ });
    const action = screen.getByRole("button", { name: "Tout arrêter" });

    expect(toggle.contains(action)).toBe(false);
    expect(action.closest(".thp-action")).not.toBeNull();
    expect(container.querySelector(".thp-header")).toHaveClass("thp-has-action");
  });

  it("ne réserve aucune place d'action sans action", () => {
    const { container } = renderPanel();

    expect(container.querySelector(".thp-action")).toBeNull();
    expect(container.querySelector(".thp-header")).not.toHaveClass("thp-has-action");
  });

  it("donne au clavier le fond du survol, à l'en-tête comme aux lignes cliquables", () => {
    const css = readFileSync("src/components/agent-local/thread-panel.css", "utf8");

    expect(css).toMatch(
      /\.thp-header:hover \.thp-toggle,\s*\.thp-toggle:focus-visible\s*\{\s*background:\s*var\(--surface-hover\);/s,
    );
    expect(css).toMatch(
      /\.thp-row-clickable:hover,\s*\.thp-row-clickable:focus-visible\s*\{\s*background:\s*var\(--surface-hover\);/s,
    );
  });
});
