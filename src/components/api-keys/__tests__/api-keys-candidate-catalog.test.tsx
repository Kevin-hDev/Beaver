import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useApiKeys } from "@/hooks/use-api-keys";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((command: string) => {
    if (command === "list_llm_configurable_providers_catalog") {
      return Promise.resolve([{
        id: "qwen",
        display_name: "Qwen",
        category: "llm",
        signup_url: "https://example.com",
        connection_kind: "qwen_model_studio",
      }]);
    }
    return Promise.resolve([]);
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

function CatalogHarness() {
  const { catalog } = useApiKeys();
  return <>{catalog.map((provider) => <span key={provider.id}>{provider.display_name}</span>)}</>;
}

describe("candidate API provider catalog", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("loads Qwen from the configurable catalog without using the public command", async () => {
    render(<CatalogHarness />);

    await waitFor(() => expect(screen.getByText("Qwen")).toBeTruthy());
    expect(invoke).toHaveBeenCalledWith("list_llm_configurable_providers_catalog");
    expect(invoke).not.toHaveBeenCalledWith("list_llm_providers_catalog");
  });
});
