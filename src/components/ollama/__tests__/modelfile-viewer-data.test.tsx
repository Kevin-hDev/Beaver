import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ModelfileViewer } from "../modelfile-viewer";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("../modelfile-view", () => ({
  ModelfileView: ({ modelfile, parameters, parameterError }: {
    modelfile: string;
    parameters: Array<{ key: string; value: string }> | null;
    parameterError: string | null;
  }) => (
    <div>
      <span>{modelfile}</span>
      <span data-testid="semantic-parameters">
        {parameters?.map(({ key, value }) => `${key}:${value}`).join("|")}
      </span>
      {parameterError && <span role="alert">{parameterError}</span>}
    </div>
  ),
}));

describe("ModelfileViewer data contract", () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
    mocks.invoke.mockResolvedValue({
      modelfile: "FROM x",
      parameters: [{ key: "stop", value: " line one\nline two " }],
      parameterError: null,
    });
  });

  it("consomme les paramètres sémantiques fournis par Rust", async () => {
    render(<ModelfileViewer modelName="gemma4:e2b" onBack={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("FROM x")).toBeTruthy();
      expect(screen.getByTestId("semantic-parameters").textContent).toBe(
        "stop: line one\nline two ",
      );
    });
    expect(mocks.invoke).toHaveBeenCalledWith("get_modelfile", {
      name: "gemma4:e2b",
    });
  });

  it("conserve le Modelfile brut quand les paramètres simplifiés sont indisponibles", async () => {
    mocks.invoke.mockResolvedValueOnce({
      modelfile: "FROM x\nPARAMETER stop oversized",
      parameters: null,
      parameterError: "ollama-invalid-response",
    });

    render(<ModelfileViewer modelName="large:latest" onBack={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByRole("alert")).toHaveTextContent(
        "errors.localStore.ollamaResponseInvalid",
      );
    });
  });
});
