import { LIMITS } from "./contract.mjs";
import {
  UI_LIMITS,
  UI_LOCALES,
  UI_PRIMITIVES,
} from "./ui-contract.mjs";

const IDENTIFIER = /^[a-zA-Z0-9](?:[a-zA-Z0-9._-]*[a-zA-Z0-9])?$/u;
const PRIMITIVES = new Set(UI_PRIMITIVES);

export function normalizeView(extensionId, root, canonicalId) {
  let nodes = 0;
  let fields = 0;
  function visit(node, depth) {
    nodes += 1;
    if (nodes > UI_LIMITS.maxViewNodes || depth > UI_LIMITS.maxViewDepth
      || !plain(node) || !PRIMITIVES.has(node.type)) throw new Error("ui_limit_exceeded");
    if (node.type === "stack" || node.type === "row") {
      if (!exactKeys(node, ["type", "children"]) || !Array.isArray(node.children)) {
        throw new Error("invalid_ui_view");
      }
      for (const child of node.children) visit(child, depth + 1);
      return;
    }
    if (["heading", "text", "badge"].includes(node.type)) {
      if (!exactKeys(node, ["type", "text"]) || !localizedText(node.text)) {
        throw new Error("invalid_ui_view");
      }
      return;
    }
    if (node.type === "separator" && exactKeys(node, ["type"])) return;
    fields += 1;
    if (fields > UI_LIMITS.maxFieldsPerView) throw new Error("ui_limit_exceeded");
    normalizeFieldOrButton(extensionId, node, canonicalId);
  }
  visit(root, 1);
}

function normalizeFieldOrButton(extensionId, node, canonicalId) {
  const common = ["type", "id", "label"];
  if (node.type === "button") {
    if (!exactKeys(node, [...common, "actionId"]) || !localizedText(node.label)) {
      throw new Error("invalid_ui_view");
    }
    node.id = canonicalId(extensionId, node.id);
    node.actionId = canonicalId(extensionId, node.actionId);
    return;
  }
  if (!["textField", "numberField", "select", "toggle"].includes(node.type)
    || !exactKeys(node, [...common, "value", ...(node.type === "select" ? ["options"] : [])])
    || !localizedText(node.label) || !fieldValue(node.value)) {
    throw new Error("invalid_ui_view");
  }
  node.id = canonicalId(extensionId, node.id);
  if (node.type === "select" && (!Array.isArray(node.options)
    || node.options.length > UI_LIMITS.maxOptionsPerField
    || node.options.some((option) => !plain(option)
      || !exactKeys(option, ["value", "label"])
      || typeof option.value !== "string"
      || [...option.value].length > UI_LIMITS.maxTextChars
      || !localizedText(option.label)))) {
    throw new Error("invalid_ui_view");
  }
}

export function localizedText(value) {
  if (!plain(value) || !exactKeys(value, ["default", ...UI_LOCALES])
    || typeof value.default !== "string" || value.default.length === 0) return false;
  return Object.values(value).every((text) => typeof text === "string"
    && text.length > 0 && [...text].length <= UI_LIMITS.maxTextChars);
}

export function validId(value) {
  return typeof value === "string"
    && value.length <= LIMITS.maxIdentifierChars
    && IDENTIFIER.test(value);
}

export function plain(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    && (Object.getPrototypeOf(value) === Object.prototype || Object.getPrototypeOf(value) === null);
}

export function exactKeys(value, allowed) {
  const keys = Object.keys(value);
  return keys.length <= allowed.length && keys.every((key) => allowed.includes(key));
}

function fieldValue(value) {
  return value === null || typeof value === "boolean" || Number.isFinite(value)
    || (typeof value === "string" && [...value].length <= UI_LIMITS.maxTextChars);
}
