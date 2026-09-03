import {
  EXTENSION_UI_API_VERSION, UI_CONTRIBUTION_TYPES, UI_ICONS, UI_LIMITS,
  UI_LOCALES, UI_PLACEMENTS, UI_THEME_BASES, UI_THEME_TOKENS, UI_VALIDATION,
} from "./ui-contract.mjs";
import {
  exactKeys, localizedText, normalizeView, plain, validId,
} from "./ui-view-validation.mjs";

const LEVELS = new Set(["info", "success", "warning", "error"]);
const LOCALES = new Set(UI_LOCALES);
const ICONS = new Set(UI_ICONS);
const THEME_BASES = new Set(UI_THEME_BASES);
const THEME_TOKENS = new Set(UI_THEME_TOKENS);
const CONTRIBUTION_TYPES = new Set(UI_CONTRIBUTION_TYPES);
const PLACEMENTS = new Map(UI_PLACEMENTS.map((entry) => [entry.key, entry]));

export function canonicalId(extensionId, value) {
  if (!validId(extensionId) || !validId(value)) throw new Error("invalid_ui_id");
  const canonical = value.startsWith(`${extensionId}.`) ? value : `${extensionId}.${value}`;
  if (!validId(canonical)) throw new Error("invalid_ui_id");
  return canonical;
}

export function validateUiManifest(manifest) {
  return plain(manifest) && exactKeys(manifest, ["apiVersion", "mode", "entry"])
    && manifest.apiVersion === EXTENSION_UI_API_VERSION && manifest.mode === "standard"
    // Rust serializes absent optional registry fields as null so a merge cannot
    // resurrect a stale advanced entry. Standard UI must accept that wire form.
    && (manifest.entry === undefined || manifest.entry === null);
}

export function normalizeContribution(extensionId, input) {
  if (!plain(input) || !CONTRIBUTION_TYPES.has(input.type)) {
    throw new Error("invalid_ui_contribution");
  }
  const contribution = cloneJson(input);
  contribution.id = canonicalId(extensionId, contribution.id);
  if (typeof contribution.actionId === "string") {
    contribution.actionId = canonicalId(extensionId, contribution.actionId);
  }
  validateContribution(extensionId, contribution);
  if (jsonBytes(contribution) > UI_LIMITS.maxUiBytesPerExtension) {
    throw new Error("ui_limit_exceeded");
  }
  return contribution;
}

export function validateActionPayload(payload) {
  if (!plain(payload) || !exactKeys(payload, ["fields"]) || !plain(payload.fields)) {
    throw new Error("invalid_ui_action_payload");
  }
  const entries = Object.entries(payload.fields);
  if (entries.length > UI_LIMITS.maxFieldsPerView) throw new Error("ui_limit_exceeded");
  for (const [key, value] of entries) {
    if (!validId(key) || !fieldValue(value)) throw new Error("invalid_ui_action_payload");
  }
  if (jsonBytes(payload) > UI_LIMITS.maxActionPayloadBytes) throw new Error("ui_limit_exceeded");
}

export function validateActionResult(extensionId, result) {
  const normalized = cloneJson(result);
  if (!plain(normalized)) {
    throw new Error("invalid_ui_action_result");
  }
  if (normalized.type === "notification") {
    if (!exactKeys(normalized, ["type", "level", "message"])
      || !LEVELS.has(normalized.level) || !localizedText(normalized.message)) {
      throw new Error("invalid_ui_action_result");
    }
  } else if (normalized.type === "view") {
    if (!exactKeys(normalized, ["type", "view"])) throw new Error("invalid_ui_action_result");
    normalizeView(extensionId, normalized.view, canonicalId);
  } else throw new Error("invalid_ui_action_result");
  if (jsonBytes(normalized) > UI_LIMITS.maxActionResultBytes) {
    throw new Error("invalid_ui_action_result");
  }
  return normalized;
}

