import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ModelfileView } from "../modelfile-view";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("@/components/system-prompts/system-prompt-settings-panel", () => ({
  SystemPromptSettingsPanel: () => null,
}));

describe("ModelfileView parameter error", () => {
  it("affiche le fichier brut et désactive l'éditeur simplifié", () => {
    const { container } = render(
      <ModelfileView
        modelName="large:latest"
        parameters={null}
        parameterError="errors.localStore.ollamaResponseInvalid"
        modelfile={"FROM x\nPARAMETER stop oversized"}
        onEditParameters={vi.fn()}
      />,
    );

    expect(container.querySelector(".mf-raw-block")?.textContent).toBe(
      "FROM x\nPARAMETER stop oversized",
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "errors.localStore.ollamaResponseInvalid",
    );
    expect(screen.getByRole("button", { name: "ollama.edit" })).toBeDisabled();
  });
});
