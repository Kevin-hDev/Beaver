import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
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
        "agentLocal.fastMode": "Rapide",
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
  fastModeEnabled = false,
  fastModePending = false,
  onFastModeChange = vi.fn(),
) {
  return render(
    <ReasoningSelector
      model={model(overrides)}
      reasoningMode={reasoningMode}
      onChange={onChange}
      fastModeEnabled={fastModeEnabled}
      fastModePending={fastModePending}
      onFastModeChange={onFastModeChange}
      align="right"
    />,
  );
}

describe("ReasoningSelector", () => {
  it("reste masqué pour un modèle sans réflexion", () => {
    const { container } = renderSelector({ supports_thinking: false });

    expect(container.firstChild).toBeNull();
  });

  it("reste masqué lorsque le catalogue n'autorise aucun niveau", () => {
    const { container } = renderSelector({ reasoning_modes: [] });

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

  it("place Rapide en tête sans prix ni promesse de vitesse", () => {
    renderSelector({ supports_fast_mode: true, reasoning_modes: ["off", "high"] });

    fireEvent.click(screen.getByRole("button", { name: /Forte/ }));

    const fastSwitch = screen.getByRole("switch", { name: "Rapide" });
    const firstReasoning = screen.getByRole("button", { name: "Désactivée" });
    expect(screen.getAllByRole("switch")).toHaveLength(1);
    expect(fastSwitch).not.toBeChecked();
    expect(fastSwitch.compareDocumentPosition(firstReasoning) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(screen.getByText("Rapide").closest("div")?.nextElementSibling).not.toBeNull();
    expect(screen.queryByText(/1\.5|2\.5|[$€]|crédit|credit/i)).toBeNull();
  });

  it("masque Rapide pour un modèle incompatible", () => {
    renderSelector({ supports_fast_mode: false });

    fireEvent.click(screen.getByRole("button", { name: /Forte/ }));

    expect(screen.queryByRole("switch", { name: "Rapide" })).toBeNull();
  });

  it("active Rapide sans fermer le menu ni changer le raisonnement", () => {
    const onReasoningChange = vi.fn();
    const onFastModeChange = vi.fn();
    renderSelector(
      { supports_fast_mode: true },
      "high",
      onReasoningChange,
      false,
      false,
      onFastModeChange,
    );

    fireEvent.click(screen.getByRole("button", { name: /Forte/ }));
    fireEvent.click(screen.getByRole("switch", { name: "Rapide" }));

    expect(onFastModeChange).toHaveBeenCalledWith(true);
    expect(onReasoningChange).not.toHaveBeenCalled();
    expect(screen.getByRole("switch", { name: "Rapide" })).toBeTruthy();
  });

  it("désactive la bascule pendant la sauvegarde", () => {
    renderSelector({ supports_fast_mode: true }, "high", vi.fn(), true, true);

    fireEvent.click(screen.getByRole("button", { name: /Forte/ }));

    expect(screen.getByRole("switch", { name: "Rapide" })).toBeDisabled();
  });

  it("s'active au clavier et garde le menu ouvert", async () => {
    const user = userEvent.setup();
    const onFastModeChange = vi.fn();

    function ControlledSelector() {
      const [enabled, setEnabled] = useState(false);
      return (
        <ReasoningSelector
          model={model({ supports_fast_mode: true })}
          reasoningMode="high"
          onChange={vi.fn()}
          fastModeEnabled={enabled}
          fastModePending={false}
          onFastModeChange={(nextEnabled) => {
            onFastModeChange(nextEnabled);
            setEnabled(nextEnabled);
          }}
        />
      );
    }
    render(<ControlledSelector />);
    await user.click(screen.getByRole("button", { name: /Forte/ }));
    const fastSwitch = screen.getByRole("switch", { name: "Rapide" });

    await user.tab();
    expect(fastSwitch).toHaveFocus();
    expect(fastSwitch).toHaveClass("uis-input");
    expect(fastSwitch.closest("label")).toHaveClass("uis-switch");

    await user.keyboard(" ");

    expect(fastSwitch).toHaveFocus();
    expect(fastSwitch).toHaveAttribute("aria-checked", "true");
    expect(onFastModeChange).toHaveBeenCalledWith(true);
    expect(screen.getByRole("button", { name: /Réflexion/ })).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByRole("switch", { name: "Rapide" })).toBeInTheDocument();
  });
});
