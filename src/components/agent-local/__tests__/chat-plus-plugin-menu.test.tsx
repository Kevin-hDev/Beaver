import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import {
  ChatPlusPluginMenu,
  chatPluginShortcuts,
} from "../chat-plus-plugin-menu";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

function extension(index: number, showInChat = true): ExtensionRecord {
  return {
    manifest: {
      id: `com.example.plugin-${index}`,
      name: `Plugin ${index}`,
      version: "1.0.0",
      beaverApi: "1",
      runtime: "node",
      access: "full",
      apiLevel: "stable",
    },
    kind: "local",
    source: "/extension",
    enabled: index % 2 === 0,
    trusted: false,
    showInChat,
    status: "active",
    contributions: { tools: [], events: [] },
  };
}

describe("ChatPlusPluginMenu", () => {
  it("ne conserve que les raccourcis choisis sans limiter leur nombre", () => {
    const records = Array.from({ length: 12 }, (_, index) => extension(index, index !== 5));
    const shortcuts = chatPluginShortcuts(records);

    expect(shortcuts).toHaveLength(11);
    expect(shortcuts.some((item) => item.manifest.id === "com.example.plugin-5")).toBe(false);
  });

  it("modifie directement l’état global enabled du plugin", () => {
    const onToggle = vi.fn();
    const view = render(
      <ChatPlusPluginMenu
        extensions={[extension(0)]}
        busyIds={new Set()}
        onToggle={onToggle}
      />,
    );

    fireEvent.click(view.getByRole("switch", { name: "Plugin 0" }));
    expect(onToggle).toHaveBeenCalledWith("com.example.plugin-0", false);
  });

  it("rend tous les raccourcis sélectionnés au-delà de huit éléments", () => {
    const records = Array.from({ length: 12 }, (_, index) => extension(index));
    const view = render(
      <ChatPlusPluginMenu
        extensions={records}
        busyIds={new Set()}
        onToggle={vi.fn()}
      />,
    );

    expect(view.getAllByRole("switch")).toHaveLength(12);
  });
});
