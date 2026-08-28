import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_APP_NAV, type SettingsNavState } from "@/types/navigation";
import type { GatewayConfig } from "@/types/channels";
import { useChannelsTabContent } from "../channels-tab";

const mocks = vi.hoisted(() => ({
  onNavChange: vi.fn(),
  onNavReplace: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const config = {
  enabled: true,
  default_provider: "openai",
  default_model: "llama",
  channels: {
    telegram: [{
      account_id: "beaver-bot",
      enabled: true,
      allowlist: [],
      require_mention: true,
      provider: "openai",
      model: "llama",
    }],
  },
} as unknown as GatewayConfig;

vi.mock("@/hooks/use-channels", () => ({
  useChannels: () => ({
    health: { channels: [{ channel_id: "telegram", account_id: "beaver-bot", status: "off" }] },
    config,
    saveConfig: vi.fn(),
    refreshHealth: vi.fn(),
  }),
}));

vi.mock("../channel-icon", () => ({ ChannelIcon: () => <span data-testid="channel-icon" /> }));

vi.mock("../channels-detail", () => ({
  ChannelsDetail: ({ onBack }: { onBack: () => void }) => (
    <div data-testid="channel-detail">
      <button type="button" onClick={onBack}>retour</button>
    </div>
  ),
}));

vi.mock("../channels-browse-modal", () => ({ ChannelsBrowseModal: () => null }));
vi.mock("../channels-config-dialog", () => ({ ChannelsConfigDialog: () => null }));

function ChannelsHarness({ navState }: { navState: SettingsNavState }) {
  return <>{useChannelsTabContent({
    navState,
    onNavChange: mocks.onNavChange,
    onNavReplace: mocks.onNavReplace,
  })}</>;
}

describe("Navigation des canaux", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    mocks.onNavChange.mockClear();
    mocks.onNavReplace.mockClear();
  });

  it("ouvre la liste sans présélectionner un compte", () => {
    render(<ChannelsHarness navState={{ ...DEFAULT_APP_NAV.settings, channelKey: null }} />);

    expect(screen.getByText("beaver-bot")).toBeTruthy();
    expect(screen.queryByTestId("channel-detail")).toBeNull();
    expect(mocks.onNavReplace).not.toHaveBeenCalled();
  });

  it("ouvre la fiche du compte choisi", () => {
    render(<ChannelsHarness navState={{ ...DEFAULT_APP_NAV.settings, channelKey: null }} />);

    fireEvent.click(screen.getByText("beaver-bot"));

    expect(mocks.onNavChange).toHaveBeenCalledWith({ channelKey: "telegram:beaver-bot" });
  });

  it("revient à la liste depuis la fiche", () => {
    render(
      <ChannelsHarness
        navState={{ ...DEFAULT_APP_NAV.settings, channelKey: "telegram:beaver-bot" }}
      />,
    );

    fireEvent.click(screen.getByText("retour"));

    expect(mocks.onNavReplace).toHaveBeenCalledWith({ channelKey: null });
  });
});
