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
  ModelfileView: ({ modelfile, parameters }: {
    modelfile: string;
    parameters: Array<{ key: string; value: string }>;
  }) => (
    <div>
      <span>{modelfile}</span>
      <span data-testid="semantic-parameters">
        {parameters.map(({ key, value }) => `${key}:${value}`).join("|")}
      </span>
    </div>
  ),
}));

describe("ModelfileViewer data contract", () => {
  beforeEach(() => {
    mocks.invoke.mockReset();
    mocks.invoke.mockResolvedValue({
      modelfile: "FROM x",
      parameters: [{ key: "stop", value: " line one\nline two " }],
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
});
