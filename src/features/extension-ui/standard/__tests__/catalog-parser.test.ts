import { describe, expect, it } from "vitest";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import { parseStandardCatalog } from "../catalog-parser";

function snapshot(contribution: Record<string, unknown>) {
  return {
    revision: 1,
    contributions: [{
      extensionId: "com.example.ui",
      contributionId: "com.example.ui.panel",
      contribution,
    }],
  };
}

function tab(detail: unknown = { type: "text", text: { default: "Hello", zh: "你好", ja: "こんにちは" } }) {
  return {
    type: "tab", id: "com.example.ui.panel", placement: "app.navigation.primary",
    order: 1, label: { default: "Panel" }, icon: "puzzle-piece", detail,
  };
}

describe("parseStandardCatalog", () => {
  it("parses localized tabs without forwarding unknown properties", () => {
    const parsed = parseStandardCatalog(snapshot(tab()));
    expect(parsed.contributions[0].contribution.type).toBe("tab");
    expect(() => parseStandardCatalog(snapshot({ ...tab(), onClick: "unsafe" }))).toThrow();
  });

  it("rejects mismatched ownership, type and field values", () => {
    expect(() => parseStandardCatalog({ ...snapshot(tab()), contributions: [{
      extensionId: "com.other", contributionId: "com.example.ui.panel", contribution: tab(),
    }] })).toThrow();
    expect(() => parseStandardCatalog(snapshot({ ...tab(), placement: "app.toolbar.primary" }))).toThrow();
    expect(() => parseStandardCatalog(snapshot(tab({
      type: "textField", id: "com.example.ui.name", label: { default: "Name" }, value: [],
    })))).toThrow();
    expect(() => parseStandardCatalog(snapshot(tab({
      type: "toggle", id: "com.example.ui.toggle", label: { default: "Toggle" }, value: "yes",
    })))).toThrow();
    expect(() => parseStandardCatalog(snapshot(tab({
      type: "select", id: "com.example.ui.select", label: { default: "Select" }, value: "missing",
      options: [{ value: "known", label: { default: "Known" } }],
    })))).toThrow();
  });

  it("rejects duplicate node and select option identifiers", () => {
    expect(() => parseStandardCatalog(snapshot(tab({ type: "stack", children: [
      { type: "textField", id: "com.example.ui.same", label: { default: "First" }, value: "" },
      { type: "toggle", id: "com.example.ui.same", label: { default: "Second" }, value: false },
    ] })))).toThrow();
    expect(() => parseStandardCatalog(snapshot(tab({
      type: "select", id: "com.example.ui.select", label: { default: "Select" }, value: "same",
      options: [
        { value: "same", label: { default: "First" } },
        { value: "same", label: { default: "Second" } },
      ],
    })))).toThrow();
  });

  it("bounds depth and node count before accepting a view", () => {
    let deep: unknown = { type: "text", text: { default: "end" } };
    for (let index = 0; index < UI_LIMITS.maxViewDepth; index += 1) {
      deep = { type: "stack", children: [deep] };
    }
    expect(() => parseStandardCatalog(snapshot(tab(deep)))).toThrow();
    const children = Array.from({ length: UI_LIMITS.maxViewNodes }, () => ({
      type: "separator",
    }));
    expect(() => parseStandardCatalog(snapshot(tab({ type: "stack", children })))).toThrow();
  });

  it("rejects duplicate entries and stale-shaped envelopes", () => {
    const valid = snapshot(tab());
    expect(() => parseStandardCatalog({ ...valid, contributions: [
      ...valid.contributions, ...valid.contributions,
    ] })).toThrow();
    expect(() => parseStandardCatalog({ ...valid, revision: -1 })).toThrow();
    expect(() => parseStandardCatalog({ ...valid, extra: true })).toThrow();
  });

  it("revalidates theme and action budgets at the TypeScript boundary", () => {
    const themes = Array.from({ length: UI_LIMITS.maxThemesPerExtension + 1 }, (_, index) => ({
      extensionId: "com.example.ui",
      contributionId: `com.example.ui.theme-${index}`,
      contribution: {
        type: "theme", id: `com.example.ui.theme-${index}`, order: index,
        label: { default: `Theme ${index}` }, base: "dark", tokens: {},
      },
    }));
    expect(() => parseStandardCatalog({ revision: 1, contributions: themes })).toThrow();

    const buttons = Array.from({ length: UI_LIMITS.maxActionsPerExtension + 1 }, (_, index) => ({
      type: "button", id: `com.example.ui.button-${index}`,
      label: { default: `Button ${index}` }, actionId: `com.example.ui.action-${index}`,
    }));
    expect(() => parseStandardCatalog(snapshot(tab({ type: "stack", children: buttons })))).toThrow();
  });
});
