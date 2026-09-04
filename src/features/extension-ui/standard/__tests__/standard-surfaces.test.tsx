/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import "@/i18n";
import { ChatInputActionsRow } from "@/components/agent-local/chat-input-actions-row";
import { ListPanelFooter } from "@/components/layout/list-panel-footer";
import { WindowToolbar } from "@/components/layout/window-toolbar";
import { SettingsSubTabList } from "@/components/settings/settings-subtab-list";
import { SlotProvider } from "../../slot-provider";
import { StandardCatalogProvider } from "../catalog-context";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type { ExtensionOccupantId } from "../../slot-types";

vi.mock("@/components/agent-local/gpu-status-badge", () => ({
  GpuStatusBadge: () => <span data-testid="gpu" />,
}));
vi.mock("@/components/agent-local/chat-plus-menu", () => ({
  ChatPlusMenu: () => <span data-testid="plus-menu" />,
}));
vi.mock("@/components/agent-local/context-progress", () => ({ ContextProgress: () => null }));
vi.mock("@/components/agent-local/model-controls", () => ({ ModelControls: () => null }));
vi.mock("@/components/agent-local/permission-mode-selector", () => ({
  PermissionModeSelector: () => null,
}));
vi.mock("@/components/agent-local/missing-directory-prompt", () => ({
  MissingDirectoryPrompt: () => null,
}));
vi.mock("@/components/agent-local/plan-mode-badge", () => ({ PlanModeBadge: () => null }));
vi.mock("@/components/agent-local/retry-indicator", () => ({ RetryIndicator: () => null }));
vi.mock("@/components/agent-local/send-stop-button", () => ({ SendStopButton: () => null }));
vi.mock("@/hooks/use-session-compression-profile", () => ({
  useSessionCompressionProfile: () => ({
    effective: null,
    profiles: [],
    profilesStatus: "ready",
    select: vi.fn(),
  }),
}));

const owner = "com.example.surfaces";
const occupant = (id: string) => `extension:${owner}:${owner}.${id}` as ExtensionOccupantId;
const label = (defaultText: string) => ({ default: defaultText });

const catalog = {
  revision: 7,
  contributions: [
    contribution("nav", {
      type: "tab",
      placement: "app.navigation.primary",
      label: label("Extension tab"),
      icon: "sparkle",
      detail: { type: "text", text: label("Tab detail") },
    }),
    contribution("settings", {
      type: "settingsTab",
      placement: "settings.navigation.preferences",
      label: label("Extension settings"),
      icon: "gear",
      detail: { type: "text", text: label("Settings detail") },
    }),
    contribution("toolbar", {
      type: "action",
      placement: "app.toolbar.primary",
      label: label("Toolbar action"),
      icon: "activity",
      actionId: `${owner}.toolbar-run`,
    }),
    contribution("composer", {
      type: "action",
      placement: "agent.composer.leading",
      label: label("Composer action"),
      icon: "plus",
      actionId: `${owner}.composer-run`,
    }),
  ],
};

function contribution(id: string, value: Record<string, unknown>) {
  const contributionId = `${owner}.${id}`;
  return {
    extensionId: owner,
    contributionId,
    contribution: { id: contributionId, order: 5, ...value },
  };
}

function Providers({ children }: { children: React.ReactNode }) {
  return (
    <StandardCatalogProvider onOpenExtension={vi.fn()}>
      <SlotProvider>{children}</SlotProvider>
    </StandardCatalogProvider>
  );
}

describe("standard contribution surfaces", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_extension_ui_catalog") return Promise.resolve(catalog);
      if (command === "begin_extension_ui_load") return Promise.resolve([1, 2, 3]);
      if (command === "invoke_extension_ui_action") {
        return Promise.resolve({ type: "notification", level: "success", message: label("Done") });
      }
      return Promise.resolve(undefined);
    });
  });

  it("opens navigation, settings and toolbar occupants from one resolved registry", async () => {
    const onTabChange = vi.fn();
    render(
      <Providers>
        <ListPanelFooter activeTab={occupant("nav")} onTabChange={onTabChange} />
        <SettingsSubTabList active="general" onSelect={vi.fn()} />
        <WindowToolbar
          sidebarOpen
          onToggleSidebar={vi.fn()}
          onBack={vi.fn()}
          onForward={vi.fn()}
          onNewSession={vi.fn()}
          onSearch={vi.fn()}
          onToggleUpdates={vi.fn()}
          updatesCount={0}
          canGoBack={false}
          canGoForward={false}
        />
      </Providers>,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Extension tab" }));
    expect(onTabChange).toHaveBeenCalledWith(occupant("nav"));
    expect(await screen.findByText("Extension settings")).toBeTruthy();
    expect(await screen.findByRole("button", { name: "Toolbar action" })).toBeTruthy();
  });

  it("renders composer actions in Agent and Plan but never in Chat", async () => {
    const { rerender } = render(
      <Providers><Composer permissionMode="chat" planModeEnabled={false} /></Providers>,
    );
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("get_extension_ui_catalog"));
    expect(screen.queryByRole("button", { name: "Composer action" })).toBeNull();

    rerender(<Providers><Composer permissionMode="auto" planModeEnabled={false} /></Providers>);
    expect(await screen.findByRole("button", { name: "Composer action" })).toBeTruthy();

    rerender(<Providers><Composer permissionMode="manual" planModeEnabled /></Providers>);
    expect(await screen.findByRole("button", { name: "Composer action" })).toBeTruthy();

    rerender(<Providers><Composer permissionMode="chat" planModeEnabled={false} /></Providers>);
    await waitFor(() => expect(
      screen.queryByRole("button", { name: "Composer action" }),
    ).toBeNull());
  });
});

function Composer({
  permissionMode,
  planModeEnabled,
}: {
  permissionMode: PermissionMode;
  planModeEnabled: boolean;
}) {
  return (
    <ChatInputActionsRow
      modelName="model"
      providerName="provider"
      fastModeEnabled={false}
      fastModePending={false}
      contextUsed={0}
      contextMax={1}
      permissionMode={permissionMode}
      planModeEnabled={planModeEnabled}
      buttonState="hidden"
      onPermissionModeChange={vi.fn()}
      onFileImport={vi.fn()}
      onModelChange={vi.fn()}
      onReasoningModeChange={vi.fn()}
      onFastModeChange={vi.fn()}
      onSend={vi.fn()}
      onStop={vi.fn()}
    />
  );
}
