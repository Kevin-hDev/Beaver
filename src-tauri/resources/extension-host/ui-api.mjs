import { UI_LIMITS } from "./ui-contract.mjs";
import {
  actionIdsInContribution,
  canonicalId,
  jsonBytes,
  normalizeContribution,
  validateUiManifest,
} from "./ui-validation.mjs";

const MAX_UI_DIAGNOSTICS = UI_LIMITS.maxContributionsPerExtension
  + UI_LIMITS.maxActionsPerExtension;

export function createUiApi(specification) {
  const contributions = [];
  const handlers = new Map();
  const declaredActions = new Map();
  const dynamicActions = new Map();
  const diagnostics = [];
  const enabled = validateUiManifest(specification.manifest?.ui);

  function register(input) {
    if (!enabled) return rejected("ui_contribution_invalid");
    if (contributions.length >= UI_LIMITS.maxContributionsPerExtension) {
      return rejected("ui_limit_exceeded");
    }
    let contribution;
    try {
      contribution = normalizeContribution(specification.id, input);
    } catch (error) {
      return rejected(error?.message === "ui_limit_exceeded"
        ? "ui_limit_exceeded"
        : "ui_contribution_invalid");
    }
    if (contributions.some((item) => item.id === contribution.id)) {
      return rejected("ui_contribution_invalid");
    }
    const actions = actionIdsInContribution(contribution);
    if (declaredActions.size + actions.size > UI_LIMITS.maxActionsPerExtension) {
      return rejected("ui_limit_exceeded");
    }
    if ([...actions].some((action) => declaredActions.has(action))) {
      return rejected("ui_contribution_invalid");
    }
    if (contribution.type === "theme"
      && contributions.filter((item) => item.type === "theme").length
        >= UI_LIMITS.maxThemesPerExtension) {
      return rejected("ui_limit_exceeded");
    }
    if (jsonBytes([...contributions, contribution]) > UI_LIMITS.maxUiBytesPerExtension) {
      return rejected("ui_limit_exceeded");
    }
    contributions.push(contribution);
    for (const action of actions) declaredActions.set(action, contribution);
    let active = true;
    return () => {
      if (!active) return;
      active = false;
      const index = contributions.indexOf(contribution);
      if (index >= 0) contributions.splice(index, 1);
      for (const action of actions) {
        if (declaredActions.get(action) === contribution) declaredActions.delete(action);
      }
      dynamicActions.delete(contribution.id);
    };
  }

  function onAction(actionId, handler) {
    let id;
    try {
      id = canonicalId(specification.id, actionId);
    } catch {
      return rejected("ui_contribution_invalid");
    }
    if (!enabled || typeof handler !== "function" || handlers.size >= UI_LIMITS.maxActionsPerExtension) {
      return rejected(handlers.size >= UI_LIMITS.maxActionsPerExtension
        ? "ui_limit_exceeded"
        : "ui_contribution_invalid");
    }
    if (handlers.has(id)) return rejected("ui_contribution_invalid");
    handlers.set(id, handler);
    let active = true;
    return () => {
      if (!active) return;
      active = false;
      if (handlers.get(id) === handler) handlers.delete(id);
    };
  }

  function rejected(code) {
    if (diagnostics.length < MAX_UI_DIAGNOSTICS) diagnostics.push({ code });
    return () => {};
  }

  return {
    api: Object.freeze({ register, onAction }),
    contributions,
    handlers,
    declaredActions,
    dynamicActions,
    diagnostics,
    extensionId: specification.id,
  };
}
