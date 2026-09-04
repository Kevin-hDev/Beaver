import {
  UI_LIMITS,
  UI_PRIMITIVES,
} from "@/types/extension-ui-contract.generated";
import { exact, fieldValue, invalid, localized, ownedIdentifier, plain } from "./parse-utils";
import type { StandardView } from "./types";

interface ViewBudget { nodes: number; fields: number; ids: Set<string> }

export function parseStandardView(extensionId: string, value: unknown): StandardView {
  return visit(extensionId, value, 1, { nodes: 0, fields: 0, ids: new Set() });
}

function visit(
  extensionId: string,
  value: unknown,
  depth: number,
  budget: ViewBudget,
): StandardView {
  budget.nodes += 1;
  if (budget.nodes > UI_LIMITS.maxViewNodes || depth > UI_LIMITS.maxViewDepth) throw invalid();
  const input = plain(value);
  if (typeof input.type !== "string" || !UI_PRIMITIVES.includes(input.type as never)) throw invalid();
  if (input.type === "stack" || input.type === "row") {
    exact(input, ["type", "children"]);
    if (!Array.isArray(input.children)) throw invalid();
    return { type: input.type, children: input.children.map((child) =>
      visit(extensionId, child, depth + 1, budget)) };
  }
  if (input.type === "heading" || input.type === "text" || input.type === "badge") {
    exact(input, ["type", "text"]);
    return { type: input.type, text: localized(input.text) };
  }
  if (input.type === "separator") {
    exact(input, ["type"]);
    return { type: "separator" };
  }
  budget.fields += 1;
  if (budget.fields > UI_LIMITS.maxFieldsPerView) throw invalid();
  const id = ownedIdentifier(extensionId, input.id);
  if (budget.ids.has(id)) throw invalid();
  budget.ids.add(id);
  const label = localized(input.label);
  if (input.type === "button") {
    exact(input, ["type", "id", "label", "actionId"]);
    return { type: "button", id, label, actionId: ownedIdentifier(extensionId, input.actionId) };
  }
  if (!(["textField", "numberField", "toggle", "select"] as const)
    .includes(input.type as never)) throw invalid();
  const fieldType = input.type as "textField" | "numberField" | "toggle" | "select";
  const allowed = ["type", "id", "label", "value", ...(input.type === "select" ? ["options"] : [])];
  exact(input, allowed);
  const parsed = { type: fieldType, id, label, value: fieldValue(input.value) };
  if (!validInitialValue(fieldType, parsed.value)) throw invalid();
  if (fieldType !== "select") return parsed as StandardView;
  if (!Array.isArray(input.options) || input.options.length > UI_LIMITS.maxOptionsPerField) throw invalid();
  const optionValues = new Set<string>();
  const options = input.options.map((option) => {
    const item = plain(option);
    exact(item, ["value", "label"]);
    if (typeof item.value !== "string"
      || Array.from(item.value).length > UI_LIMITS.maxTextChars
      || optionValues.has(item.value)) throw invalid();
    optionValues.add(item.value);
    return { value: item.value, label: localized(item.label) };
  });
  if (typeof parsed.value === "string" && !optionValues.has(parsed.value)) throw invalid();
  return { ...parsed, type: "select", options };
}

function validInitialValue(
  type: "textField" | "numberField" | "toggle" | "select",
  value: ReturnType<typeof fieldValue>,
): boolean {
  if (value === null) return true;
  if (type === "textField" || type === "select") return typeof value === "string";
  if (type === "numberField") return typeof value === "number";
  return typeof value === "boolean";
}
