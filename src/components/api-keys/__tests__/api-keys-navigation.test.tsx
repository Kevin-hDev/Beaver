import { cleanup, render, fireEvent, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useApiKeysTabContent } from "../api-keys-tab";
import { DEFAULT_APP_NAV, type SettingsNavState } from "@/types/navigation";
import type { ProviderSpec } from "@/types/api";

const mocks = vi.hoisted(() => ({
  configured: [] as ProviderSpec[],
  onNavChange: vi.fn(),
  onNavReplace: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/hooks/use-api-keys", () => ({
  useApiKeys: () => ({
    catalog: [],
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
  ApiKeysConfigDialog: () => null,
}));

vi.mock("../connectors-modal", () => ({
  ConnectorsModal: () => null,
}));

function provider(id: string): ProviderSpec {
  return {
    id,
    display_name: id,
    category: "llm",
    signup_url: "",
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
    mocks.configured = [provider("openai"), provider("groq")];
    mocks.onNavChange.mockClear();
    mocks.onNavReplace.mockClear();
  });

  it("ouvre la liste sans présélectionner un fournisseur", async () => {
    const { getByText, queryByTestId } = renderTab({
      ...DEFAULT_APP_NAV.settings,
      apiKeyProviderId: null,
    });

    await waitFor(() => expect(getByText("openai")).toBeTruthy());
    expect(getByText("groq")).toBeTruthy();
    expect(queryByTestId("api-key-detail")).toBeNull();
    expect(mocks.onNavReplace).not.toHaveBeenCalled();
    expect(mocks.onNavChange).not.toHaveBeenCalled();
  });

  it("push la selection utilisateur", () => {
    const { getByText } = renderTab({ ...DEFAULT_APP_NAV.settings, apiKeyProviderId: null });

    fireEvent.click(getByText("groq"));

    expect(mocks.onNavChange).toHaveBeenCalledWith({ apiKeyProviderId: "groq" });
  });

  it("remplace la fiche par la liste au retour", () => {
    const { getByText, getByTestId } = renderTab({
      ...DEFAULT_APP_NAV.settings,
      apiKeyProviderId: "groq",
    });

    expect(getByTestId("api-key-detail")).toBeTruthy();
    fireEvent.click(getByText("retour"));

    expect(mocks.onNavReplace).toHaveBeenCalledWith({ apiKeyProviderId: null });
  });
});
