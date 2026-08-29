import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiKeysConfigDialog } from "../api-keys-config-dialog";
import type { ProviderSpec } from "@/types/api";
import { invoke } from "@tauri-apps/api/core";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, string>) =>
      values?.name ? `${key}:${values.name}` : key,
  }),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const qwen: ProviderSpec = {
  id: "qwen",
  display_name: "Qwen",
  category: "llm",
  signup_url: "https://example.com",
  connection_kind: "qwen_model_studio",
};

describe("ApiKeysConfigDialog Qwen connection", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

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

  it("reloads and preserves the configured Qwen connection while editing", async () => {
    vi.mocked(invoke).mockResolvedValue({
      region: "frankfurt",
      endpointMode: "workspace",
      workspaceId: "team-42",
    });
    const onTest = vi.fn().mockResolvedValue(undefined);
    const onSave = vi.fn().mockResolvedValue(undefined);
    const { container } = render(
      <ApiKeysConfigDialog
        provider={qwen}
        alreadyConfigured
        onClose={vi.fn()}
        onTest={onTest}
        onSave={onSave}
      />,
    );

    await screen.findByText("apiKeys.connection.regions.frankfurt");
    fireEvent.change(container.querySelector('input[type="password"]')!, {
      target: { value: "sk-new" },
    });
    fireEvent.click(screen.getByText("apiKeys.dialog.save"));

    await waitFor(() => expect(onSave).toHaveBeenCalledWith("sk-new", {
      region: "frankfurt",
      endpointMode: "workspace",
      workspaceId: "team-42",
    }));
  });

  it("fails closed when an edited Qwen connection cannot be reloaded", async () => {
    vi.mocked(invoke).mockResolvedValue(null);
    const { container } = render(
      <ApiKeysConfigDialog
        provider={qwen}
        alreadyConfigured
        onClose={vi.fn()}
        onTest={vi.fn()}
        onSave={vi.fn()}
      />,
    );

    fireEvent.change(container.querySelector('input[type="password"]')!, {
      target: { value: "sk-new" },
    });

    expect(await screen.findByText("errors.operationFailed")).toBeInTheDocument();
    expect(screen.getByText("apiKeys.dialog.save")).toBeDisabled();
  });

  it("disables save when the Qwen workspace is invalid", () => {
    const { container } = render(
      <ApiKeysConfigDialog
        provider={qwen}
        alreadyConfigured={false}
        onClose={vi.fn()}
        onTest={vi.fn()}
        onSave={vi.fn()}
      />,
    );
    fireEvent.change(container.querySelector('input[type="password"]')!, {
      target: { value: "sk-fixture" },
    });
    fireEvent.click(screen.getByLabelText("apiKeys.connection.endpointMode"));
    fireEvent.click(screen.getByText("apiKeys.connection.modes.workspace"));

    expect(screen.getByText("apiKeys.dialog.addAndTest")).toBeDisabled();
  });
});
