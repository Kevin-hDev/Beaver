import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { McpConfigDialog } from "../mcp-config-dialog";
import type { McpConnectorSpec } from "@/types/mcp";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itJson from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: "fr" } }),
}));

const connector: McpConnectorSpec = {
  id: "huggingface",
  display_name: "Hugging Face",
  category: "ai-ml",
  auth_type: "token",
  short_descriptions: { fr: "Test", en: "Test", es: "Test", de: "Test", it: "Test", zh: "Test", ja: "Test" },
  author: "Test",
  url: "https://example.com",
  install_command: "npx @llmindset/hf-mcp-server@0.3.13",
  env_keys: ["HF_TOKEN"],
  tools: [],
};

describe("McpConfigDialog", () => {
  beforeEach(() => vi.mocked(invoke).mockReset().mockResolvedValue(undefined));

  it("teste et stocke le token dans une seule opération atomique", async () => {
    const onValidated = vi.fn().mockResolvedValue(undefined);
    const { container } = render(
      <McpConfigDialog connector={connector} onClose={vi.fn()} onValidated={onValidated} />,
    );
    const input = container.querySelector("input")!;
    fireEvent.change(input, { target: { value: "hf-secret" } });
    fireEvent.click(screen.getByText("connectors.config.addAndTest"));

    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
    expect(invoke).toHaveBeenCalledWith("configure_mcp_connector_tokens", {
      connector: {
        id: "huggingface",
        status: "connected",
        enabled_in_chat: true,
        endpoint: undefined,
        install_command: "npx @llmindset/hf-mcp-server@0.3.13",
        env_keys: ["HF_TOKEN"],
      },
      envTokens: [{ env_key: "HF_TOKEN", value: "hf-secret" }],
    });
    await waitFor(() => expect(input).toHaveValue(""));
    expect(screen.getByText("connectors.localSecurityWarning")).toBeInTheDocument();
  });

  it("efface le secret après un échec et à la fermeture", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("secret detail"));
    const onClose = vi.fn();
    const { container } = render(
      <McpConfigDialog connector={connector} onClose={onClose} onValidated={vi.fn()} />,
    );
    const input = container.querySelector("input")!;
    fireEvent.change(input, { target: { value: "first-secret" } });
    fireEvent.click(screen.getByText("connectors.config.addAndTest"));
    await waitFor(() => expect(input).toHaveValue(""));

    fireEvent.change(input, { target: { value: "second-secret" } });
    fireEvent.click(screen.getByRole("button", { name: "connectors.config.cancel" }));
    expect(input).toHaveValue("");
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("fournit l’avertissement local dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as Array<{
      connectors: { localSecurityWarning: string };
    }>;
    for (const locale of locales) {
      expect(locale.connectors.localSecurityWarning.length).toBeGreaterThan(20);
    }
  });
});
