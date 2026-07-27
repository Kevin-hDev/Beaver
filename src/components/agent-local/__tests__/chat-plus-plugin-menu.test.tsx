import { readFileSync } from "node:fs";
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

  it("affiche huit lignes puis active le défilement", () => {
    const css = readFileSync("src/components/agent-local/chat-plus-menu.css", "utf8");
    const tokens = readFileSync("src/styles/tokens.css", "utf8");

    expect(css).toMatch(/\.cpm-plugin-list\s*\{[^}]*overflow-y:\s*auto;/s);
    expect(tokens).toContain(
      "--extensions-chat-list-max-height: calc(var(--extensions-chat-row-height) * 8);",
    );
  });
});
