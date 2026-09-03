/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import i18n from "@/i18n";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { StandardCatalogProvider, useStandardCatalog } from "../catalog-context";
import { StandardContributionBoundary } from "../contribution-boundary";
import { localizedText } from "../localized-text";
import { parseStandardCatalog } from "../catalog-parser";
import { StandardViewRenderer } from "../view-renderer";
import {
  StandardPlacementAction,
  StandardSettingsContent,
  StandardTabContent,
} from "../standard-contributions";
import { PanelSlotProvider, PanelSlotTarget } from "@/components/ui/panel-slots";
import type { StandardView } from "../types";

const owner = "com.example.ui";
const contributionId = `${owner}.panel`;
const action = `${owner}.run`;

function text(value: string) {
  return { default: value, fr: `FR ${value}`, de: `DE ${value}`, zh: `中${value}`, ja: `日${value}` };
}

function catalog(detail: StandardView) {
  return {
    revision: 1,
    contributions: [{
      extensionId: owner,
      contributionId,
      contribution: {
        type: "tab",
        id: contributionId,
        placement: "app.navigation.primary",
        order: 1,
        label: text("Panel"),
        icon: "puzzle-piece",
        detail,
      },
    }],
  };
}

function entry(detail: StandardView) {
  return parseStandardCatalog(catalog(detail)).contributions[0];
}

function renderView(view: StandardView) {
  const parsed = entry(view);
  return render(
    <StandardCatalogProvider onOpenExtension={vi.fn()}>
      <StandardViewRenderer entry={parsed} view={view} />
    </StandardCatalogProvider>,
  );
}

