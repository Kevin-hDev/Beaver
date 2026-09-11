import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { HeartbeatTab } from "../heartbeat-tab";
import type { AutomationMigrationStatus } from "@/types/wakeup";

type WakeupsApi = ReturnType<typeof import("@/hooks/use-wakeups").useWakeups>;

const chooseTimezone = vi.fn();
const resolveConflict = vi.fn();
const setPaused = vi.fn();
const useWakeups = vi.fn<() => WakeupsApi>();

vi.mock("react-i18next", () => ({
  initReactI18next: { type: "3rdParty", init: vi.fn() },
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("@/components/ui/panel-slots", () => ({
  PanelSlot: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));
vi.mock("@/hooks/use-arrow-navigation", () => ({ useArrowNavigation: vi.fn() }));
vi.mock("@/hooks/use-wakeups", () => ({ useWakeups: () => useWakeups() }));

function api(migration: AutomationMigrationStatus): WakeupsApi {
  return {
    wakeups: [], globalPaused: false, migration, detail: null,
    history: { entries: [], next_cursor: null }, loading: false,
    detailLoading: false, error: null, refresh: vi.fn(), loadDetail: vi.fn(),
    loadMoreHistory: vi.fn(), create: vi.fn(), update: vi.fn(), remove: vi.fn(),
    toggle: vi.fn(), setPaused, chooseTimezone, resolveConflict,
  };
}

describe("HeartbeatTab migration", () => {
  beforeEach(() => vi.clearAllMocks());

  it("reprend la migration avec le fuseau choisi", () => {
    useWakeups.mockReturnValue(api({ status: "needs_timezone" }));
    render(<HeartbeatTab />);
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "Europe/Paris" } });
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.migration.continue" }));
    expect(chooseTimezone).toHaveBeenCalledWith("Europe/Paris");
  });

  it("expose les deux décisions de conflit et la pause globale", () => {
    useWakeups.mockReturnValue(api({
      status: "conflicts", conflicts: [{ legacy_id: "legacy-1" }],
    }));
    render(<HeartbeatTab />);
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.migration.remove" }));
    fireEvent.click(screen.getByRole("button", { name: "heartbeat.migration.import" }));
    fireEvent.click(screen.getByRole("switch", { name: "heartbeat.sidebar.pause" }));
    expect(resolveConflict).toHaveBeenNthCalledWith(1, "legacy-1", "remove_historical");
    expect(resolveConflict).toHaveBeenNthCalledWith(2, "legacy-1", "import_as_new", expect.any(String));
    expect(setPaused).toHaveBeenCalledWith(true);
  });
});
