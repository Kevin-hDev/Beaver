/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ListPanelFooter } from "@/components/layout/list-panel-footer";
import { WindowToolbar } from "@/components/layout/window-toolbar";
import { SettingsSubTabList } from "@/components/settings/settings-subtab-list";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "../core-occupants";
import { SlotResolutionContext } from "../slot-contexts";
import { createSlotRegistry } from "../slot-registry";
import { resolveSlots } from "../slot-resolution";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("@/components/agent-local/gpu-status-badge", () => ({
  GpuStatusBadge: () => <span data-testid="gpu-badge" />,
}));

afterEach(cleanup);

function withOccupants(ids: readonly string[], child: React.ReactNode) {
  const occupants = CORE_SLOT_OCCUPANTS.filter(({ id }) => ids.includes(id));
  const resolution = resolveSlots(createSlotRegistry(UI_PLACEMENTS, occupants), []);
  return render(
    <SlotResolutionContext.Provider value={resolution}>
      {child}
    </SlotResolutionContext.Provider>,
  );
}

describe("core surfaces route through the slot registry", () => {
  it("renders only navigation occupants selected by the registry", () => {
    withOccupants(
      ["beaver.settings"],
      <ListPanelFooter activeTab="settings" onTabChange={vi.fn()} />,
    );

    expect(screen.getAllByRole("button").map((button) => button.getAttribute("aria-label")))
      .toEqual(["nav.settings"]);
  });

  it("renders settings sections from their resolved occupants", () => {
    withOccupants(
      ["beaver.general", "beaver.extensions"],
      <SettingsSubTabList active="general" onSelect={vi.fn()} />,
    );

    expect(screen.getAllByRole("button").map(({ textContent }) => textContent))
      .toEqual(["settings.tabs.general", "settings.tabs.extensions"]);
  });

  it("renders only toolbar actions selected by the registry", () => {
    const onSearch = vi.fn();
    withOccupants(
      ["beaver.toolbar.search"],
      <WindowToolbar
        sidebarOpen={false}
        onToggleSidebar={vi.fn()}
        onBack={vi.fn()}
        onForward={vi.fn()}
        onNewSession={vi.fn()}
        onSearch={onSearch}
        onToggleUpdates={vi.fn()}
        updatesCount={0}
        canGoBack
        canGoForward
      />,
    );

    expect(screen.getAllByRole("button")).toHaveLength(1);
    fireEvent.click(screen.getByRole("button"));
    expect(onSearch).toHaveBeenCalledOnce();
  });
});
