/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UI_PLACEMENTS } from "@/types/extension-ui-contract.generated";
import { CORE_SLOT_OCCUPANTS } from "@/features/extension-ui/core-occupants";
import { SlotResolutionContext } from "@/features/extension-ui/slot-contexts";
import { createSlotRegistry } from "@/features/extension-ui/slot-registry";
import { resolveSlots } from "@/features/extension-ui/slot-resolution";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import { ChatInputActionsRow } from "../chat-input-actions-row";

vi.mock("@/hooks/use-session-compression-profile", () => ({
  useSessionCompressionProfile: () => ({
    profiles: [], profilesStatus: "ready", effective: undefined, select: vi.fn(),
  }),
}));
vi.mock("../chat-plus-menu", () => ({
  ChatPlusMenu: ({
    agentic,
    showCompression,
    onFileImport,
  }: {
    agentic: boolean;
    showCompression: boolean;
    onFileImport: () => void;
  }) => (
    <button
      data-testid="plus-menu"
      data-agentic={String(agentic)}
      data-compression={String(showCompression)}
      onClick={onFileImport}
    >plus</button>
  ),
}));
vi.mock("../context-progress", () => ({ ContextProgress: () => null }));
vi.mock("../model-controls", () => ({ ModelControls: () => null }));
vi.mock("../permission-mode-selector", () => ({ PermissionModeSelector: () => null }));
vi.mock("../missing-directory-prompt", () => ({ MissingDirectoryPrompt: () => null }));
vi.mock("../plan-mode-badge", () => ({ PlanModeBadge: () => null }));
vi.mock("../retry-indicator", () => ({ RetryIndicator: () => null }));
vi.mock("../send-stop-button", () => ({ SendStopButton: () => null }));

afterEach(cleanup);

function renderRow(
  ids: readonly string[],
  permissionMode: PermissionMode,
  planModeEnabled: boolean,
  onFileImport = vi.fn(),
) {
  const occupants = CORE_SLOT_OCCUPANTS.filter(({ id }) => ids.includes(id));
  const resolution = resolveSlots(createSlotRegistry(UI_PLACEMENTS, occupants), []);
  return render(
    <SlotResolutionContext.Provider value={resolution}>
      <ChatInputActionsRow
        sessionId="session-1"
        modelName="model"
        providerName="provider"
        fastModeEnabled={false}
        fastModePending={false}
        contextUsed={0}
        contextMax={1}
        permissionMode={permissionMode}
        planModeEnabled={planModeEnabled}
        buttonState="send"
        onPermissionModeChange={vi.fn()}
        onFileImport={onFileImport}
        onModelChange={vi.fn()}
        onReasoningModeChange={vi.fn()}
        onFastModeChange={vi.fn()}
        onSend={vi.fn()}
        onStop={vi.fn()}
      />
    </SlotResolutionContext.Provider>,
  );
}

describe("ChatInputActionsRow slot routing", () => {
  it("does not invent the core menu when its occupant is absent", () => {
    renderRow(["beaver.settings"], "chat", false);

    expect(screen.queryByTestId("plus-menu")).toBeNull();
  });

  it.each([
    ["chat", false],
    ["auto", false],
    ["manual", true],
  ] as const)("keeps the core menu in mode %s plan=%s", (permissionMode, planModeEnabled) => {
    const onFileImport = vi.fn();
    renderRow(["beaver.composer-menu"], permissionMode, planModeEnabled, onFileImport);

    const menu = screen.getByTestId("plus-menu");
    expect(menu).toHaveAttribute("data-agentic", String(permissionMode !== "chat"));
    expect(menu).toHaveAttribute("data-compression", "true");
    menu.click();
    expect(onFileImport).toHaveBeenCalledOnce();
  });
});
