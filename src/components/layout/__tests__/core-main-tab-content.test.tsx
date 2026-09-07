/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { useEffect } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "@/features/extension-ui/core-occupants";
import { SlotResolutionContext } from "@/features/extension-ui/slot-contexts";
import { createSlotRegistry } from "@/features/extension-ui/slot-registry";
import { resolveSlots } from "@/features/extension-ui/slot-resolution";
import { DEFAULT_AGENT_LOCAL_NAV, DEFAULT_APP_NAV } from "@/types/navigation";
import type { MainTabId } from "@/features/extension-ui/slot-types";
import { CoreMainTabContent } from "../core-main-tab-content";
import { useAppSurfaceActive } from "../app-surface-activity";

const doubles = vi.hoisted(() => ({ mounts: 0, unmounts: 0 }));

vi.mock("@/components/heartbeat/heartbeat-tab", () => ({
  HeartbeatTab: () => <div data-testid="heartbeat-content" />,
}));
vi.mock("@/components/personality/personality-tab", () => ({
  PersonalityTab: () => <div data-testid="personality-content" />,
}));
vi.mock("@/components/agent-local/agent-local-tab", () => ({
  AgentLocalTab: ({ listFocused }: { listFocused: boolean }) => {
    const active = useAppSurfaceActive();
    useEffect(() => {
      doubles.mounts += 1;
      return () => { doubles.unmounts += 1; };
    }, []);
    return (
      <div
        data-testid="agent-content"
        data-surface-active={String(active)}
        data-list-focused={String(listFocused)}
      />
    );
  },
}));
vi.mock("@/components/settings/settings-tab", () => ({
  SettingsTab: () => <div data-testid="settings-content" />,
}));
vi.mock("@/features/extension-ui/standard/standard-contributions", () => ({
  StandardTabContent: ({ entry }: { entry: { contributionId: string } }) => (
    <div data-testid="standard-content" data-entry={entry.contributionId} />
  ),
  useStandardEntry: (occupant: { source: { kind: string }; id: string } | undefined) =>
    occupant?.source.kind === "extension" ? { contributionId: occupant.id } : undefined,
}));

afterEach(() => {
  cleanup();
  doubles.mounts = 0;
  doubles.unmounts = 0;
});

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

  it("keeps Agent Local mounted and changes only its activity across navigation", () => {
    const resolution = resolveSlots(
      createSlotRegistry(UI_PLACEMENTS, CORE_SLOT_OCCUPANTS),
      [{
        extensionId: "acme",
        contributionId: "tab",
        operation: "add",
        placement: "app.navigation.primary",
        contributionType: "tab",
        order: 25,
      }],
    );
    const props = {
      nav: { ...DEFAULT_APP_NAV, tab: "agent-local" as const },
      agentNavState: DEFAULT_AGENT_LOCAL_NAV,
      themeChoice: "dark" as const,
      focusedPanel: "list" as const,
      onWakeupChange: vi.fn(),
      onPathChange: vi.fn(),
      onSessionChange: vi.fn(),
      onAgentNavChange: vi.fn(),
      onWorkspaceClear: vi.fn(),
      onThemeChange: vi.fn(),
      onSettingsNavChange: vi.fn(),
      onSettingsNavReplace: vi.fn(),
    };
    const view = (activeTab: string) => (
      <SlotResolutionContext.Provider value={resolution}>
        <CoreMainTabContent {...props} activeTab={activeTab as MainTabId} />
      </SlotResolutionContext.Provider>
    );
    const rendered = render(view("agent-local"));

    expect(doubles).toEqual({ mounts: 1, unmounts: 0 });
    for (const activeTab of [
      "settings", "heartbeat", "personality", "extension:acme:tab", "agent-local",
    ]) {
      rendered.rerender(view(activeTab));
      expect(doubles).toEqual({ mounts: 1, unmounts: 0 });
      expect(screen.getByTestId("agent-content").dataset.surfaceActive)
        .toBe(String(activeTab === "agent-local"));
      expect(screen.getByTestId("agent-content").dataset.listFocused)
        .toBe(String(activeTab === "agent-local"));
      expect(screen.queryByTestId("standard-content") !== null)
        .toBe(activeTab === "extension:acme:tab");
      for (const [target, testId] of [
        ["settings", "settings-content"],
        ["heartbeat", "heartbeat-content"],
        ["personality", "personality-content"],
      ]) {
        expect(screen.queryByTestId(testId) !== null).toBe(activeTab === target);
      }
    }
  });
});
