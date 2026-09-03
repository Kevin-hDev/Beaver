import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";
import { LIMITS } from "../../src-tauri/resources/extension-host/contract.mjs";
import {
  clearUiActions,
  invokeUiAction,
} from "../../src-tauri/resources/extension-host/ui-actions.mjs";
import {
  UI_LIMITS,
  UI_LOCALES,
  UI_THEME_TOKENS,
} from "../../src-tauri/resources/extension-host/ui-contract.mjs";
import {
  jsonBytes,
  validateActionPayload,
  validateActionResult,
} from "../../src-tauri/resources/extension-host/ui-validation.mjs";
import { createHost, resetAndLoad } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostScript = join(root, "src-tauri/target/extension-host/host.mjs");

function specification(id = "com.example.ui") {
  return {
    id,
    mainPath: "/tmp/index.mjs",
    manifest: {
      apiLevel: "stable",
      ui: { apiVersion: "1", mode: "standard" },
    },
  };
}

function action(id) {
  return {
    type: "action",
    id,
    placement: "app.toolbar.primary",
    order: 0,
    label: { default: `Action ${id}` },
    icon: "sparkle",
    actionId: id,
  };
}

function tabWithFields(count, targetId = undefined) {
  return {
    type: "tab",
    id: "panel",
    placement: "app.navigation.primary",
    order: 0,
    label: { default: "Panel" },
    ...(targetId ? { operation: "after", targetId } : {}),
    detail: {
      type: "stack",
      children: Array.from({ length: count }, (_, index) => ({
        type: index === 0 ? "button" : "textField",
        id: `field-${index}`,
        label: { default: `Field ${index}` },
        ...(index === 0 ? { actionId: "nested-run" } : { value: "" }),
      })),
    },
  };
}

function theme(id) {
  return {
    type: "theme", id, order: 0, label: { default: id }, base: "dark",
    tokens: { "--surface": "#112233" },
  };
}

function actionPanel(id, count) {
  return {
    type: "settingsTab",
    id,
    placement: "settings.navigation.preferences",
    order: 0,
    label: { default: id },
    detail: {
      type: "stack",
      children: Array.from({ length: count }, (_, index) => ({
        type: "button",
        id: `${id}-button-${index}`,
        label: { default: `Button ${index}` },
        actionId: `${id}-action-${index}`,
      })),
    },
  };
}

function tabWithNodes(count) {
  return {
    type: "tab", id: "nodes", placement: "app.navigation.primary", order: 0,
    label: { default: "Nodes" },
    detail: {
      type: "stack",
      children: Array.from({ length: count - 1 }, () => ({
        type: "text", text: { default: "node" },
      })),
    },
  };
}

function tabAtDepth(depth) {
  let detail = { type: "text", text: { default: "leaf" } };
  for (let index = 1; index < depth; index += 1) {
    detail = { type: "stack", children: [detail] };
  }
  return {
    type: "tab", id: "depth", placement: "app.navigation.primary", order: 0,
    label: { default: "Depth" }, detail,
  };
}

function contributionsAtBytes(target) {
  const items = Array.from({ length: 17 }, (_, index) => ({
    ...action(`com.example.ui.sized-${index}`),
    label: Object.fromEntries(["default", ...UI_LOCALES]
      .map((locale) => [locale, "x".repeat(1800)])),
  }));
  let remaining = target - jsonBytes(items);
  assert.ok(remaining >= 0);
  for (const item of items) {
    for (const locale of ["default", ...UI_LOCALES]) {
      if (remaining === 0) return items;
      const room = UI_LIMITS.maxTextChars - item.label[locale].length;
      const added = Math.min(room, remaining);
      item.label[locale] += "x".repeat(added);
      remaining -= added;
    }
  }
  assert.equal(remaining, 0);
  return items;
}

function fillStringSlotsToBytes(slots, root, target) {
  let remaining = target - jsonBytes(root);
  assert.ok(remaining >= 0);
  for (const [object, key] of slots) {
    const room = UI_LIMITS.maxTextChars - [...object[key]].length;
    const added = Math.min(room, remaining);
    object[key] += "x".repeat(added);
    remaining -= added;
    if (remaining === 0) return;
  }
  assert.equal(remaining, 0);
}

