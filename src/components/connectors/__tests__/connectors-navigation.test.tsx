import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_APP_NAV, type SettingsNavState } from "@/types/navigation";
import type { ConfiguredMcpFull } from "@/types/mcp";
import { useConnectorsTabContent } from "../connectors-tab";

const mocks = vi.hoisted(() => ({
  configured: [] as ConfiguredMcpFull[],
  onNavChange: vi.fn(),
  onNavReplace: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/hooks/use-connectors", () => ({
  useConnectors: () => ({
    catalog: [],
    configured: mocks.configured,
    configuredIds: mocks.configured.map((item) => item.id),
    loadError: false,
    addConnector: vi.fn(),
    removeConnector: vi.fn(),
    toggleStatus: vi.fn(),
  }),
}));

vi.mock("@/lib/mcp-icons", () => ({
  McpIcon: () => <span data-testid="mcp-icon" />,
  mcpHasTextIcon: () => false,
}));

vi.mock("../connectors-detail", () => ({
  ConnectorsDetail: ({ connector, onBack }: { connector: ConfiguredMcpFull; onBack: () => void }) => (
    <div data-testid="connector-detail">
      {connector.id}
      <button type="button" onClick={onBack}>retour</button>
    </div>
  ),
}));

vi.mock("../mcp-browse-modal", () => ({ McpBrowseModal: () => null }));
vi.mock("../mcp-config-dialog", () => ({ McpConfigDialog: () => null }));
vi.mock("../mcp-oauth-dialog", () => ({ McpOauthDialog: () => null }));
vi.mock("../connectors-confirm-dialogs", () => ({ ConnectorsConfirmDialogs: () => null }));

function connector(id: string, status: ConfiguredMcpFull["status"]): ConfiguredMcpFull {
  return {
    id,
    display_name: id,
    status,
    tools: [],
    author: "",
    url: "",
    category: "productivity",
    auth_type: "none",
    short_descriptions: {},
  } as unknown as ConfiguredMcpFull;
}

function ConnectorsHarness({ navState }: { navState: SettingsNavState }) {
  return <>{useConnectorsTabContent({
    navState,
    onNavChange: mocks.onNavChange,
    onNavReplace: mocks.onNavReplace,
  })}</>;
}

describe("Navigation des connecteurs", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    mocks.configured = [connector("canva", "connected"), connector("notion", "disconnected")];
    mocks.onNavChange.mockClear();
    mocks.onNavReplace.mockClear();
  });

  it("ouvre la liste sans présélectionner un connecteur", () => {
    render(<ConnectorsHarness navState={{ ...DEFAULT_APP_NAV.settings, connectorId: null }} />);

    expect(screen.getByText("canva")).toBeTruthy();
    expect(screen.queryByTestId("connector-detail")).toBeNull();
    expect(mocks.onNavReplace).not.toHaveBeenCalled();
  });

  it("signale un connecteur déconnecté dans le nom accessible", () => {
    render(<ConnectorsHarness navState={{ ...DEFAULT_APP_NAV.settings, connectorId: null }} />);

    const rows = screen.getAllByRole("button").filter((row) => row.textContent?.includes("notion"));

    expect(rows[0].getAttribute("aria-label")).toBe("notion — connectors.detail.disconnected");
  });

  it("revient à la liste depuis la fiche", () => {
    render(<ConnectorsHarness navState={{ ...DEFAULT_APP_NAV.settings, connectorId: "canva" }} />);

    expect(screen.getByTestId("connector-detail")).toBeTruthy();
    fireEvent.click(screen.getByText("retour"));

    expect(mocks.onNavReplace).toHaveBeenCalledWith({ connectorId: null });
  });
});
