import {
  UI_CONTRIBUTION_TYPES,
  UI_ICONS,
  UI_LIMITS,
  UI_PLACEMENTS,
  UI_PLACEMENT_OPERATIONS,
  UI_THEME_BASES,
  UI_THEME_TOKENS,
  UI_VALIDATION,
} from "@/types/extension-ui-contract.generated";
import { exact, identifier, invalid, jsonBytes, localized, ownedIdentifier, plain } from "./parse-utils";
import { parseStandardView } from "./view-parser";
import type {
  StandardCatalogEntry,
  StandardCatalogSnapshot,
  StandardContribution,
  StandardThemeContribution,
} from "./types";

export function parseStandardCatalog(value: unknown): StandardCatalogSnapshot {
  const input = plain(value);
  exact(input, ["revision", "contributions"]);
  if (!Number.isSafeInteger(input.revision) || (input.revision as number) < 0
    || !Array.isArray(input.contributions)
    || input.contributions.length > UI_LIMITS.maxGlobalStandardContributions
    || jsonBytes(value) > UI_LIMITS.maxGlobalUiBytes) throw invalid();
  const entries = input.contributions.map(parseEntry);
  const keys = new Set<string>();
  const perExtension = new Map<string, StandardCatalogEntry[]>();
  const themes = new Map<string, number>();
  const actions = new Map<string, Set<string>>();
  for (const entry of entries) {
    const key = `${entry.extensionId}:${entry.contributionId}`;
    if (keys.has(key)) throw invalid();
    keys.add(key);
    const group = perExtension.get(entry.extensionId) ?? [];
    group.push(entry);
    if (group.length > UI_LIMITS.maxContributionsPerExtension
      || jsonBytes(group.map((item) => item.contribution)) > UI_LIMITS.maxUiBytesPerExtension) {
      throw invalid();
    }
    const themeCount = (themes.get(entry.extensionId) ?? 0)
      + Number(entry.contribution.type === "theme");
    if (themeCount > UI_LIMITS.maxThemesPerExtension) throw invalid();
    themes.set(entry.extensionId, themeCount);
    const declaredActions = actions.get(entry.extensionId) ?? new Set<string>();
    collectActionIds(entry.contribution, declaredActions);
    if (declaredActions.size > UI_LIMITS.maxActionsPerExtension) throw invalid();
    actions.set(entry.extensionId, declaredActions);
    perExtension.set(entry.extensionId, group);
  }
  return { revision: input.revision as number, contributions: entries };
}

function collectActionIds(value: unknown, actions: Set<string>): void {
  if (!value || typeof value !== "object") return;
  const object = value as Record<string, unknown>;
  if (typeof object.actionId === "string") actions.add(object.actionId);
  if (Array.isArray(object.children)) {
    for (const child of object.children) collectActionIds(child, actions);
  }
  collectActionIds(object.list, actions);
  collectActionIds(object.detail, actions);
}

function parseEntry(value: unknown): StandardCatalogEntry {
  const input = plain(value);
  exact(input, ["extensionId", "contributionId", "contribution"]);
  const extensionId = identifier(input.extensionId);
  const contributionId = ownedIdentifier(extensionId, input.contributionId);
  const contribution = parseContribution(extensionId, input.contribution);
  if (contribution.id !== contributionId) throw invalid();
  return { extensionId, contributionId, contribution };
}

function parseContribution(
  extensionId: string,
  value: unknown,
): StandardContribution | StandardThemeContribution {
  const input = plain(value);
  if (typeof input.type !== "string" || !UI_CONTRIBUTION_TYPES.includes(input.type as never)) {
    throw invalid();
  }
  if (!Number.isInteger(input.order)
    || (input.order as number) < UI_VALIDATION.minOrder
    || (input.order as number) > UI_VALIDATION.maxOrder) throw invalid();
  const id = ownedIdentifier(extensionId, input.id);
  const label = localized(input.label);
  if (input.type === "theme") return parseTheme(input, id, label);
  const placement = UI_PLACEMENTS.find(({ key }) => key === input.placement);
  if (!placement || placement.contributionType !== input.type) throw invalid();
  const icon = input.icon === undefined ? undefined : UI_ICONS.find((item) => item === input.icon);
  if (input.icon !== undefined && !icon) throw invalid();
  const mutation = parseMutation(extensionId, input);
  const common = { id, type: input.type, placement: placement.key, order: input.order as number,
    label, ...(icon ? { icon } : {}), ...mutation };
  if (input.type === "action") {
    exact(input, [...commonKeys(), "actionId"]);
    return { ...common, type: "action", actionId: ownedIdentifier(extensionId, input.actionId) };
  }
  if (input.type === "settingsTab") {
    exact(input, [...commonKeys(), "detail"]);
    return { ...common, type: "settingsTab", detail: parseStandardView(extensionId, input.detail) };
  }
  exact(input, [...commonKeys(), "list", "detail"]);
  return { ...common, type: "tab",
    ...(input.list === undefined ? {} : { list: parseStandardView(extensionId, input.list) }),
    detail: parseStandardView(extensionId, input.detail) };
}

function parseMutation(extensionId: string, input: Record<string, unknown>) {
  if (input.operation === undefined && input.targetId === undefined) return {};
  if (typeof input.operation !== "string"
    || !UI_PLACEMENT_OPERATIONS.includes(input.operation as never)
    || typeof input.targetId !== "string") throw invalid();
  const targetId = input.targetId.startsWith("beaver.")
    ? identifier(input.targetId)
    : ownedIdentifier(extensionId, input.targetId);
  return { operation: input.operation as typeof UI_PLACEMENT_OPERATIONS[number], targetId };
}

function parseTheme(
  input: Record<string, unknown>, id: string, label: ReturnType<typeof localized>,
): StandardThemeContribution {
  exact(input, ["type", "id", "order", "label", "base", "tokens"]);
  if (typeof input.base !== "string" || !UI_THEME_BASES.includes(input.base as never)) throw invalid();
  const tokens = plain(input.tokens);
  if (Object.keys(tokens).length > UI_LIMITS.maxThemeTokens) throw invalid();
  for (const [name, token] of Object.entries(tokens)) {
    if (!UI_THEME_TOKENS.includes(name as never) || typeof token !== "string"
      || !validThemeToken(token)) throw invalid();
  }
  return { type: "theme", id, order: input.order as number, label,
    base: input.base as "light" | "dark", tokens: tokens as Record<string, string> };
}

function validThemeToken(value: string): boolean {
  return matchesContractThemePattern(value)
    && UI_VALIDATION.themeValuePattern === "^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$";
}

function matchesContractThemePattern(value: string): boolean {
  return (value.length === 7 || value.length === 9) && value.startsWith("#")
    && Array.from(value.slice(1)).every((character) => /[0-9A-Fa-f]/u.test(character));
}

function commonKeys(): string[] {
  return ["type", "id", "placement", "order", "label", "icon", "operation", "targetId"];
}
