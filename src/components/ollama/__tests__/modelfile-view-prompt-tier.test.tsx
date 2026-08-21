import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ModelfileView } from "../modelfile-view";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/system-prompts/system-prompt-settings-panel", async () => {
  const { useState } = await vi.importActual<typeof import("react")>("react");
  return {
    SystemPromptSettingsPanel: ({ initialTier }: { initialTier: string }) => {
      const [mountedTier] = useState(initialTier);
      return <div data-testid="mounted-prompt-tier">{mountedTier}</div>;
    },
  };
});

describe("ModelfileView prompt tier", () => {
  it("remonte le panneau quand Ollama signale un nouveau palier", () => {
    const props = {
      modelName: "custom-model",
      parameters: [],
      parameterError: null,
      modelfile: "FROM custom-model",
      onEditParameters: vi.fn(),
    };
    const { rerender } = render(<ModelfileView {...props} promptTier="compact" />);

    expect(screen.getByTestId("mounted-prompt-tier")).toHaveTextContent("compact");

    rerender(<ModelfileView {...props} promptTier="detailed" />);

    expect(screen.getByTestId("mounted-prompt-tier")).toHaveTextContent("detailed");
  });
});
