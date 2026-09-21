import { act, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { McpOauthDialog } from "../mcp-oauth-dialog";
import type { McpConnectorSpec } from "@/types/mcp";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itJson from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

const connector: McpConnectorSpec = {
  id: "vercel",
  display_name: "Vercel",
  category: "devtools",
  auth_type: "oauth",
  short_descriptions: { fr: "Test", en: "Test", es: "Test", de: "Test", it: "Test", zh: "Test", ja: "Test" },
  author: "Test",
  url: "https://vercel.com",
  endpoint: "https://mcp.vercel.com",
  tools: [],
};

describe("McpOauthDialog", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockResolvedValue(undefined);
    vi.mocked(listen).mockReset();
  });

  it("explique le refus d’une adresse OAuth inconnue sans exposer le détail interne", async () => {
    let sendResult: ((payload: unknown) => void) | undefined;
    vi.mocked(listen).mockImplementation((_event, handler) => {
      sendResult = (payload) => handler({ payload } as Parameters<typeof handler>[0]);
      return Promise.resolve(() => {});
    });

    render(<McpOauthDialog connector={connector} onClose={vi.fn()} onConnected={vi.fn()} />);
    await waitFor(() => expect(sendResult).toBeDefined());
    act(() => sendResult!({ connector_id: "vercel", success: false, error: "endpoint OAuth non autorisé" }));

    expect(screen.getByText("connectors.oauth.errorUntrusted")).toBeInTheDocument();
    expect(screen.queryByText("endpoint OAuth non autorisé")).not.toBeInTheDocument();
  });

  it("traduit le refus d’adresse dans les sept langues", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as Array<{
      connectors: { oauth: { errorUntrusted: string } };
    }>;
    for (const locale of locales) {
      expect(locale.connectors.oauth.errorUntrusted.length).toBeGreaterThan(20);
    }
  });
});
