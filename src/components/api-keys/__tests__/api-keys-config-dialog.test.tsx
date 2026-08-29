import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiKeysConfigDialog } from "../api-keys-config-dialog";
import type { ProviderSpec } from "@/types/api";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, string>) =>
      values?.name ? `${key}:${values.name}` : key,
  }),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));

const qwen: ProviderSpec = {
  id: "qwen",
  display_name: "Qwen",
  category: "llm",
  signup_url: "https://example.com",
  connection_kind: "qwen_model_studio",
};

describe("ApiKeysConfigDialog Qwen connection", () => {
  afterEach(cleanup);

  it("passes the validated regional connection to test and save", async () => {
    const onTest = vi.fn().mockResolvedValue(undefined);
    const onSave = vi.fn().mockResolvedValue(undefined);
    const { container } = render(
      <ApiKeysConfigDialog
        provider={qwen}
        alreadyConfigured={false}
        onClose={vi.fn()}
        onTest={onTest}
        onSave={onSave}
      />,
    );

    fireEvent.change(container.querySelector('input[type="password"]')!, {
      target: { value: "sk-fixture" },
    });
    fireEvent.click(screen.getByText("apiKeys.dialog.addAndTest"));

    await waitFor(() => expect(onSave).toHaveBeenCalledWith("sk-fixture", {
      region: "singapore",
      endpointMode: "shared",
    }));
    expect(onTest).toHaveBeenCalledWith("sk-fixture", {
      region: "singapore",
      endpointMode: "shared",
    });
  });
});