function growFirstStringSlot(slots) {
  const slot = slots.find(([object, key]) => [...object[key]].length < UI_LIMITS.maxTextChars);
  assert.ok(slot);
  slot[0][slot[1]] += "x";
}

test("UI registrations are bounded and unsubscribe idempotently", () => {
  const context = createExtensionApi(specification());
  const unsubscribe = [];
  for (let index = 0; index < UI_LIMITS.maxContributionsPerExtension; index += 1) {
    unsubscribe.push(context.api.ui.register(action(`action-${index}`)));
  }
  assert.equal(context.ui.contributions.length, UI_LIMITS.maxContributionsPerExtension);

  const rejected = context.api.ui.register(action("overflow"));
  assert.equal(context.ui.contributions.length, UI_LIMITS.maxContributionsPerExtension);
  assert.equal(context.ui.diagnostics.at(-1).code, "ui_limit_exceeded");

  unsubscribe[0]();
  unsubscribe[0]();
  rejected();
  context.api.ui.register(action("replacement"));
  assert.equal(context.ui.contributions.length, UI_LIMITS.maxContributionsPerExtension);
});

test("aggregate UI bytes and themes accept max then reject max plus one", () => {
  const atLimit = contributionsAtBytes(UI_LIMITS.maxUiBytesPerExtension);
  assert.equal(jsonBytes(atLimit), UI_LIMITS.maxUiBytesPerExtension);
  const context = createExtensionApi(specification());
  const unsubscribes = atLimit.map((item) => context.api.ui.register(item));
  assert.equal(context.ui.contributions.length, atLimit.length);

  const over = structuredClone(atLimit);
  const locale = ["default", ...UI_LOCALES]
    .find((candidate) => over.at(-1).label[candidate].length < UI_LIMITS.maxTextChars);
  assert.ok(locale);
  over.at(-1).label[locale] += "x";
  assert.equal(jsonBytes(over), UI_LIMITS.maxUiBytesPerExtension + 1);
  const overflow = createExtensionApi(specification());
  over.forEach((item) => overflow.api.ui.register(item));
  assert.equal(overflow.ui.contributions.length, over.length - 1);
  assert.equal(overflow.ui.diagnostics.at(-1).code, "ui_limit_exceeded");

  unsubscribes.at(-1)();
  context.api.ui.register(atLimit.at(-1));
  assert.equal(context.ui.contributions.length, atLimit.length);

  const themes = createExtensionApi(specification());
  const removeThemes = Array.from({ length: UI_LIMITS.maxThemesPerExtension }, (_, index) =>
    themes.api.ui.register(theme(`theme-${index}`)));
  themes.api.ui.register(theme("theme-overflow"));
  assert.equal(themes.ui.contributions.length, UI_LIMITS.maxThemesPerExtension);
  removeThemes[0]();
  themes.api.ui.register(theme("theme-replacement"));
  assert.equal(themes.ui.contributions.length, UI_LIMITS.maxThemesPerExtension);
});

test("view fields are bounded and nested IDs plus targets are canonicalized", () => {
  const atLimit = createExtensionApi(specification());
  atLimit.api.ui.register(tabWithFields(UI_LIMITS.maxFieldsPerView, "sibling"));
  assert.equal(atLimit.ui.contributions.length, 1);
  const contribution = atLimit.ui.contributions[0];
  assert.equal(contribution.targetId, "com.example.ui.sibling");
  assert.equal(contribution.detail.children[0].actionId, "com.example.ui.nested-run");
  assert.equal(contribution.detail.children[0].id, "com.example.ui.field-0");

  const overflow = createExtensionApi(specification());
  overflow.api.ui.register(tabWithFields(UI_LIMITS.maxFieldsPerView + 1));
  assert.equal(overflow.ui.contributions.length, 0);
  assert.equal(overflow.ui.diagnostics.at(-1).code, "ui_limit_exceeded");
});

