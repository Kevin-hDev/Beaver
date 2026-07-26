import { cleanup, render, fireEvent, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useApiKeysTabSlots } from "../api-keys-tab";
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
  ApiKeysDetails: ({ provider }: { provider: ProviderSpec }) => (
    <div data-testid="api-key-detail">{provider.id}</div>
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
    free_tier_label: "",
    short_description: "",
    short_description_en: "",
  };
}

function ApiKeysHarness({ navState }: { navState: SettingsNavState }) {
  const slots = useApiKeysTabSlots({
    navState,
    onNavChange: mocks.onNavChange,
    onNavReplace: mocks.onNavReplace,
  });
  return <>{slots.list}{slots.detail}</>;
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

  it("remplace la selection par defaut sans push", async () => {
    renderTab({ ...DEFAULT_APP_NAV.settings, apiKeyProviderId: null });

    await waitFor(() => expect(mocks.onNavReplace).toHaveBeenCalledWith({ apiKeyProviderId: "openai" }));
    expect(mocks.onNavChange).not.toHaveBeenCalled();
  });

  it("push la selection utilisateur", () => {
    const { getByText } = renderTab({ ...DEFAULT_APP_NAV.settings, apiKeyProviderId: "openai" });

    fireEvent.click(getByText("groq"));

    expect(mocks.onNavChange).toHaveBeenCalledWith({ apiKeyProviderId: "groq" });
  });
});