describe("standard UI renderer", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockImplementation((command) => Promise.resolve(
      command === "get_extension_ui_catalog" ? catalog({ type: "text", text: text("ready") }) : undefined,
    ));
  });

  it("renders every standard primitive with explicit DOM properties", () => {
    renderView({
      type: "stack",
      children: [
        { type: "row", children: [
          { type: "heading", text: text("Heading") },
          { type: "text", text: text("Body") },
          { type: "badge", text: text("Badge") },
        ] },
        { type: "separator" },
        { type: "textField", id: `${owner}.text`, label: text("Text"), value: "hello" },
        { type: "numberField", id: `${owner}.number`, label: text("Number"), value: 3 },
        { type: "toggle", id: `${owner}.toggle`, label: text("Toggle"), value: true },
        { type: "select", id: `${owner}.select`, label: text("Select"), value: "a", options: [
          { value: "a", label: text("Option") },
        ] },
        { type: "button", id: `${owner}.button`, label: text("Run"), actionId: action },
      ],
    });

    expect(screen.getByText("Heading")).toBeTruthy();
    expect(screen.getByText("Body")).toBeTruthy();
    expect(screen.getByText("Badge")).toBeTruthy();
    expect(screen.getByLabelText("Text")).toHaveValue("hello");
    expect(screen.getByLabelText("Number")).toHaveValue(3);
    expect(screen.getByLabelText("Toggle")).toBeChecked();
    expect(screen.getByLabelText("Select")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Run" })).not.toHaveAttribute("onclick");
  });

  it("selects long and unsegmented localized text without truncating it", async () => {
    await i18n.changeLanguage("en");
    const value = {
      default: "Default",
      fr: "Une description française volontairement très longue",
      de: "Eine absichtlich sehr lange deutsche Beschreibung",
      zh: "这是没有空格的中文说明",
      ja: "これは空白のない日本語の説明です",
    };
    expect(localizedText(value, "fr-FR")).toBe(value.fr);
    expect(localizedText(value, "de-DE")).toBe(value.de);
    expect(localizedText(value, "zh-CN")).toBe(value.zh);
    expect(localizedText(value, "ja-JP")).toBe(value.ja);
  });

  it("renders tab list/detail and settings detail in their Beaver surfaces", async () => {
    vi.mocked(invoke).mockImplementation((command) => Promise.resolve(
      command === "get_extension_ui_catalog" ? panelsCatalog() : undefined,
    ));
    render(
      <StandardCatalogProvider onOpenExtension={vi.fn()}>
        <PanelSlotProvider>
          <PanelSlotTarget name="list" />
          <PanelSlotTarget name="detail" />
          <CatalogPanels />
        </PanelSlotProvider>
      </StandardCatalogProvider>,
    );

    expect(await screen.findByText("Tab list")).toBeTruthy();
    expect(await screen.findByText("Tab detail")).toBeTruthy();
    expect(await screen.findByText("Settings detail")).toBeTruthy();
  });

  it("ignores double clicks and drops fields orphaned by a replacement view", async () => {
    let release!: (value: unknown) => void;
    let actionCalls = 0;
    vi.mocked(invoke).mockImplementation((command, args) => {
      if (command === "get_extension_ui_catalog") return Promise.resolve(catalog(initialView()));
      if (command !== "invoke_extension_ui_action") return Promise.resolve(undefined);
      actionCalls += 1;
      if (actionCalls === 1) {
        expect(args).toMatchObject({ payload: { fields: { [`${owner}.old`]: "edited" } } });
        return new Promise((resolve) => { release = resolve; });
      }
      expect(args).toMatchObject({ payload: { fields: { [`${owner}.new`]: "fresh" } } });
      return Promise.resolve({ type: "notification", level: "success", message: text("Done") });
    });
    renderView(initialView());

    fireEvent.change(screen.getByLabelText("Old"), { target: { value: "edited" } });
    const run = screen.getByRole("button", { name: "Run" });
    fireEvent.click(run);
    fireEvent.click(run);
    expect(actionCalls).toBe(1);
    release({ type: "view", view: replacementView() });

    await waitFor(() => expect(screen.getByLabelText("New")).toBeTruthy());
    expect(screen.queryByLabelText("Old")).toBeNull();
    fireEvent.change(screen.getByLabelText("New"), { target: { value: "fresh" } });
    fireEvent.click(screen.getByRole("button", { name: "Run again" }));
    await waitFor(() => expect(actionCalls).toBe(2));
  });

  it("serializes an emptied number field as null", async () => {
    const view: StandardView = {
      type: "stack",
      children: [
        { type: "numberField", id: `${owner}.amount`, label: text("Amount"), value: 7 },
        { type: "button", id: `${owner}.submit`, label: text("Submit"), actionId: action },
      ],
    };
    vi.mocked(invoke).mockImplementation((command, args) => {
      if (command === "get_extension_ui_catalog") return Promise.resolve(catalog(view));
      if (command === "invoke_extension_ui_action") {
        expect(args).toMatchObject({ payload: { fields: { [`${owner}.amount`]: null } } });
        return Promise.resolve({ type: "notification", level: "success", message: text("Done") });
      }
      return Promise.resolve(undefined);
    });
    renderView(view);

    fireEvent.change(screen.getByLabelText("Amount"), { target: { value: "" } });
    fireEvent.click(screen.getByRole("button", { name: "Submit" }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith(
      "invoke_extension_ui_action",
      expect.objectContaining({ payload: { fields: { [`${owner}.amount`]: null } } }),
    ));
  });

  it("releases the action button after a backend timeout", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_extension_ui_catalog") return Promise.resolve(catalog(initialView()));
      if (command === "invoke_extension_ui_action") return Promise.reject(new Error("timeout"));
      return Promise.resolve(undefined);
    });
    renderView(initialView());

    const run = screen.getByRole("button", { name: "Run" });
    fireEvent.click(run);
    expect(run).toBeDisabled();
    await waitFor(() => expect(run).not.toBeDisabled());
  });

  it("contains a render crash to one contribution and reports a bounded diagnostic", async () => {
    const open = vi.fn();
    const boundaryOwner = "beaver.test";
    const boundaryId = `${boundaryOwner}.panel`;
    const parsed = parseStandardCatalog({
      revision: 1,
      contributions: [{
        extensionId: boundaryOwner,
        contributionId: boundaryId,
        contribution: {
          type: "tab",
          id: boundaryId,
          placement: "app.navigation.primary",
          order: 1,
          label: text("Panel"),
          detail: { type: "text", text: text("ok") },
        },
      }],
    }).contributions[0];
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    function Broken(): never { throw new Error("private stack"); }
    try {
      render(
        <StandardCatalogProvider onOpenExtension={open}>
          <StandardContributionBoundary entry={parsed}><Broken /></StandardContributionBoundary>
        </StandardCatalogProvider>,
      );
      expect(await screen.findByRole("alert")).toHaveTextContent("This extension could not be displayed.");
      await waitFor(() => expect(invoke).toHaveBeenCalledWith(
        "report_extension_ui_mount_failure",
        { extensionId: boundaryOwner, contributionId: boundaryId },
      ));
      fireEvent.click(screen.getByRole("button", { name: "Open detail" }));
      expect(open).toHaveBeenCalledWith(boundaryOwner);
    } finally {
      consoleError.mockRestore();
    }
  });

  it("ignores an in-flight response after the contribution is disabled", async () => {
    let changed: ((event: { payload: number }) => void) | undefined;
    let release!: (value: unknown) => void;
    const actionCatalog = actionSnapshot(1);
    vi.mocked(listen).mockImplementation((_event, handler) => {
      changed = handler as (event: { payload: number }) => void;
      return Promise.resolve(() => {});
    });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_extension_ui_catalog") return Promise.resolve(actionCatalog);
      if (command === "invoke_extension_ui_action") {
        return new Promise((resolve) => { release = resolve; });
      }
      return Promise.resolve(undefined);
    });
    render(
      <StandardCatalogProvider onOpenExtension={vi.fn()}>
        <CatalogAction />
      </StandardCatalogProvider>,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Catalog action" }));
    vi.mocked(invoke).mockImplementation((command) => Promise.resolve(
      command === "get_extension_ui_catalog" ? { revision: 2, contributions: [] } : undefined,
    ));
    changed?.({ payload: 2 });
    await waitFor(() => expect(
      screen.queryByRole("button", { name: "Catalog action" }),
    ).toBeNull());

    release({ type: "view", view: { type: "text", text: text("Stale content") } });
    await Promise.resolve();
    expect(screen.queryByText("Stale content")).toBeNull();
  });

  it("ports a composer result above the trigger and closes it with Escape or its button", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_extension_ui_catalog") return Promise.resolve(actionSnapshot(1));
      if (command === "invoke_extension_ui_action") {
        return Promise.resolve({
          type: "view",
          view: { type: "text", text: text("Action result") },
        });
      }
      return Promise.resolve(undefined);
    });
    render(
      <StandardCatalogProvider onOpenExtension={vi.fn()}>
        <CatalogAction surface="composer" />
      </StandardCatalogProvider>,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Catalog action" }));
    expect(await screen.findByText("Action result")).toBeInTheDocument();
    const panel = document.body.querySelector(".xui-action-result");
    expect(panel).toHaveClass("xui-action-result-composer");
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByText("Action result")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Catalog action" }));
    await screen.findByText("Action result");
    fireEvent.click(screen.getByRole("button", { name: i18n.t("a11y.close") }));
    expect(screen.queryByText("Action result")).toBeNull();
  });

  it("acknowledges the remount after the same contribution changes revision", async () => {
    let changed: ((event: { payload: number }) => void) | undefined;
    let current = actionSnapshot(1);
    vi.mocked(listen).mockImplementation((_event, handler) => {
      changed = handler as (event: { payload: number }) => void;
      return Promise.resolve(() => {});
    });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_extension_ui_catalog") return Promise.resolve(current);
      if (command === "begin_extension_ui_load") return Promise.resolve([1, 2, 3]);
      return Promise.resolve(undefined);
    });
    render(
      <StandardCatalogProvider onOpenExtension={vi.fn()}>
        <CatalogAction />
      </StandardCatalogProvider>,
    );

    await waitFor(() => expect(acknowledgements()).toBe(1));
    current = actionSnapshot(2);
    changed?.({ payload: 2 });
    await waitFor(() => expect(acknowledgements()).toBe(2));
  });
});

