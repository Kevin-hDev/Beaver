import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ReasoningSelector } from "../reasoning-selector";
import type { AvailableModel } from "@/hooks/use-available-models";
import type { ReasoningMode } from "@/lib/reasoning-modes";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const labels: Record<string, string> = {
        "agentLocal.reasoningTitle": "Réflexion",
        "agentLocal.reasoningOff": "Désactivée",
        "agentLocal.reasoningAuto": "Activée",
        "agentLocal.reasoningMedium": "Moyenne",
        "agentLocal.reasoningHigh": "Forte",
      };
      return labels[key] ?? key;
    },
  }),
}));

afterEach(cleanup);

function model(overrides: Partial<AvailableModel> = {}): AvailableModel {
  return {
    id: "gpt-5",
    provider_id: "openai",
    provider_name: "OpenAI",
    is_local: false,
    supports_tools: true,
    supports_thinking: true,
    ...overrides,
  };
}

function renderSelector(
  overrides: Partial<AvailableModel> = {},
  reasoningMode: ReasoningMode = "high",
  onChange = vi.fn(),
) {
  return render(
    <ReasoningSelector
      model={model(overrides)}
      reasoningMode={reasoningMode}
      onChange={onChange}
      align="right"
    />,
  );
}

describe("ReasoningSelector", () => {
  it("reste masqué pour un modèle sans réflexion", () => {
    const { container } = renderSelector({ supports_thinking: false });

    expect(container.firstChild).toBeNull();
  });

  it("affiche le niveau dans un bouton séparé", () => {
    renderSelector();

    expect(screen.getByRole("button", { name: /Forte/ })).toBeTruthy();
  });

  it("propose uniquement les niveaux acceptés par le modèle", () => {
    renderSelector({ reasoning_modes: ["off", "high"] });

    fireEvent.click(screen.getByRole("button", { name: /Forte/ }));

    expect(screen.getByRole("button", { name: "Désactivée" })).toBeTruthy();
    expect(screen.getAllByText("Forte").length).toBeGreaterThan(0);
    expect(screen.queryByText("Moyenne")).toBeNull();
  });

  it("transmet le nouveau niveau choisi", () => {
    const onChange = vi.fn();
    renderSelector({ reasoning_modes: ["off", "high"] }, "high", onChange);

    fireEvent.click(screen.getByRole("button", { name: /Forte/ }));
    fireEvent.click(screen.getByRole("button", { name: "Désactivée" }));

    expect(onChange).toHaveBeenCalledWith("off");
  });
});
