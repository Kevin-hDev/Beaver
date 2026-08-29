import { cleanup, render, fireEvent, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useApiKeysTabContent } from "../api-keys-tab";
import { DEFAULT_APP_NAV, type SettingsNavState } from "@/types/navigation";
import type { ProviderSpec } from "@/types/api";

const mocks = vi.hoisted(() => ({
  catalog: [] as ProviderSpec[],
  configured: [] as ProviderSpec[],
  onNavChange: vi.fn(),
  onNavReplace: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/hooks/use-api-keys", () => ({
  useApiKeys: () => ({
    catalog: mocks.catalog,
    configuredIds: mocks.configured.map((item) => item.id),
    configured: mocks.configured,
    setKey: vi.fn(),
    deleteKey: vi.fn(),
    testKeyRaw: vi.fn(),
  }),
}));

vi.mock("@/lib/provider-icons", () => ({
  ProviderIcon: () => <span data-testid="provider-icon" />,
}));

vi.mock("../api-keys-details", () => ({
  ApiKeysDetails: ({ provider, onBack }: { provider: ProviderSpec; onBack: () => void }) => (
    <div data-testid="api-key-detail">
      {provider.id}
      <button type="button" onClick={onBack}>retour</button>
    </div>
  ),
}));

vi.mock("../api-keys-config-dialog", () => ({
  ApiKeysConfigDialog: ({ provider }: { provider: ProviderSpec }) => (
    <div role="dialog">config:{provider.id}</div>
  ),
}));

function provider(id: string): ProviderSpec {
  return {
    id,
    display_name: id,
    category: "llm",
    signup_url: "",
    connection_kind: "api_key",
  };
}

function ApiKeysHarness({ navState }: { navState: SettingsNavState }) {
  return <>{useApiKeysTabContent({
    navState,
    onNavChange: mocks.onNavChange,
    onNavReplace: mocks.onNavReplace,
  })}</>;
}

function renderTab(navState: SettingsNavState) {
  return render(<ApiKeysHarness navState={navState} />);
}

describe("ApiKeysTab navigation", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    mocks.catalog = [
      provider("openai"),
      provider("anthropic"),
      { ...provider("qwen"), connection_kind: "qwen_model_studio" },
    ];
    mocks.configured = [provider("openai"), provider("mistral")];
    mocks.onNavChange.mockClear();
    mocks.onNavReplace.mockClear();
  });

  it("permet de configurer Anthropic sans le publier dans l'onboarding", () => {
    const { getByText } = renderTab({ ...DEFAULT_APP_NAV.settings, apiKeyProviderId: null });

    fireEvent.click(getByText("apiKeys.main.connectorsBtn"));
    fireEvent.click(getByText("anthropic"));

    expect(getByText("config:anthropic")).toBeTruthy();
  });

  it("ouvre la configuration régionale Qwen depuis les connecteurs", () => {
    const { getByText } = renderTab({ ...DEFAULT_APP_NAV.settings, apiKeyProviderId: null });

    fireEvent.click(getByText("apiKeys.main.connectorsBtn"));
    fireEvent.click(getByText("qwen"));

    expect(getByText("config:qwen")).toBeTruthy();
  });

  it("ouvre la liste sans présélectionner un fournisseur", async () => {
    const { getByText, queryByTestId } = renderTab({
      ...DEFAULT_APP_NAV.settings,
      apiKeyProviderId: null,
    });

    await waitFor(() => expect(getByText("openai")).toBeTruthy());
    expect(getByText("mistral")).toBeTruthy();
    expect(queryByTestId("api-key-detail")).toBeNull();
    expect(mocks.onNavReplace).not.toHaveBeenCalled();
    expect(mocks.onNavChange).not.toHaveBeenCalled();
  });

  it("push la selection utilisateur", () => {
    const { getByText } = renderTab({ ...DEFAULT_APP_NAV.settings, apiKeyProviderId: null });

    fireEvent.click(getByText("mistral"));

    expect(mocks.onNavChange).toHaveBeenCalledWith({ apiKeyProviderId: "mistral" });
  });

  it("remplace la fiche par la liste au retour", () => {
    const { getByText, getByTestId } = renderTab({
      ...DEFAULT_APP_NAV.settings,
      apiKeyProviderId: "mistral",
    });

    expect(getByTestId("api-key-detail")).toBeTruthy();
    fireEvent.click(getByText("retour"));

    expect(mocks.onNavReplace).toHaveBeenCalledWith({ apiKeyProviderId: null });
  });
});