test("view nodes, depth, options, text and published theme tokens are bounded", () => {
  const accepts = [
    tabWithNodes(UI_LIMITS.maxViewNodes),
    tabAtDepth(UI_LIMITS.maxViewDepth),
    {
      type: "settingsTab", id: "options", placement: "settings.navigation.preferences",
      order: 0, label: { default: "Options" }, detail: {
        type: "select", id: "choice", label: { default: "Choice" }, value: "",
        options: Array.from({ length: UI_LIMITS.maxOptionsPerField }, (_, index) => ({
          value: `choice-${index}`, label: { default: "Choice" },
        })),
      },
    },
    {
      ...action("text-limit"),
      label: { default: "x".repeat(UI_LIMITS.maxTextChars) },
    },
    {
      type: "theme", id: "all-public-tokens", order: 0,
      label: { default: "Theme" }, base: "dark",
      tokens: Object.fromEntries(UI_THEME_TOKENS.map((token) => [token, "#112233"])),
    },
  ];
  for (const contribution of accepts) {
    const context = createExtensionApi(specification());
    context.api.ui.register(contribution);
    assert.equal(context.ui.contributions.length, 1);
  }

  const rejects = [
    tabWithNodes(UI_LIMITS.maxViewNodes + 1),
    tabAtDepth(UI_LIMITS.maxViewDepth + 1),
    {
      ...accepts[2],
      detail: {
        ...accepts[2].detail,
        options: [...accepts[2].detail.options, { value: "overflow", label: { default: "Overflow" } }],
      },
    },
    { ...action("text-overflow"), label: { default: "x".repeat(UI_LIMITS.maxTextChars + 1) } },
    { ...accepts[4], tokens: { ...accepts[4].tokens, "--unknown": "#112233" } },
  ];
  for (const contribution of rejects) {
    const context = createExtensionApi(specification());
    context.api.ui.register(contribution);
    assert.equal(context.ui.contributions.length, 0);
  }
});

test("action payload and result byte limits accept max then reject max plus one", () => {
  const fields = {};
  for (let index = 0; index < UI_LIMITS.maxFieldsPerView; index += 1) {
    const prefix = `field-${index}-`;
    fields[`${prefix}${"x".repeat(LIMITS.maxIdentifierChars - prefix.length)}`] = "x";
  }
  const payload = { fields };
  const payloadSlots = Object.keys(fields).map((key) => [fields, key]);
  fillStringSlotsToBytes(payloadSlots, payload, UI_LIMITS.maxActionPayloadBytes);
  assert.equal(jsonBytes(payload), UI_LIMITS.maxActionPayloadBytes);
  assert.doesNotThrow(() => validateActionPayload(payload));
  growFirstStringSlot(payloadSlots);
  assert.equal(jsonBytes(payload), UI_LIMITS.maxActionPayloadBytes + 1);
  assert.throws(() => validateActionPayload(payload));

  const result = {
    type: "view",
    view: {
      type: "stack",
      children: Array.from({ length: 17 }, () => ({
        type: "text",
        text: Object.fromEntries(["default", ...UI_LOCALES].map((locale) => [locale, "x"])),
      })),
    },
  };
  const texts = result.view.children.flatMap((node) =>
    Object.keys(node.text).map((key) => [node.text, key]));
  fillStringSlotsToBytes(texts, result, UI_LIMITS.maxActionResultBytes);
  assert.equal(jsonBytes(result), UI_LIMITS.maxActionResultBytes);
  assert.doesNotThrow(() => validateActionResult("com.example.ui", result));
  growFirstStringSlot(texts);
  assert.equal(jsonBytes(result), UI_LIMITS.maxActionResultBytes + 1);
  assert.throws(() => validateActionResult("com.example.ui", result));
});

test("declared actions are bounded globally and unsubscribe releases admission", () => {
  const context = createExtensionApi(specification());
  const first = context.api.ui.register(actionPanel("first", 32));
  context.api.ui.register(actionPanel("second", 32));
  assert.equal(context.ui.contributions.length, 2);
  context.api.ui.register(action("overflow-action"));
  assert.equal(context.ui.contributions.length, 2);
  assert.equal(context.ui.diagnostics.at(-1).code, "ui_limit_exceeded");
  first();
  context.api.ui.register(action("replacement-action"));
  assert.equal(context.ui.contributions.length, 2);
});

