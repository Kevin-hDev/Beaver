import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { MODEL_PARAMETER_DEFINITIONS } from "../model-parameter-catalog";
import { ParametersEditor } from "../parameters-editor";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const modelName = "gemma4:e2b";

describe("ParametersEditor catalog", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockResolvedValue(undefined);
  });

  it("affiche les 11 paramètres officiels avec leur description", () => {
    renderEditor([]);

    expect(MODEL_PARAMETER_DEFINITIONS).toHaveLength(11);
    for (const definition of MODEL_PARAMETER_DEFINITIONS) {
      expect(screen.getByText(definition.key)).toBeTruthy();
      expect(screen.getByText(`ollama.parameterDescriptions.${definition.key}`)).toBeTruthy();
    }
    expect(screen.getByLabelText("num_ctx")).toHaveValue(null);
    expect(screen.getByLabelText("temperature")).toHaveValue(null);
    expect(screen.getByLabelText("stop 1")).toHaveValue("");
  });

  it("restaure les valeurs officielles, les stops multiples et les paramètres personnalisés", () => {
    renderEditor([
      { key: "TEMPERATURE", value: "0.4" },
      { key: "stop", value: "<end>" },
      { key: "stop", value: "User:" },
      { key: "num_gpu", value: "0" },
    ]);

    expect(screen.getByLabelText("temperature")).toHaveValue(0.4);
    expect(screen.getByLabelText("stop 1")).toHaveValue("<end>");
    expect(screen.getByLabelText("stop 2")).toHaveValue("User:");
    expect(screen.getByLabelText("ollama.customParameterName 1")).toHaveValue("num_gpu");
    expect(screen.getByLabelText("ollama.customParameterValue 1")).toHaveValue("0");
  });

  it("ne sauvegarde que les valeurs renseignées", async () => {
    const onSave = vi.fn();
    renderEditor([
      { key: "temperature", value: "0.4" },
      { key: "stop", value: "<end>" },
      { key: "stop", value: "User:" },
      { key: "num_gpu", value: "0" },
    ], onSave);

    fireEvent.click(screen.getByText("ollama.save"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("update_parameters", {
        name: modelName,
        parameters: [
          ["temperature", "0.4"],
          ["stop", "<end>"],
          ["stop", "User:"],
          ["num_gpu", "0"],
        ],
      });
      expect(onSave).toHaveBeenCalledOnce();
    });
  });

  it("ajoute plusieurs séquences stop", async () => {
    renderEditor([]);

    fireEvent.change(screen.getByLabelText("stop 1"), { target: { value: "First" } });
    fireEvent.click(screen.getByText("ollama.addStopSequence"));
    fireEvent.change(screen.getByLabelText("stop 2"), { target: { value: "Second" } });
    fireEvent.click(screen.getByText("ollama.save"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("update_parameters", {
        name: modelName,
        parameters: [["stop", "First"], ["stop", "Second"]],
      });
    });
  });

  it("bloque un paramètre personnalisé qui duplique un paramètre officiel", async () => {
    renderEditor([{ key: "future_option", value: "1" }]);

    fireEvent.change(screen.getByLabelText("ollama.customParameterName 1"), {
      target: { value: "temperature" },
    });
    fireEvent.click(screen.getByText("ollama.save"));

    expect(await screen.findByRole("alert")).toHaveTextContent("ollama.invalidCustomParameter");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("conserve la bonne ligne quand une séquence stop intermédiaire est supprimée", () => {
    renderEditor([
      { key: "stop", value: "First" },
      { key: "stop", value: "Second" },
      { key: "stop", value: "Third" },
    ]);
    const thirdInput = screen.getByLabelText("stop 3");

    fireEvent.click(screen.getAllByRole("button", {
      name: "ollama.removeStopSequence",
    })[1]);

    expect(screen.getByLabelText("stop 2")).toBe(thirdInput);
    expect(screen.getByLabelText("stop 2")).toHaveValue("Third");
  });

  it("bloque une valeur numérique officielle invalide", async () => {
    renderEditor([{ key: "num_ctx", value: "1.5" }]);

    fireEvent.click(screen.getByText("ollama.save"));

    expect(await screen.findByRole("alert")).toHaveTextContent("ollama.invalidOfficialParameter");
    expect(invoke).not.toHaveBeenCalled();
  });
});

function renderEditor(
  initialParameters: Array<{ key: string; value: string }>,
  onSave = vi.fn(),
) {
  return render(
    <ParametersEditor
      modelName={modelName}
      initialParameters={initialParameters}
      onSave={onSave}
      onCancel={vi.fn()}
    />,
  );
}
