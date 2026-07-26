/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { OAuthLoginProgress, OAuthProviderStatus } from "@/types/oauth-provider";
import { OAuthProviderLoginDialog } from "../oauth-provider-login-dialog";

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve()) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn(() => Promise.resolve()) }));

const moonshot: OAuthProviderStatus = {
  id: "moonshot",
  display_name: "Moonshot AI",
  connected: false,
  account: null,
  experimental: true,
};

describe("OAuthProviderLoginDialog", () => {
  let progress: ((event: { payload: OAuthLoginProgress }) => void) | undefined;
  let clipboardWrite: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.mocked(invoke).mockClear().mockResolvedValue(undefined);
    vi.mocked(listen).mockImplementation((_event, handler) => {
      progress = handler as typeof progress;
      return Promise.resolve(() => undefined);
    });
    clipboardWrite = vi.fn(() => Promise.resolve());
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText: clipboardWrite },
    });
    Object.defineProperty(globalThis.crypto, "randomUUID", {
      configurable: true,
      value: () => "12345678-1234-4234-8234-123456789abc",
    });
  });

  it("lance directement la connexion web native", async () => {
    render(<OAuthProviderLoginDialog provider={moonshot} onClose={vi.fn()} onConnected={vi.fn()} />);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("start_oauth_provider_login", {
      providerId: "moonshot",
      diagnosticId: "12345678-1234-4234-8234-123456789abc",
    }));
  });

  it("affiche et copie le code d'appareil pour Kimi", async () => {
    render(<OAuthProviderLoginDialog provider={moonshot} onClose={vi.fn()} onConnected={vi.fn()} />);
    await waitFor(() => expect(progress).toBeTypeOf("function"));
    progress?.({ payload: {
      provider_id: "moonshot",
      stage: "device_code",
      user_code: "KIMI-CODE",
      verification_url: "https://auth.kimi.com/activate",
    } });
    expect(await screen.findByText("KIMI-CODE")).toBeTruthy();
    fireEvent.click(screen.getByText("providers.oauth.copyCode"));
    expect(clipboardWrite).toHaveBeenCalledWith("KIMI-CODE");
  });

  it("ne valide la connexion qu'après le signal de succès", async () => {
    const onConnected = vi.fn();
    render(<OAuthProviderLoginDialog provider={moonshot} onClose={vi.fn()} onConnected={onConnected} />);
    await waitFor(() => expect(progress).toBeTypeOf("function"));
    progress?.({ payload: { provider_id: "moonshot", stage: "success" } });
    await waitFor(() => expect(onConnected).toHaveBeenCalledTimes(1));
  });
});