test("nested button actions stay owner-bound", async () => {
  const context = createExtensionApi(specification());
  context.api.ui.register(tabWithFields(1));
  context.api.ui.onAction("nested-run", () => ({
    type: "notification", level: "success", message: { default: "nested" },
  }));
  const extension = { context };
  const params = {
    extensionId: "com.example.ui",
    contributionId: "panel",
    actionId: "nested-run",
    payload: { fields: {} },
    context: { locale: "en" },
  };
  assert.equal((await invokeUiAction(extension, params)).message.default, "nested");
  await assert.rejects(invokeUiAction(extension, {
    ...params, extensionId: "com.example.other",
  }));
});

test("a returned view keeps its follow-up button bound to the same owner", async () => {
  const context = createExtensionApi(specification());
  context.api.ui.register(action("open"));
  context.api.ui.onAction("open", () => ({
    type: "view",
    view: {
      type: "button", id: "confirm", label: { default: "Confirm" },
      actionId: "confirm",
    },
  }));
  context.api.ui.onAction("confirm", () => ({
    type: "notification", level: "success", message: { default: "Confirmed" },
  }));
  const extension = { context };
  const common = {
    extensionId: "com.example.ui", contributionId: "open",
    payload: { fields: {} }, context: { locale: "en" },
  };
  const view = await invokeUiAction(extension, { ...common, actionId: "open" });
  assert.equal(view.view.actionId, "com.example.ui.confirm");
  const confirmed = await invokeUiAction(extension, { ...common, actionId: "confirm" });
  assert.equal(confirmed.message.default, "Confirmed");
});

test("a returned view cannot steal an action owned by another contribution", async () => {
  const context = createExtensionApi(specification());
  context.api.ui.register(action("first"));
  context.api.ui.register(action("second"));
  context.api.ui.onAction("first", () => ({
    type: "view",
    view: {
      type: "button", id: "stolen", label: { default: "Stolen" },
      actionId: "second",
    },
  }));
  context.api.ui.onAction("second", () => ({
    type: "notification", level: "success", message: { default: "Second" },
  }));
  await assert.rejects(invokeUiAction({ context }, {
    extensionId: "com.example.ui",
    contributionId: "first",
    actionId: "first",
    payload: { fields: {} },
    context: { locale: "en" },
  }), /invalid_ui_action_result/u);
});

test("returned views keep the action limit extension-wide", async () => {
  const context = createExtensionApi(specification());
  context.api.ui.register(actionPanel("first", 32));
  context.api.ui.register(actionPanel("second", 32));
  context.api.ui.onAction("first-action-0", () => ({
    type: "view",
    view: {
      type: "button", id: "overflow", label: { default: "Overflow" },
      actionId: "overflow",
    },
  }));
  await assert.rejects(invokeUiAction({ context }, {
    extensionId: "com.example.ui",
    contributionId: "first",
    actionId: "first-action-0",
    payload: { fields: {} },
    context: { locale: "en" },
  }), /invalid_ui_action_result/u);
});

test("timed out handlers keep bounded admission until settle and reset changes generation", async () => {
  const resolvers = [];
  const context = createExtensionApi(specification());
  context.api.ui.register(action("slow"));
  context.api.ui.onAction("slow", () => new Promise((resolve) => resolvers.push(resolve)));
  const extension = { context };
  const params = {
    extensionId: "com.example.ui", contributionId: "slow", actionId: "slow",
    payload: { fields: {} }, context: { locale: "en" },
  };
  await assert.rejects(invokeUiAction(extension, params, 2), /ui_action_timeout/u);
  await assert.rejects(invokeUiAction(extension, params, 2), /ui_action_denied/u);

  for (let index = 1; index < LIMITS.maxInFlightHandlers; index += 1) {
    clearUiActions();
    await assert.rejects(invokeUiAction(extension, params, 2), /ui_action_timeout/u);
  }
  clearUiActions();
  await assert.rejects(invokeUiAction(extension, params, 2), /ui_action_denied/u);
  for (const resolve of resolvers) {
    resolve({ type: "notification", level: "info", message: { default: "settled" } });
  }
  await new Promise((resolve) => setImmediate(resolve));
});

