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
vi.mock("../usage/provider-usage-card", () => ({
  ProviderUsageCard: ({ connectionId, siteUrl }: { connectionId: string; siteUrl: string }) => (
    <div data-testid="usage-card" data-connection-id={connectionId} data-site-url={siteUrl} />
  ),
}));

const moonshot: OAuthProviderStatus = {
  id: "moonshot",
  display_name: "Moonshot AI",
  connection_id: "moonshot-oauth",
  usage_url: "https://www.kimi.com/code/console",
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

  it("utilise directement l’identité publique fournie par Rust", () => {
    render(<OAuthProviderDetail provider={{
      ...moonshot,
      connection_id: "provider-fictif-oauth",
      usage_url: "https://example.com/usage",
    }} refresh={vi.fn(() => Promise.resolve([]))} />);

    expect(screen.getByTestId("usage-card")).toHaveAttribute(
      "data-connection-id",
      "provider-fictif-oauth",
    );
    expect(screen.getByTestId("usage-card")).toHaveAttribute(
      "data-site-url",
      "https://example.com/usage",
    );
  });
});
