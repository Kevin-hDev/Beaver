/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "@/features/extension-ui/core-occupants";
import { SlotResolutionContext } from "@/features/extension-ui/slot-contexts";
import { createSlotRegistry } from "@/features/extension-ui/slot-registry";
import { resolveSlots } from "@/features/extension-ui/slot-resolution";
import { DEFAULT_AGENT_LOCAL_NAV, DEFAULT_APP_NAV } from "@/types/navigation";
import { CoreMainTabContent } from "../core-main-tab-content";

vi.mock("@/components/heartbeat/heartbeat-tab", () => ({
  HeartbeatTab: () => <div data-testid="heartbeat-content" />,
}));
vi.mock("@/components/personality/personality-tab", () => ({
  PersonalityTab: () => <div data-testid="personality-content" />,
}));
vi.mock("@/components/agent-local/agent-local-tab", () => ({
  AgentLocalTab: () => <div data-testid="agent-content" />,
}));
vi.mock("@/components/settings/settings-tab", () => ({
  SettingsTab: () => <div data-testid="settings-content" />,
}));

afterEach(cleanup);

describe("CoreMainTabContent", () => {
  it("renders only the active occupant selected by the registry", () => {
    const heartbeat = CORE_SLOT_OCCUPANTS.filter(({ id }) => id === "beaver.heartbeat");
    const resolution = resolveSlots(createSlotRegistry(UI_PLACEMENTS, heartbeat), []);
    render(
      <SlotResolutionContext.Provider value={resolution}>
        <CoreMainTabContent
          activeTab="heartbeat"
          nav={{ ...DEFAULT_APP_NAV, tab: "heartbeat" }}
          agentNavState={DEFAULT_AGENT_LOCAL_NAV}
          themeChoice="dark"
          focusedPanel="list"
          onWakeupChange={vi.fn()}
          onPathChange={vi.fn()}
          onSessionChange={vi.fn()}
          onAgentNavChange={vi.fn()}
          onWorkspaceClear={vi.fn()}
          onThemeChange={vi.fn()}
          onSettingsNavChange={vi.fn()}
          onSettingsNavReplace={vi.fn()}
        />
      </SlotResolutionContext.Provider>,
    );

    expect(screen.getByTestId("heartbeat-content")).toBeTruthy();
    expect(screen.queryByTestId("agent-content")).toBeNull();
    expect(screen.queryByTestId("settings-content")).toBeNull();
  });
});