test("invalid UI is rejected without losing valid tools", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-ui-partial-"));
  const source = join(directory, "index.mjs");
  await writeFile(source, `export default function (api) {
    api.registerTool({
      name: "healthy",
      description: "Healthy tool",
      parameters: { type: "object" },
      effect: "read-only",
      execute() { return "healthy"; }
    });
    api.ui.register({
      type: "action",
      id: "broken",
      placement: "app.toolbar.primary",
      order: 0,
      label: { default: "x".repeat(${UI_LIMITS.maxTextChars + 1}) },
      actionId: "broken"
    });
  }`);
  const host = createHost(hostScript);
  try {
    const loaded = (await resetAndLoad(host, [{
      ...specification("com.example.partial"),
      mainPath: source,
    }])).extensions[0];
    assert.equal(loaded.error, undefined);
    assert.equal(loaded.contributions.tools[0].name, "com.example.partial.healthy");
    assert.deepEqual(loaded.contributions.ui, []);
    assert.equal(loaded.uiDiagnostics[0].code, "ui_contribution_invalid");
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});

test("the standard fixture crosses the host and routes only to its owner", async () => {
  const fixture = join(root, "src-tauri/tests/fixtures/extensions/ui-standard");
  const manifest = JSON.parse(await readFile(join(fixture, "beaver-extension.json"), "utf8"));
  const host = createHost(hostScript);
  try {
    const loaded = (await resetAndLoad(host, [{
      id: "ui-standard-proof",
      mainPath: join(fixture, "index.mjs"),
      manifest,
    }])).extensions[0];
    assert.equal(loaded.contributions.ui[0].id, "ui-standard-proof.toolbar-proof");

    const result = await host.request("ui.action", {
      extensionId: "ui-standard-proof",
      contributionId: "ui-standard-proof.toolbar-proof",
      actionId: "ui-standard-proof.run-proof",
      payload: { fields: { value: "Node" } },
      context: { locale: "fr" },
    });
    assert.equal(result.message.fr, "Preuve Node");
    await assert.rejects(host.request("ui.action", {
      extensionId: "com.example.other",
      contributionId: "ui-standard-proof.toolbar-proof",
      actionId: "ui-standard-proof.run-proof",
      payload: { fields: {} },
      context: { locale: "fr" },
    }));
  } finally {
    host.stop();
  }
});

test("action payloads and results fail closed above their byte limits", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-ui-bounds-"));
  const source = join(directory, "index.mjs");
  await writeFile(source, `export default function (api) {
    api.ui.register(${JSON.stringify(action("bounded"))});
    api.ui.onAction("bounded", () => ({
      type: "notification",
      level: "info",
      message: { default: "x".repeat(${UI_LIMITS.maxActionResultBytes}) }
    }));
  }`);
  const host = createHost(hostScript);
  try {
    const loaded = (await resetAndLoad(host, [{ ...specification(), mainPath: source }]))
      .extensions[0];
    assert.equal(loaded.error, undefined);
    assert.equal(loaded.contributions.ui[0].id, "com.example.ui.bounded");
    await assert.rejects(host.request("ui.action", {
      extensionId: "com.example.ui",
      contributionId: "com.example.ui.bounded",
      actionId: "com.example.ui.bounded",
      payload: { fields: { value: "x".repeat(UI_LIMITS.maxActionPayloadBytes) } },
      context: { locale: "en" },
    }));
    await assert.rejects(host.request("ui.action", {
      extensionId: "com.example.ui",
      contributionId: "com.example.ui.bounded",
      actionId: "com.example.ui.bounded",
      payload: { fields: {} },
      context: { locale: "en" },
    }));
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});