function acknowledgements(): number {
  return vi.mocked(invoke).mock.calls.filter(([command]) =>
    command === "acknowledge_extension_ui_load").length;
}

function initialView(): StandardView {
  return { type: "stack", children: [
    { type: "textField", id: `${owner}.old`, label: text("Old"), value: "" },
    { type: "button", id: `${owner}.run-button`, label: text("Run"), actionId: action },
  ] };
}

function replacementView(): StandardView {
  return { type: "stack", children: [
    { type: "textField", id: `${owner}.new`, label: text("New"), value: "" },
    { type: "button", id: `${owner}.again-button`, label: text("Run again"), actionId: `${owner}.again` },
  ] };
}

function CatalogPanels() {
  const snapshot = useStandardCatalog().snapshot;
  if (!snapshot) return null;
  const tabEntry = snapshot.contributions.find(({ contribution }) => contribution.type === "tab");
  const settingsEntry = snapshot.contributions.find(
    ({ contribution }) => contribution.type === "settingsTab",
  );
  if (!tabEntry || !settingsEntry) return null;
  return (
    <>
      <StandardTabContent entry={tabEntry} />
      <StandardSettingsContent entry={settingsEntry} />
    </>
  );
}

function panelsCatalog() {
  const tab = catalog({ type: "text", text: text("Tab detail") });
  (tab.contributions[0].contribution as Record<string, unknown>).list = {
    type: "text",
    text: text("Tab list"),
  };
  return {
    ...tab,
    contributions: [
      ...tab.contributions,
      {
        extensionId: owner,
        contributionId: `${owner}.settings`,
        contribution: {
          type: "settingsTab",
          id: `${owner}.settings`,
          placement: "settings.navigation.preferences",
          order: 2,
          label: text("Settings"),
          detail: { type: "text", text: text("Settings detail") },
        },
      },
    ],
  };
}

function actionSnapshot(revision: number) {
  return {
    revision,
    contributions: [{
      extensionId: owner,
      contributionId: `${owner}.catalog-action`,
      contribution: {
        type: "action",
        id: `${owner}.catalog-action`,
        placement: "app.toolbar.primary",
        order: 1,
        label: text("Catalog action"),
        actionId: `${owner}.catalog-run`,
      },
    }],
  };
}

function CatalogAction({ surface = "toolbar" }: { surface?: "toolbar" | "composer" }) {
  const entry = useStandardCatalog().snapshot?.contributions[0];
  return entry ? <StandardPlacementAction entry={entry} surface={surface} /> : null;
}
