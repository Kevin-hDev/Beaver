import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ALT_LABEL, MOD_LABEL } from "@/lib/platform";
import { ShortcutsSettings } from "../shortcuts-settings";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

function displayedKeys(label: string): string[] {
  const row = screen.getByText(label).closest(".scs-row");
  return Array.from(row?.querySelectorAll("kbd") ?? [], (key) => key.textContent ?? "");
}

describe("ShortcutsSettings", () => {
  it("affiche la touche réellement utilisée pour le panneau de prévisualisation", () => {
    render(<ShortcutsSettings />);

    expect(displayedKeys("settings.shortcuts.togglePreview")).toEqual([
      ALT_LABEL,
      MOD_LABEL,
      "B",
    ]);
  });

  it.each([
    ["settings.shortcuts.zoomIn", [MOD_LABEL, "+"]],
    ["settings.shortcuts.zoomOut", [MOD_LABEL, "-"]],
    ["settings.shortcuts.resetZoom", [MOD_LABEL, "0"]],
    ["settings.shortcuts.openSettings", [MOD_LABEL, ","]],
    ["settings.shortcuts.searchConversation", [MOD_LABEL, "F"]],
    ["settings.shortcuts.focusComposer", [MOD_LABEL, "L"]],
    ["settings.shortcuts.selectSessionTab", [MOD_LABEL, "1–9"]],
    ["settings.shortcuts.changePermissions", ["Shift", "Tab"]],
    ["settings.shortcuts.sendMessage", ["Enter"]],
    ["settings.shortcuts.newLine", ["Shift", "Enter"]],
    ["settings.shortcuts.stopResponse", ["Esc", "Esc"]],
    ["settings.shortcuts.submitEdit", [MOD_LABEL, "Enter"]],
    ["settings.shortcuts.cancelEdit", ["Esc"]],
  ])("affiche le raccourci %s", (label, keys) => {
    render(<ShortcutsSettings />);

    expect(displayedKeys(label)).toEqual(keys);
  });
});
