/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fetchOAuthModels } from "@/hooks/oauth-models";
import type { OAuthProviderStatus } from "@/types/oauth-provider";
import { OAuthProviderDetail } from "../oauth-provider-detail";

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve()) }));
vi.mock("@/hooks/oauth-models", () => ({ fetchOAuthModels: vi.fn() }));

const moonshot: OAuthProviderStatus = {
  id: "moonshot",
  display_name: "Moonshot AI",
  connected: true,
  account: null,
  experimental: true,
};

describe("OAuthProviderDetail", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockClear();
    vi.mocked(fetchOAuthModels).mockReset().mockResolvedValue({
      groups: new Map(),
      issues: new Map([["moonshot", "moonshot_membership_unverified"]]),
    });
  });

  it("affiche uniquement le message sûr correspondant à l’erreur Moonshot", async () => {
    render(<OAuthProviderDetail provider={moonshot} refresh={vi.fn(() => Promise.resolve([]))} />);

    expect(await screen.findByText("providers.oauth.issues.moonshotMembershipUnverified")).toBeTruthy();
    expect(screen.queryByText(/membership benefits/i)).toBeNull();
  });

  it("permet de retester le catalogue sans relancer l’authentification", async () => {
    render(<OAuthProviderDetail provider={moonshot} refresh={vi.fn(() => Promise.resolve([]))} />);
    fireEvent.click(await screen.findByText("providers.oauth.retryCatalog"));

    await waitFor(() => expect(fetchOAuthModels).toHaveBeenLastCalledWith(true));
    expect(invoke).not.toHaveBeenCalledWith("start_oauth_provider_login", expect.anything());
  });
});
