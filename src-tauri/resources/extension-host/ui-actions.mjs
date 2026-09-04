import { LIMITS, TIMEOUTS } from "./contract.mjs";
import { UI_LIMITS } from "./ui-contract.mjs";
import {
  actionIdsInResult,
  canonicalId,
  validateActionContext,
  validateActionPayload,
  validateActionResult,
} from "./ui-validation.mjs";

const executions = new Set();
const executionsByExtension = new Map();
const admissions = new Set();
let generation = 0;

export async function invokeUiAction(
  extension,
  params,
  timeoutMs = TIMEOUTS.uiActionTimeoutMs,
) {
  if (!extension || params?.extensionId !== extension.context.ui.extensionId
    || !Number.isSafeInteger(timeoutMs) || timeoutMs < 1
    || timeoutMs > TIMEOUTS.uiActionTimeoutMs) {
    throw new Error("ui_action_denied");
  }
  const contributionId = canonicalId(params.extensionId, params.contributionId);
  const actionId = canonicalId(params.extensionId, params.actionId);
  const contribution = extension.context.ui.contributions
    .find((item) => item.id === contributionId
      && contributionOwnsAction(extension.context.ui, item, actionId));
  const handler = extension.context.ui.handlers.get(actionId);
  const admission = `${generation}:${params.extensionId}:${contributionId}`;
  const extensionExecutions = executionsByExtension.get(params.extensionId) ?? new Set();
  if (!contribution || !handler || admissions.has(admission)
    || extensionExecutions.size >= UI_LIMITS.maxInFlightActionsPerExtension
    || executions.size >= LIMITS.maxInFlightHandlers) {
    throw new Error("ui_action_denied");
  }
  validateActionPayload(params.payload);
  validateActionContext(params.context);
  const execution = Promise.resolve().then(() => handler(params.payload, params.context));
  admissions.add(admission);
  executions.add(execution);
  extensionExecutions.add(execution);
  executionsByExtension.set(params.extensionId, extensionExecutions);
  void execution.finally(() => {
    admissions.delete(admission);
    executions.delete(execution);
    extensionExecutions.delete(execution);
    if (extensionExecutions.size === 0) executionsByExtension.delete(params.extensionId);
  }).catch(() => {});
  let timer;
  try {
    const raw = await Promise.race([
      execution,
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error("ui_action_timeout")), timeoutMs);
        timer.unref();
      }),
    ]);
    const result = validateActionResult(params.extensionId, raw);
    const dynamicActions = actionIdsInResult(result);
    if ([...dynamicActions].some((action) => extension.context.ui.contributions
      .some((item) => item !== contribution
        && contributionOwnsAction(extension.context.ui, item, action)))) {
      throw new Error("invalid_ui_action_result");
    }
    const allActions = new Set(extension.context.ui.declaredActions.keys());
    for (const [owner, actions] of extension.context.ui.dynamicActions) {
      if (owner === contributionId) continue;
      for (const action of actions) allActions.add(action);
    }
    for (const action of dynamicActions) allActions.add(action);
    if (allActions.size > UI_LIMITS.maxActionsPerExtension) {
      throw new Error("invalid_ui_action_result");
    }
    extension.context.ui.dynamicActions.set(contributionId, dynamicActions);
    return result;
  } finally {
    clearTimeout(timer);
  }
}

export function clearUiActions() {
  generation += 1;
  if (!Number.isSafeInteger(generation)) generation = 0;
}

function contributionOwnsAction(ui, contribution, actionId) {
  if (contribution.actionId === actionId) return true;
  if (ui.dynamicActions.get(contribution.id)?.has(actionId)) return true;
  return [contribution.list, contribution.detail].some((view) => viewHasAction(view, actionId));
}

function viewHasAction(node, actionId) {
  if (!node || typeof node !== "object") return false;
  if (node.type === "button") return node.actionId === actionId;
  return Array.isArray(node.children)
    && node.children.some((child) => viewHasAction(child, actionId));
}