export function actionIdsInResult(result) {
  return result?.type === "view" ? actionIdsInView(result.view) : new Set();
}

export function actionIdsInContribution(contribution) {
  const actions = new Set();
  if (typeof contribution?.actionId === "string") actions.add(contribution.actionId);
  for (const view of [contribution?.list, contribution?.detail]) {
    for (const action of actionIdsInView(view)) actions.add(action);
  }
  return actions;
}

function actionIdsInView(root) {
  const actions = new Set();
  function visit(node) {
    if (!node || typeof node !== "object") return;
    if (node.type === "button") actions.add(node.actionId);
    if (Array.isArray(node.children)) node.children.forEach(visit);
  }
  visit(root);
  return actions;
}

export function validateActionContext(context) {
  if (!plain(context) || !exactKeys(context, ["locale"]) || !LOCALES.has(context.locale)) {
    throw new Error("invalid_ui_action_context");
  }
}

function validateContribution(extensionId, value) {
  if (!validId(value.id) || !Number.isInteger(value.order)
    || value.order < UI_VALIDATION.minOrder || value.order > UI_VALIDATION.maxOrder) {
    throw new Error("invalid_ui_contribution");
  }
  if (value.type === "theme") return validateTheme(value);
  const allowed = ["type", "id", "placement", "order", "label", "icon", "operation", "targetId"];
  if (value.type === "tab") allowed.push("list", "detail");
  if (value.type === "settingsTab") allowed.push("detail");
  if (value.type === "action") allowed.push("actionId");
  const placement = PLACEMENTS.get(value.placement);
  if (!exactKeys(value, allowed) || placement?.contributionType !== value.type
    || !localizedText(value.label) || (value.icon !== undefined && !ICONS.has(value.icon))) {
    throw new Error("invalid_ui_contribution");
  }
  normalizePlacementMutation(extensionId, value);
  if (value.type === "tab") {
    if (value.list !== undefined) normalizeView(extensionId, value.list, canonicalId);
    normalizeView(extensionId, value.detail, canonicalId);
  } else if (value.type === "settingsTab") {
    normalizeView(extensionId, value.detail, canonicalId);
  } else if (value.type === "action") {
    value.actionId = canonicalId(extensionId, value.actionId);
  }
}

function normalizePlacementMutation(extensionId, value) {
  const hasMutation = value.operation !== undefined || value.targetId !== undefined;
  if (!hasMutation) return;
  if (!["before", "after", "replace", "move", "remove"].includes(value.operation)
    || typeof value.targetId !== "string") throw new Error("invalid_ui_contribution");
  value.targetId = value.targetId.startsWith("beaver.")
    ? canonicalId("beaver", value.targetId)
    : canonicalId(extensionId, value.targetId);
}

function validateTheme(value) {
  if (!exactKeys(value, ["type", "id", "order", "label", "base", "tokens"])
    || !localizedText(value.label) || !THEME_BASES.has(value.base) || !plain(value.tokens)) {
    throw new Error("invalid_ui_contribution");
  }
  const tokens = Object.entries(value.tokens);
  if (tokens.length > UI_LIMITS.maxThemeTokens) throw new Error("ui_limit_exceeded");
  const pattern = new RegExp(UI_VALIDATION.themeValuePattern, "u");
  if (tokens.some(([key, token]) => !THEME_TOKENS.has(key)
    || typeof token !== "string" || !pattern.test(token))) {
    throw new Error("invalid_ui_contribution");
  }
}

function fieldValue(value) {
  return value === null || typeof value === "boolean" || Number.isFinite(value)
    || (typeof value === "string" && [...value].length <= UI_LIMITS.maxTextChars);
}

export function jsonBytes(value) {
  try { return Buffer.byteLength(JSON.stringify(value), "utf8"); }
  catch { throw new Error("invalid_ui_json"); }
}

function cloneJson(value) {
  try { return JSON.parse(JSON.stringify(value)); }
  catch { throw new Error("invalid_ui_json"); }
}
